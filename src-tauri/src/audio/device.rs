use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{
    BufferSize, SampleFormat, Stream, StreamConfig, SupportedBufferSize, SupportedStreamConfig,
};
use nnnoiseless::DenoiseState;
use once_cell::sync::OnceCell;
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;
use serde::Serialize;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const MODEL_SAMPLE_RATE: u32 = 48_000;
const DENOISE_FRAME_SIZE: usize = DenoiseState::FRAME_SIZE;
const RING_CAPACITY: usize = DENOISE_FRAME_SIZE * 12;
const PCM_SCALE: f32 = 32_768.0;
const INPUT_SILENCE_RMS_THRESHOLD: f32 = 0.004;
const OUTPUT_SILENCE_RMS_THRESHOLD: f32 = 0.003;
const SPEECH_PROB_THRESHOLD: f32 = 0.28;
const GATE_ATTACK: f32 = 0.35;
const GATE_RELEASE: f32 = 0.08;
const QUIET_FRAME_GAIN: f32 = 0.15;

struct AudioState {
    handle: Option<JoinHandle<()>>,
    running: Arc<AtomicBool>,
    monitor_enabled: Arc<AtomicBool>,
    _input_stream: Option<Stream>,
    _virtual_output_stream: Option<Stream>,
    monitor_output_stream: Option<Stream>,
    input_device_name: String,
    virtual_output_device_name: String,
    monitor_output_device_name: Option<String>,
    input_channels: u16,
    virtual_output_channels: u16,
    monitor_output_channels: Option<u16>,
    input_sample_format: String,
    virtual_output_sample_format: String,
    monitor_output_sample_format: Option<String>,
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
    virtual_output_device_name: Option<String>,
    monitor_output_device_name: Option<String>,
    input_channels: Option<u16>,
    virtual_output_channels: Option<u16>,
    monitor_output_channels: Option<u16>,
    input_sample_format: Option<String>,
    virtual_output_sample_format: Option<String>,
    monitor_output_sample_format: Option<String>,
    stats: AudioStatsSnapshot,
}

#[derive(Serialize)]
pub struct AudioDeviceInfo {
    name: String,
    compatible: bool,
    supported_sample_formats: Vec<String>,
    virtual_mic_candidate: bool,
}

#[derive(Serialize)]
pub struct AudioDeviceCatalog {
    model_sample_rate: u32,
    default_input: Option<String>,
    default_output: Option<String>,
    preferred_virtual_output: Option<String>,
    preferred_monitor_output: Option<String>,
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

fn frame_rms(frame: &[f32]) -> f32 {
    if frame.is_empty() {
        return 0.0;
    }

    let energy = frame.iter().map(|sample| sample * sample).sum::<f32>() / frame.len() as f32;
    energy.sqrt()
}

fn normalize_requested_device_name(name: Option<String>) -> Option<String> {
    name.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn looks_like_virtual_microphone_output(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.contains("cable input")
        || lower.contains("voice meeter input")
        || lower.contains("voicemeeter input")
        || lower.contains("virtual cable")
        || lower.contains("virtual audio cable")
        || lower.contains("vb-audio")
}

fn looks_like_vbcable_output(name: &str) -> bool {
    name.to_ascii_lowercase().contains("cable input")
}

fn looks_like_vbcable_input(name: &str) -> bool {
    name.to_ascii_lowercase().contains("cable output")
}

fn choose_low_latency_buffer_size(config: &SupportedStreamConfig) -> BufferSize {
    match config.buffer_size() {
        SupportedBufferSize::Range { min, max } => {
            let preferred = DENOISE_FRAME_SIZE as u32;
            BufferSize::Fixed(preferred.clamp(*min, *max))
        }
        SupportedBufferSize::Unknown => BufferSize::Default,
    }
}

fn select_supported_config(configs: Vec<SupportedStreamConfig>) -> Option<SupportedStreamConfig> {
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
            .filter(|cfg| {
                cfg.min_sample_rate() <= sample_rate && cfg.max_sample_rate() >= sample_rate
            })
            .map(|cfg| cfg.with_sample_rate(sample_rate))
            .collect()
    } else {
        device
            .supported_output_configs()
            .map_err(|err| err.to_string())?
            .filter(|cfg| {
                cfg.min_sample_rate() <= sample_rate && cfg.max_sample_rate() >= sample_rate
            })
            .map(|cfg| cfg.with_sample_rate(sample_rate))
            .collect()
    };

    Ok(result)
}

fn device_supports_rate(device: &cpal::Device, is_input: bool, sample_rate: u32) -> bool {
    supported_configs_for_rate(device, is_input, sample_rate)
        .map(|configs| !configs.is_empty())
        .unwrap_or(false)
}

fn find_named_input_device(host: &cpal::Host, target: &str) -> Result<cpal::Device, String> {
    let devices = host.input_devices().map_err(|err| err.to_string())?;
    devices
        .into_iter()
        .find(|device| device_name(device) == target)
        .ok_or_else(|| format!("未找到输入设备: {target}"))
}

fn find_named_output_device(host: &cpal::Host, target: &str) -> Result<cpal::Device, String> {
    let devices = host.output_devices().map_err(|err| err.to_string())?;
    devices
        .into_iter()
        .find(|device| device_name(device) == target)
        .ok_or_else(|| format!("未找到输出设备: {target}"))
}

fn preferred_virtual_output_name(host: &cpal::Host) -> Option<String> {
    host.output_devices().ok()?.find_map(|device| {
        let name = device_name(&device);
        if looks_like_virtual_microphone_output(&name)
            && device_supports_rate(&device, false, MODEL_SAMPLE_RATE)
        {
            Some(name)
        } else {
            None
        }
    })
}

fn preferred_monitor_output_name(host: &cpal::Host, excluded: Option<&str>) -> Option<String> {
    if let Some(default_device) = host.default_output_device() {
        let name = device_name(&default_device);
        if Some(name.as_str()) != excluded
            && device_supports_rate(&default_device, false, MODEL_SAMPLE_RATE)
        {
            return Some(name);
        }
    }

    if let Ok(devices) = host.output_devices() {
        for device in devices {
            let name = device_name(&device);
            if Some(name.as_str()) == excluded {
                continue;
            }
            if !device_supports_rate(&device, false, MODEL_SAMPLE_RATE) {
                continue;
            }
            if !looks_like_virtual_microphone_output(&name) {
                return Some(name);
            }
        }
    }

    if let Ok(devices) = host.output_devices() {
        for device in devices {
            let name = device_name(&device);
            if Some(name.as_str()) == excluded {
                continue;
            }
            if device_supports_rate(&device, false, MODEL_SAMPLE_RATE) {
                return Some(name);
            }
        }
    }

    None
}

fn resolve_input_device(host: &cpal::Host, requested: Option<&str>) -> Result<cpal::Device, String> {
    if let Some(name) = requested {
        return find_named_input_device(host, name);
    }

    host.default_input_device()
        .ok_or_else(|| "未找到默认输入设备".to_string())
}

fn resolve_virtual_output_device(
    host: &cpal::Host,
    requested: Option<&str>,
) -> Result<cpal::Device, String> {
    if let Some(name) = requested {
        return find_named_output_device(host, name);
    }

    if let Some(name) = preferred_virtual_output_name(host) {
        return find_named_output_device(host, &name);
    }

    host.default_output_device()
        .ok_or_else(|| "未找到默认处理后输出设备".to_string())
}

fn resolve_monitor_output_device(
    host: &cpal::Host,
    requested: Option<&str>,
    excluded_name: &str,
) -> Result<Option<cpal::Device>, String> {
    if let Some(name) = requested {
        if name == excluded_name {
            return Err("监听设备不能和虚拟麦输出设备相同".to_string());
        }
        return find_named_output_device(host, name).map(Some);
    }

    if let Some(name) = preferred_monitor_output_name(host, Some(excluded_name)) {
        return find_named_output_device(host, &name).map(Some);
    }

    Ok(None)
}

fn build_input_stream<T, P, F>(
    device: &cpal::Device,
    config: &StreamConfig,
    in_channels: usize,
    running: Arc<AtomicBool>,
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
                if !running.load(Ordering::SeqCst) {
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
    active: Option<Arc<AtomicBool>>,
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
                if let Some(flag) = active.as_ref() {
                    if !flag.load(Ordering::SeqCst) {
                        while consumer.try_pop().is_some() {}
                        for sample in data.iter_mut() {
                            *sample = convert(0.0);
                        }
                        return;
                    }
                }

                for frame in data.chunks_mut(out_channels) {
                    let sample = consumer.try_pop().unwrap_or_else(|| {
                        stats.output_underflows.fetch_add(1, Ordering::SeqCst);
                        0.0
                    });

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

fn input_device_info(device: &cpal::Device) -> AudioDeviceInfo {
    let supported = supported_configs_for_rate(device, true, MODEL_SAMPLE_RATE).unwrap_or_default();
    AudioDeviceInfo {
        name: device_name(device),
        compatible: !supported.is_empty(),
        supported_sample_formats: supported
            .into_iter()
            .map(|cfg| sample_format_label(cfg.sample_format()))
            .collect(),
        virtual_mic_candidate: false,
    }
}

fn output_device_info(device: &cpal::Device) -> AudioDeviceInfo {
    let name = device_name(device);
    let supported = supported_configs_for_rate(device, false, MODEL_SAMPLE_RATE).unwrap_or_default();
    AudioDeviceInfo {
        name: name.clone(),
        compatible: !supported.is_empty(),
        supported_sample_formats: supported
            .into_iter()
            .map(|cfg| sample_format_label(cfg.sample_format()))
            .collect(),
        virtual_mic_candidate: looks_like_virtual_microphone_output(&name),
    }
}

#[tauri::command]
pub fn list_audio_devices() -> Result<AudioDeviceCatalog, String> {
    let host = cpal::default_host();
    let default_input = host.default_input_device().map(|device| device_name(&device));
    let default_output = host.default_output_device().map(|device| device_name(&device));
    let preferred_virtual_output = preferred_virtual_output_name(&host);
    let preferred_monitor_output = preferred_monitor_output_name(
        &host,
        preferred_virtual_output.as_deref(),
    );

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
        preferred_virtual_output,
        preferred_monitor_output,
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
            listening: state.monitor_enabled.load(Ordering::SeqCst),
            sample_rate: state.sample_rate,
            model_sample_rate: MODEL_SAMPLE_RATE,
            input_device_name: Some(state.input_device_name.clone()),
            virtual_output_device_name: Some(state.virtual_output_device_name.clone()),
            monitor_output_device_name: state.monitor_output_device_name.clone(),
            input_channels: Some(state.input_channels),
            virtual_output_channels: Some(state.virtual_output_channels),
            monitor_output_channels: state.monitor_output_channels,
            input_sample_format: Some(state.input_sample_format.clone()),
            virtual_output_sample_format: Some(state.virtual_output_sample_format.clone()),
            monitor_output_sample_format: state.monitor_output_sample_format.clone(),
            stats: stats_snapshot(&state.stats),
        };
    }

    AudioStatus {
        running: false,
        listening: false,
        sample_rate: 0,
        model_sample_rate: MODEL_SAMPLE_RATE,
        input_device_name: None,
        virtual_output_device_name: None,
        monitor_output_device_name: None,
        input_channels: None,
        virtual_output_channels: None,
        monitor_output_channels: None,
        input_sample_format: None,
        virtual_output_sample_format: None,
        monitor_output_sample_format: None,
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
pub fn start_noise_reduction(
    input_device_name: Option<String>,
    virtual_output_device_name: Option<String>,
    monitor_output_device_name: Option<String>,
) -> Result<String, String> {
    let mut guard = audio_cell().lock().unwrap();
    if let Some(state) = &*guard {
        if state.running.load(Ordering::SeqCst) {
            return Ok("降噪引擎已经启动".into());
        }
    }

    let input_device_name = normalize_requested_device_name(input_device_name);
    let virtual_output_device_name = normalize_requested_device_name(virtual_output_device_name);
    let monitor_output_device_name = normalize_requested_device_name(monitor_output_device_name);

    let host = cpal::default_host();
    let input_device = resolve_input_device(&host, input_device_name.as_deref())?;
    let virtual_output_device =
        resolve_virtual_output_device(&host, virtual_output_device_name.as_deref())?;
    let resolved_virtual_output_name = device_name(&virtual_output_device);
    let monitor_output_device = resolve_monitor_output_device(
        &host,
        monitor_output_device_name.as_deref(),
        &resolved_virtual_output_name,
    )?;
    let resolved_monitor_output_name = monitor_output_device
        .as_ref()
        .map(|device| device_name(device));

    let input_supported = supported_configs_for_rate(&input_device, true, MODEL_SAMPLE_RATE)?;
    let input_supported = select_supported_config(input_supported).ok_or_else(|| {
        format!(
            "输入设备 {} 不支持 {}Hz",
            device_name(&input_device),
            MODEL_SAMPLE_RATE
        )
    })?;

    let virtual_output_supported =
        supported_configs_for_rate(&virtual_output_device, false, MODEL_SAMPLE_RATE)?;
    let virtual_output_supported = select_supported_config(virtual_output_supported).ok_or_else(|| {
        format!(
            "输出设备 {} 不支持 {}Hz",
            device_name(&virtual_output_device),
            MODEL_SAMPLE_RATE
        )
    })?;

    let input_sample_format = input_supported.sample_format();
    let mut input_config = input_supported.config();
    input_config.buffer_size = choose_low_latency_buffer_size(&input_supported);
    let input_channels = input_config.channels as usize;

    let virtual_output_sample_format = virtual_output_supported.sample_format();
    let mut virtual_output_config = virtual_output_supported.config();
    virtual_output_config.buffer_size = choose_low_latency_buffer_size(&virtual_output_supported);
    let virtual_output_channels = virtual_output_config.channels as usize;

    let monitor_output_setup = if let Some(device) = monitor_output_device.as_ref() {
        let supported = supported_configs_for_rate(device, false, MODEL_SAMPLE_RATE)?;
        let supported = select_supported_config(supported).ok_or_else(|| {
            format!(
                "监听设备 {} 不支持 {}Hz",
                device_name(device),
                MODEL_SAMPLE_RATE
            )
        })?;
        let sample_format = supported.sample_format();
        let mut config = supported.config();
        config.buffer_size = choose_low_latency_buffer_size(&supported);
        let channels = config.channels as usize;
        Some((sample_format, config, channels))
    } else {
        None
    };
    let monitor_output_channels = monitor_output_setup
        .as_ref()
        .map(|(_, config, _)| config.channels);
    let monitor_output_sample_format = monitor_output_setup
        .as_ref()
        .map(|(sample_format, _, _)| sample_format_label(*sample_format));

    let running = Arc::new(AtomicBool::new(true));
    let monitor_enabled = Arc::new(AtomicBool::new(false));
    let stats = Arc::new(AudioStatsCounters::default());

    let input_ring = HeapRb::<f32>::new(RING_CAPACITY);
    let (producer_in, mut consumer_in) = input_ring.split();
    let virtual_output_ring = HeapRb::<f32>::new(RING_CAPACITY);
    let (mut virtual_output_producer, virtual_output_consumer) = virtual_output_ring.split();
    let monitor_output_ring = HeapRb::<f32>::new(RING_CAPACITY);
    let (mut monitor_output_producer, monitor_output_consumer) = monitor_output_ring.split();

    let input_stream = match input_sample_format {
        SampleFormat::F32 => build_input_stream::<f32, _, _>(
            &input_device,
            &input_config,
            input_channels,
            running.clone(),
            producer_in,
            stats.clone(),
            |sample| sample,
        ),
        SampleFormat::I16 => build_input_stream::<i16, _, _>(
            &input_device,
            &input_config,
            input_channels,
            running.clone(),
            producer_in,
            stats.clone(),
            |sample| sample as f32 / i16::MAX as f32,
        ),
        SampleFormat::U16 => build_input_stream::<u16, _, _>(
            &input_device,
            &input_config,
            input_channels,
            running.clone(),
            producer_in,
            stats.clone(),
            |sample| (sample as f32 / u16::MAX as f32) * 2.0 - 1.0,
        ),
        other => Err(format!("暂不支持输入采样格式: {other:?}")),
    }?;

    let virtual_output_stream = match virtual_output_sample_format {
        SampleFormat::F32 => build_output_stream::<f32, _, _>(
            &virtual_output_device,
            &virtual_output_config,
            virtual_output_channels,
            Some(running.clone()),
            virtual_output_consumer,
            stats.clone(),
            |sample| sample,
        ),
        SampleFormat::I16 => build_output_stream::<i16, _, _>(
            &virtual_output_device,
            &virtual_output_config,
            virtual_output_channels,
            Some(running.clone()),
            virtual_output_consumer,
            stats.clone(),
            |sample| (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16,
        ),
        SampleFormat::U16 => build_output_stream::<u16, _, _>(
            &virtual_output_device,
            &virtual_output_config,
            virtual_output_channels,
            Some(running.clone()),
            virtual_output_consumer,
            stats.clone(),
            |sample| (((sample.clamp(-1.0, 1.0) + 1.0) * 0.5) * u16::MAX as f32) as u16,
        ),
        other => Err(format!("暂不支持虚拟麦输出采样格式: {other:?}")),
    }?;

    let monitor_output_stream = match (monitor_output_device.as_ref(), monitor_output_setup) {
        (Some(device), Some((sample_format, config, channels))) => {
            let stream = match sample_format {
                SampleFormat::F32 => build_output_stream::<f32, _, _>(
                    device,
                    &config,
                    channels,
                    Some(monitor_enabled.clone()),
                    monitor_output_consumer,
                    stats.clone(),
                    |sample| sample,
                ),
                SampleFormat::I16 => build_output_stream::<i16, _, _>(
                    device,
                    &config,
                    channels,
                    Some(monitor_enabled.clone()),
                    monitor_output_consumer,
                    stats.clone(),
                    |sample| (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16,
                ),
                SampleFormat::U16 => build_output_stream::<u16, _, _>(
                    device,
                    &config,
                    channels,
                    Some(monitor_enabled.clone()),
                    monitor_output_consumer,
                    stats.clone(),
                    |sample| (((sample.clamp(-1.0, 1.0) + 1.0) * 0.5) * u16::MAX as f32) as u16,
                ),
                other => Err(format!("暂不支持监听输出采样格式: {other:?}")),
            }?;
            Some(stream)
        }
        _ => None,
    };

    input_stream.play().map_err(|err| err.to_string())?;
    virtual_output_stream.play().map_err(|err| err.to_string())?;
    if let Some(stream) = monitor_output_stream.as_ref() {
        stream.play().map_err(|err| err.to_string())?;
    }

    let thread_running = running.clone();
    let thread_monitor_enabled = monitor_enabled.clone();
    let thread_stats = stats.clone();

    let handle = thread::spawn(move || {
        let mut denoise = DenoiseState::new();
        let mut frame_buffer = Vec::with_capacity(DENOISE_FRAME_SIZE * 2);
        let mut input_frame = [0.0f32; DENOISE_FRAME_SIZE];
        let mut output_frame = [0.0f32; DENOISE_FRAME_SIZE];
        let mut discard_first_frame = true;
        let mut gate_gain = 0.0f32;

        println!(
            "RNNoise 降噪线程已启动，目标采样率: {}Hz，帧长: {}",
            MODEL_SAMPLE_RATE, DENOISE_FRAME_SIZE
        );

        while thread_running.load(Ordering::SeqCst) {
            while frame_buffer.len() < DENOISE_FRAME_SIZE && thread_running.load(Ordering::SeqCst) {
                if let Some(sample) = consumer_in.try_pop() {
                    frame_buffer.push(sample);
                } else {
                    thread::sleep(Duration::from_millis(1));
                }
            }

            if frame_buffer.len() < DENOISE_FRAME_SIZE {
                continue;
            }

            let input_rms = frame_rms(&frame_buffer[..DENOISE_FRAME_SIZE]);
            for (index, sample) in frame_buffer[..DENOISE_FRAME_SIZE].iter().enumerate() {
                input_frame[index] = sample.clamp(-1.0, 1.0) * PCM_SCALE;
            }

            let infer_started = Instant::now();
            let speech_prob = denoise.process_frame(&mut output_frame, &input_frame);
            let inference_micros = infer_started.elapsed().as_micros() as u64;
            thread_stats
                .last_inference_micros
                .store(inference_micros, Ordering::SeqCst);
            thread_stats.inference_frames.fetch_add(1, Ordering::SeqCst);
            thread_stats
                .total_inference_micros
                .fetch_add(inference_micros, Ordering::SeqCst);

            if discard_first_frame {
                discard_first_frame = false;
                frame_buffer.drain(..DENOISE_FRAME_SIZE);
                continue;
            }

            let mut normalized_output = [0.0f32; DENOISE_FRAME_SIZE];
            for (index, sample) in output_frame.iter().enumerate() {
                normalized_output[index] = (sample / PCM_SCALE).clamp(-1.0, 1.0);
            }

            let output_rms = frame_rms(&normalized_output);
            let gate_target = if input_rms < INPUT_SILENCE_RMS_THRESHOLD
                && output_rms < OUTPUT_SILENCE_RMS_THRESHOLD
                && speech_prob < SPEECH_PROB_THRESHOLD
            {
                0.0
            } else if speech_prob < SPEECH_PROB_THRESHOLD {
                QUIET_FRAME_GAIN
            } else {
                1.0
            };
            let smoothing = if gate_target > gate_gain {
                GATE_ATTACK
            } else {
                GATE_RELEASE
            };
            gate_gain += (gate_target - gate_gain) * smoothing;

            let monitor_is_enabled = thread_monitor_enabled.load(Ordering::SeqCst);
            for sample in normalized_output.iter_mut() {
                *sample *= gate_gain;
                let routed_sample = sample.clamp(-1.0, 1.0);

                if virtual_output_producer.try_push(routed_sample).is_err() {
                    thread_stats.output_underflows.fetch_add(1, Ordering::SeqCst);
                }

                if monitor_is_enabled && monitor_output_producer.try_push(routed_sample).is_err() {
                    thread_stats.output_underflows.fetch_add(1, Ordering::SeqCst);
                }
            }

            frame_buffer.drain(..DENOISE_FRAME_SIZE);
        }

        println!("RNNoise 降噪线程已退出");
    });

    *guard = Some(AudioState {
        handle: Some(handle),
        running,
        monitor_enabled,
        _input_stream: Some(input_stream),
        _virtual_output_stream: Some(virtual_output_stream),
        monitor_output_stream,
        input_device_name: device_name(&input_device),
        virtual_output_device_name: resolved_virtual_output_name.clone(),
        monitor_output_device_name: resolved_monitor_output_name.clone(),
        input_channels: input_config.channels,
        virtual_output_channels: virtual_output_config.channels,
        monitor_output_channels,
        input_sample_format: sample_format_label(input_sample_format),
        virtual_output_sample_format: sample_format_label(virtual_output_sample_format),
        monitor_output_sample_format,
        sample_rate: MODEL_SAMPLE_RATE,
        stats,
    });

    let monitor_summary = resolved_monitor_output_name.unwrap_or_else(|| "未配置".to_string());
    Ok(format!(
        "RNNoise 降噪引擎已启动，输入: {}，虚拟麦输出: {}，监听: {}，采样率: {}Hz",
        device_name(&input_device),
        resolved_virtual_output_name,
        monitor_summary,
        MODEL_SAMPLE_RATE
    ))
}

#[tauri::command]
pub fn open_listener() -> Result<String, String> {
    let mut guard = audio_cell().lock().unwrap();
    let state = guard
        .as_mut()
        .ok_or_else(|| "降噪引擎未启动".to_string())?;

    if state.monitor_enabled.load(Ordering::SeqCst) {
        return Ok("监听已经打开".into());
    }

    let stream = state
        .monitor_output_stream
        .as_ref()
        .ok_or_else(|| "未配置监听输出设备，请先选择本地监听设备".to_string())?;
    stream.play().map_err(|err| err.to_string())?;
    state.monitor_enabled.store(true, Ordering::SeqCst);

    Ok(format!(
        "监听已打开，设备: {}",
        state
            .monitor_output_device_name
            .clone()
            .unwrap_or_else(|| "未配置".to_string())
    ))
}

#[tauri::command]
pub fn close_listener() -> Result<String, String> {
    let mut guard = audio_cell().lock().unwrap();
    let state = guard
        .as_mut()
        .ok_or_else(|| "降噪引擎未启动".to_string())?;

    if !state.monitor_enabled.load(Ordering::SeqCst) {
        return Ok("监听已经关闭".into());
    }

    state.monitor_enabled.store(false, Ordering::SeqCst);
    Ok("监听已关闭".into())
}

#[tauri::command]
pub fn stop_noise_reduction() -> Result<String, String> {
    let mut guard = audio_cell().lock().unwrap();
    if let Some(state) = guard.as_mut() {
        state.monitor_enabled.store(false, Ordering::SeqCst);
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
    let has_playback = host
        .output_devices()
        .ok()
        .map(|mut devices| devices.any(|device| looks_like_vbcable_output(&device_name(&device))))
        .unwrap_or(false);
    let has_recording = host
        .input_devices()
        .ok()
        .map(|mut devices| devices.any(|device| looks_like_vbcable_input(&device_name(&device))))
        .unwrap_or(false);

    if has_playback && has_recording {
        "true".into()
    } else {
        "false".into()
    }
}

#[tauri::command]
pub fn on_install_vbcable() -> String {
    install_vbcable()
}