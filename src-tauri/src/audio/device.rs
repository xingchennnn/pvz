use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig, SupportedStreamConfig};
use ndarray::Array3;
use once_cell::sync::OnceCell;
use onnxruntime::{environment::Environment, LoggingLevel};
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;
use rustfft::{num_complex::Complex, FftPlanner};
use serde::Serialize;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const MODEL_SAMPLE_RATE: u32 = 16_000;
const FFT_SIZE: usize = 400;
const HOP_SIZE: usize = 160;
const RING_CAPACITY: usize = MODEL_SAMPLE_RATE as usize * 4;

struct AudioState {
    handle: Option<JoinHandle<()>>,
    running: Arc<AtomicBool>,
    enabled: Arc<AtomicBool>,
    input_stream: Option<Stream>,
    output_stream: Option<Stream>,
    input_device_name: String,
    output_device_name: String,
    input_channels: u16,
    output_channels: u16,
    input_sample_format: String,
    output_sample_format: String,
    sample_rate: u32,
    stats: Arc<AudioStatsCounters>,
}

#[derive(Default)]
struct AudioStatsCounters {
    input_overflows: AtomicU64,
    output_underflows: AtomicU64,
    inference_frames: AtomicU64,
    inference_errors: AtomicU64,
    total_inference_micros: AtomicU64,
    last_inference_micros: AtomicU64,
}

#[derive(Serialize)]
pub struct AudioStatsSnapshot {
    input_overflows: u64,
    output_underflows: u64,
    inference_frames: u64,
    inference_errors: u64,
    average_inference_ms: f64,
    last_inference_ms: f64,
}

#[derive(Serialize)]
pub struct AudioStatus {
    running: bool,
    listening: bool,
    sample_rate: u32,
    model_sample_rate: u32,
    input_device_name: Option<String>,
    output_device_name: Option<String>,
    input_channels: Option<u16>,
    output_channels: Option<u16>,
    input_sample_format: Option<String>,
    output_sample_format: Option<String>,
    stats: AudioStatsSnapshot,
}

#[derive(Serialize)]
pub struct AudioDeviceInfo {
    name: String,
    compatible: bool,
    supported_sample_formats: Vec<String>,
}

#[derive(Serialize)]
pub struct AudioDeviceCatalog {
    model_sample_rate: u32,
    default_input: Option<String>,
    default_output: Option<String>,
    inputs: Vec<AudioDeviceInfo>,
    outputs: Vec<AudioDeviceInfo>,
}

static AUDIO_THREAD: OnceCell<Arc<Mutex<Option<AudioState>>>> = OnceCell::new();

fn audio_cell() -> &'static Arc<Mutex<Option<AudioState>>> {
    AUDIO_THREAD.get_or_init(|| Arc::new(Mutex::new(None)))
}

fn stats_snapshot(stats: &AudioStatsCounters) -> AudioStatsSnapshot {
    let inference_frames = stats.inference_frames.load(Ordering::SeqCst);
    let total_inference_micros = stats.total_inference_micros.load(Ordering::SeqCst);
    AudioStatsSnapshot {
        input_overflows: stats.input_overflows.load(Ordering::SeqCst),
        output_underflows: stats.output_underflows.load(Ordering::SeqCst),
        inference_frames,
        inference_errors: stats.inference_errors.load(Ordering::SeqCst),
        average_inference_ms: if inference_frames == 0 {
            0.0
        } else {
            total_inference_micros as f64 / inference_frames as f64 / 1000.0
        },
        last_inference_ms: stats.last_inference_micros.load(Ordering::SeqCst) as f64 / 1000.0,
    }
}

#[allow(deprecated)]
fn device_name(device: &cpal::Device) -> String {
    device.name().unwrap_or_else(|_| "未知设备".to_string())
}

fn sample_format_label(format: SampleFormat) -> String {
    format!("{format:?}")
}

fn select_supported_config(
    configs: Vec<SupportedStreamConfig>,
) -> Option<SupportedStreamConfig> {
    let mut configs = configs;
    configs.sort_by_key(|cfg| {
        let format_score = match cfg.sample_format() {
            SampleFormat::F32 => 0,
            SampleFormat::I16 => 1,
            SampleFormat::U16 => 2,
            _ => 3,
        };
        (format_score, cfg.channels())
    });
    configs.into_iter().next()
}

fn supported_configs_for_rate(
    device: &cpal::Device,
    is_input: bool,
    sample_rate: u32,
) -> Result<Vec<SupportedStreamConfig>, String> {
    let result = if is_input {
        device
            .supported_input_configs()
            .map_err(|err| err.to_string())?
            .filter(|cfg| cfg.min_sample_rate() <= sample_rate && cfg.max_sample_rate() >= sample_rate)
            .map(|cfg| cfg.with_sample_rate(sample_rate))
            .collect()
    } else {
        device
            .supported_output_configs()
            .map_err(|err| err.to_string())?
            .filter(|cfg| cfg.min_sample_rate() <= sample_rate && cfg.max_sample_rate() >= sample_rate)
            .map(|cfg| cfg.with_sample_rate(sample_rate))
            .collect()
    };

    Ok(result)
}

fn build_input_stream<T, P, F>(
    device: &cpal::Device,
    config: &StreamConfig,
    in_channels: usize,
    enabled: Arc<AtomicBool>,
    mut producer: P,
    stats: Arc<AudioStatsCounters>,
    convert: F,
) -> Result<Stream, String>
where
    T: cpal::SizedSample,
    P: Producer<Item = f32> + Send + 'static,
    F: Fn(T) -> f32 + Send + Sync + Copy + 'static,
{
    device
        .build_input_stream(
            config,
            move |data: &[T], _| {
                if !enabled.load(Ordering::SeqCst) {
                    return;
                }

                for frame in data.chunks(in_channels) {
                    let mono = if in_channels == 1 {
                        convert(frame[0])
                    } else {
                        frame.iter().copied().map(convert).sum::<f32>() / in_channels as f32
                    };

                    if producer.try_push(mono).is_err() {
                        stats.input_overflows.fetch_add(1, Ordering::SeqCst);
                    }
                }
            },
            move |err| eprintln!("输入流错误: {err}"),
            None,
        )
        .map_err(|err| err.to_string())
}

fn build_output_stream<T, C, F>(
    device: &cpal::Device,
    config: &StreamConfig,
    out_channels: usize,
    enabled: Arc<AtomicBool>,
    mut consumer: C,
    stats: Arc<AudioStatsCounters>,
    convert: F,
) -> Result<Stream, String>
where
    T: cpal::SizedSample,
    C: Consumer<Item = f32> + Send + 'static,
    F: Fn(f32) -> T + Send + Sync + Copy + 'static,
{
    device
        .build_output_stream(
            config,
            move |data: &mut [T], _| {
                if !enabled.load(Ordering::SeqCst) {
                    for sample in data.iter_mut() {
                        *sample = convert(0.0);
                    }
                    return;
                }

                for frame in data.chunks_mut(out_channels) {
                    let sample = match consumer.try_pop() {
                        Some(value) => value,
                        None => {
                            stats.output_underflows.fetch_add(1, Ordering::SeqCst);
                            0.0
                        }
                    };

                    for channel in frame.iter_mut() {
                        *channel = convert(sample);
                    }
                }
            },
            move |err| eprintln!("输出流错误: {err}"),
            None,
        )
        .map_err(|err| err.to_string())
}

fn stft_frame(frame: &[f32], fft_size: usize) -> Vec<Complex<f32>> {
    let mut buf: Vec<Complex<f32>> = frame.iter().map(|&x| Complex { re: x, im: 0.0 }).collect();
    buf.resize(fft_size, Complex { re: 0.0, im: 0.0 });
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(fft_size);
    fft.process(&mut buf);
    buf
}

fn stft_audio(audio: &[f32], fft_size: usize, hop_size: usize) -> Result<(Array3<f32>, Array3<f32>), String> {
    let mut frames_real = Vec::new();
    let mut frames_imag = Vec::new();

    let mut pos = 0;
    while pos + fft_size <= audio.len() {
        let frame = &audio[pos..pos + fft_size];
        let spectrum = stft_frame(frame, fft_size);

        let mut real = Vec::with_capacity(fft_size / 2 + 1);
        let mut imag = Vec::with_capacity(fft_size / 2 + 1);

        for &complex in spectrum.iter().take(fft_size / 2 + 1) {
            real.push(complex.re);
            imag.push(complex.im);
        }

        frames_real.push(real);
        frames_imag.push(imag);
        pos += hop_size;
    }

    let frames = frames_real.len();
    let bins = fft_size / 2 + 1;
    if frames == 0 {
        return Err("输入帧不足，无法执行 STFT".into());
    }

    let real_array = Array3::from_shape_vec(
        (1, bins, frames),
        frames_real.into_iter().flatten().collect(),
    )
    .map_err(|err| err.to_string())?;
    let imag_array = Array3::from_shape_vec(
        (1, bins, frames),
        frames_imag.into_iter().flatten().collect(),
    )
    .map_err(|err| err.to_string())?;
    Ok((real_array, imag_array))
}

fn input_device_info(device: &cpal::Device) -> AudioDeviceInfo {
    let supported = supported_configs_for_rate(device, true, MODEL_SAMPLE_RATE).unwrap_or_default();
    AudioDeviceInfo {
        name: device_name(device),
        compatible: !supported.is_empty(),
        supported_sample_formats: supported
            .into_iter()
            .map(|cfg| sample_format_label(cfg.sample_format()))
            .collect(),
    }
}

fn output_device_info(device: &cpal::Device) -> AudioDeviceInfo {
    let supported = supported_configs_for_rate(device, false, MODEL_SAMPLE_RATE).unwrap_or_default();
    AudioDeviceInfo {
        name: device_name(device),
        compatible: !supported.is_empty(),
        supported_sample_formats: supported
            .into_iter()
            .map(|cfg| sample_format_label(cfg.sample_format()))
            .collect(),
    }
}

#[tauri::command]
pub fn list_audio_devices() -> Result<AudioDeviceCatalog, String> {
    let host = cpal::default_host();
    let default_input = host.default_input_device().map(|device| device_name(&device));
    let default_output = host.default_output_device().map(|device| device_name(&device));

    let inputs = host
        .input_devices()
        .map_err(|err| err.to_string())?
        .map(|device| input_device_info(&device))
        .collect();
    let outputs = host
        .output_devices()
        .map_err(|err| err.to_string())?
        .map(|device| output_device_info(&device))
        .collect();

    Ok(AudioDeviceCatalog {
        model_sample_rate: MODEL_SAMPLE_RATE,
        default_input,
        default_output,
        inputs,
        outputs,
    })
}

#[tauri::command]
pub fn get_audio_status() -> AudioStatus {
    let guard = audio_cell().lock().unwrap();
    if let Some(state) = guard.as_ref() {
        return AudioStatus {
            running: state.running.load(Ordering::SeqCst),
            listening: state.enabled.load(Ordering::SeqCst),
            sample_rate: state.sample_rate,
            model_sample_rate: MODEL_SAMPLE_RATE,
            input_device_name: Some(state.input_device_name.clone()),
            output_device_name: Some(state.output_device_name.clone()),
            input_channels: Some(state.input_channels),
            output_channels: Some(state.output_channels),
            input_sample_format: Some(state.input_sample_format.clone()),
            output_sample_format: Some(state.output_sample_format.clone()),
            stats: stats_snapshot(&state.stats),
        };
    }

    AudioStatus {
        running: false,
        listening: false,
        sample_rate: 0,
        model_sample_rate: MODEL_SAMPLE_RATE,
        input_device_name: None,
        output_device_name: None,
        input_channels: None,
        output_channels: None,
        input_sample_format: None,
        output_sample_format: None,
        stats: AudioStatsSnapshot {
            input_overflows: 0,
            output_underflows: 0,
            inference_frames: 0,
            inference_errors: 0,
            average_inference_ms: 0.0,
            last_inference_ms: 0.0,
        },
    }
}

#[tauri::command]
pub fn start_noise_reduction() -> Result<String, String> {
    let mut guard = audio_cell().lock().unwrap();
    if let Some(state) = &*guard {
        if state.running.load(Ordering::SeqCst) {
            return Ok("降噪引擎已经启动".into());
        }
    }

    let host = cpal::default_host();
    let input_device = host
        .default_input_device()
        .ok_or_else(|| "未找到默认输入设备".to_string())?;
    let output_device = host
        .default_output_device()
        .ok_or_else(|| "未找到默认输出设备".to_string())?;

    let input_supported = supported_configs_for_rate(&input_device, true, MODEL_SAMPLE_RATE)?;
    let input_supported = select_supported_config(input_supported)
        .ok_or_else(|| format!("输入设备 {} 不支持 {}Hz", device_name(&input_device), MODEL_SAMPLE_RATE))?;
    let output_supported = supported_configs_for_rate(&output_device, false, MODEL_SAMPLE_RATE)?;
    let output_supported = select_supported_config(output_supported)
        .ok_or_else(|| format!("输出设备 {} 不支持 {}Hz", device_name(&output_device), MODEL_SAMPLE_RATE))?;

    let input_sample_format = input_supported.sample_format();
    let output_sample_format = output_supported.sample_format();
    let input_config: StreamConfig = input_supported.clone().into();
    let output_config: StreamConfig = output_supported.clone().into();
    let input_channels = input_config.channels as usize;
    let output_channels = output_config.channels as usize;

    let running = Arc::new(AtomicBool::new(true));
    let enabled = Arc::new(AtomicBool::new(false));
    let stats = Arc::new(AudioStatsCounters::default());

    let input_ring = HeapRb::<f32>::new(RING_CAPACITY);
    let (producer_in, mut consumer_in) = input_ring.split();
    let output_ring = HeapRb::<f32>::new(RING_CAPACITY);
    let (mut producer_out, consumer_out) = output_ring.split();

    let input_stream = match input_sample_format {
        SampleFormat::F32 => build_input_stream::<f32, _, _>(
            &input_device,
            &input_config,
            input_channels,
            enabled.clone(),
            producer_in,
            stats.clone(),
            |sample| sample,
        ),
        SampleFormat::I16 => build_input_stream::<i16, _, _>(
            &input_device,
            &input_config,
            input_channels,
            enabled.clone(),
            producer_in,
            stats.clone(),
            |sample| sample as f32 / i16::MAX as f32,
        ),
        SampleFormat::U16 => build_input_stream::<u16, _, _>(
            &input_device,
            &input_config,
            input_channels,
            enabled.clone(),
            producer_in,
            stats.clone(),
            |sample| (sample as f32 / u16::MAX as f32) * 2.0 - 1.0,
        ),
        other => Err(format!("暂不支持输入采样格式: {other:?}")),
    }?;

    let output_stream = match output_sample_format {
        SampleFormat::F32 => build_output_stream::<f32, _, _>(
            &output_device,
            &output_config,
            output_channels,
            enabled.clone(),
            consumer_out,
            stats.clone(),
            |sample| sample,
        ),
        SampleFormat::I16 => build_output_stream::<i16, _, _>(
            &output_device,
            &output_config,
            output_channels,
            enabled.clone(),
            consumer_out,
            stats.clone(),
            |sample| (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16,
        ),
        SampleFormat::U16 => build_output_stream::<u16, _, _>(
            &output_device,
            &output_config,
            output_channels,
            enabled.clone(),
            consumer_out,
            stats.clone(),
            |sample| (((sample.clamp(-1.0, 1.0) + 1.0) * 0.5) * u16::MAX as f32) as u16,
        ),
        other => Err(format!("暂不支持输出采样格式: {other:?}")),
    }?;

    let thread_running = running.clone();
    let thread_enabled = enabled.clone();
    let thread_stats = stats.clone();

    let handle = thread::spawn(move || {
        let env = match Environment::builder()
            .with_name("denoise")
            .with_log_level(LoggingLevel::Warning)
            .build()
        {
            Ok(env) => env,
            Err(err) => {
                eprintln!("创建 ONNX 运行环境失败: {err}");
                thread_stats.inference_errors.fetch_add(1, Ordering::SeqCst);
                return;
            }
        };

        let mut session = match env.new_session_builder() {
            Ok(builder) => match builder.with_model_from_file("public/onnx_model.onnx") {
                Ok(session) => session,
                Err(err) => {
                    eprintln!("加载 ONNX 模型失败: {err}");
                    thread_stats.inference_errors.fetch_add(1, Ordering::SeqCst);
                    return;
                }
            },
            Err(err) => {
                eprintln!("创建 ONNX session builder 失败: {err}");
                thread_stats.inference_errors.fetch_add(1, Ordering::SeqCst);
                return;
            }
        };

        let mut pcm_buffer = Vec::with_capacity(FFT_SIZE * 2);
        println!("降噪线程已启动，目标采样率: {}Hz", MODEL_SAMPLE_RATE);

        while thread_running.load(Ordering::SeqCst) {
            if !thread_enabled.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(10));
                continue;
            }

            while pcm_buffer.len() < FFT_SIZE && thread_running.load(Ordering::SeqCst) {
                if let Some(sample) = consumer_in.try_pop() {
                    pcm_buffer.push(sample);
                } else {
                    thread::sleep(Duration::from_millis(1));
                }
            }

            if pcm_buffer.len() < FFT_SIZE {
                continue;
            }

            let (real, imag) = match stft_audio(&pcm_buffer, FFT_SIZE, HOP_SIZE) {
                Ok(value) => value,
                Err(err) => {
                    eprintln!("STFT 失败: {err}");
                    thread_stats.inference_errors.fetch_add(1, Ordering::SeqCst);
                    pcm_buffer.drain(..HOP_SIZE.min(pcm_buffer.len()));
                    continue;
                }
            };

            let infer_started = Instant::now();
            let outputs = session.run(vec![real.into_dyn(), imag.into_dyn()]);
            let inference_micros = infer_started.elapsed().as_micros() as u64;
            thread_stats
                .last_inference_micros
                .store(inference_micros, Ordering::SeqCst);

            match outputs {
                Ok(values) => {
                    thread_stats.inference_frames.fetch_add(1, Ordering::SeqCst);
                    thread_stats
                        .total_inference_micros
                        .fetch_add(inference_micros, Ordering::SeqCst);

                    if let Some(output) = values.first() {
                        for &sample in output.iter() {
                            if producer_out.try_push(sample).is_err() {
                                thread_stats.output_underflows.fetch_add(1, Ordering::SeqCst);
                            }
                        }
                    } else {
                        thread_stats.inference_errors.fetch_add(1, Ordering::SeqCst);
                    }
                }
                Err(err) => {
                    eprintln!("ONNX 推理失败: {err}");
                    thread_stats.inference_errors.fetch_add(1, Ordering::SeqCst);
                }
            }

            pcm_buffer.drain(..HOP_SIZE.min(pcm_buffer.len()));
        }

        println!("降噪线程已退出");
    });

    *guard = Some(AudioState {
        handle: Some(handle),
        running,
        enabled,
        input_stream: Some(input_stream),
        output_stream: Some(output_stream),
        input_device_name: device_name(&input_device),
        output_device_name: device_name(&output_device),
        input_channels: input_config.channels,
        output_channels: output_config.channels,
        input_sample_format: sample_format_label(input_sample_format),
        output_sample_format: sample_format_label(output_sample_format),
        sample_rate: MODEL_SAMPLE_RATE,
        stats,
    });

    Ok(format!(
        "降噪引擎已启动，输入: {}，输出: {}，采样率: {}Hz",
        device_name(&input_device),
        device_name(&output_device),
        MODEL_SAMPLE_RATE
    ))
}

#[tauri::command]
pub fn open_listener() -> Result<String, String> {
    let mut guard = audio_cell().lock().unwrap();
    let state = guard
        .as_mut()
        .ok_or_else(|| "降噪引擎未启动".to_string())?;

    if let Some(stream) = &state.input_stream {
        stream.play().map_err(|err| err.to_string())?;
    }
    if let Some(stream) = &state.output_stream {
        stream.play().map_err(|err| err.to_string())?;
    }

    state.enabled.store(true, Ordering::SeqCst);
    Ok("监听已打开".into())
}

#[tauri::command]
pub fn close_listener() -> Result<String, String> {
    let mut guard = audio_cell().lock().unwrap();
    let state = guard
        .as_mut()
        .ok_or_else(|| "降噪引擎未启动".to_string())?;

    state.enabled.store(false, Ordering::SeqCst);
    Ok("监听已关闭".into())
}

#[tauri::command]
pub fn stop_noise_reduction() -> Result<String, String> {
    let mut guard = audio_cell().lock().unwrap();
    if let Some(state) = guard.as_mut() {
        state.enabled.store(false, Ordering::SeqCst);
        state.running.store(false, Ordering::SeqCst);
        if let Some(handle) = state.handle.take() {
            let _ = handle.join();
        }
        *guard = None;
        return Ok("降噪已停止".into());
    }

    Ok("当前没有正在运行的降噪任务".into())
}

#[warn(dead_code)]
fn install_vbcable() -> String {
    let installed = check_vbcable_installed();
    if installed == "true" {
        return "VB-Cable 已安装".into();
    }
    "当前版本尚未集成自动安装程序，请先手动安装 VB-Cable".into()
}

#[warn(dead_code)]
fn check_vbcable_installed() -> String {
    let host = cpal::default_host();
    if let Ok(devices) = host.output_devices() {
        for device in devices {
            if device_name(&device).contains("CABLE Output") {
                return "true".into();
            }
        }
    }

    "false".into()
}

#[tauri::command]
pub fn on_install_vbcable() -> String {
    install_vbcable()
}
