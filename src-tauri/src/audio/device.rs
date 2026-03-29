use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{
    BufferSize, SampleFormat, Stream, StreamConfig, SupportedBufferSize, SupportedStreamConfig,
};
use nnnoiseless::DenoiseState;
use once_cell::sync::OnceCell;
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;
use serde::Serialize;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::str::FromStr;
use std::time::{Duration, Instant};

const MODEL_SAMPLE_RATE: u32 = 48_000;
const DENOISE_FRAME_SIZE: usize = DenoiseState::FRAME_SIZE;
/** 环形缓冲区容量 */
const RING_CAPACITY: usize = DENOISE_FRAME_SIZE * 12;
const PCM_SCALE: f32 = 32_768.0;
const INPUT_SILENCE_RMS_THRESHOLD: f32 = 0.004;
const OUTPUT_SILENCE_RMS_THRESHOLD: f32 = 0.003;
const SPEECH_PROB_THRESHOLD: f32 = 0.28;
const GATE_ATTACK: f32 = 0.35;
const GATE_RELEASE: f32 = 0.08;
const QUIET_FRAME_GAIN: f32 = 0.15;
const VBCABLE_DOWNLOAD_URL: &str =
    "https://download.vb-audio.com/Download_CABLE/VBCABLE_Driver_Pack45.zip";
const VBCABLE_INTEGRATION_NOTICE: &str =
    "VB-CABLE 官方许可不允许未授权集成到别的软件安装流程，本应用仅提供状态检测、官方入口和推荐配置。";

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
    id: String,
    name: String,
    detail: Option<String>,
    compatible: bool,
    supported_sample_formats: Vec<String>,
    virtual_mic_candidate: bool,
}

#[derive(Serialize)]
pub struct AudioDeviceCatalog {
    model_sample_rate: u32,
    default_input_id: Option<String>,
    default_output_id: Option<String>,
    preferred_virtual_output_id: Option<String>,
    preferred_monitor_output_id: Option<String>,
    inputs: Vec<AudioDeviceInfo>,
    outputs: Vec<AudioDeviceInfo>,
}

#[derive(Serialize)]
pub struct VBCableStatus {
    installed: bool,
    package_dir: Option<String>,
    setup_path: Option<String>,
    control_panel_path: Option<String>,
    input_device_id: Option<String>,
    input_device_name: Option<String>,
    output_device_id: Option<String>,
    output_device_name: Option<String>,
    official_download_url: String,
    integration_notice: String,
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

fn device_name(device: &cpal::Device) -> String {
    device
        .description()
        .map(|description| description.name().trim().to_string())
        .ok()
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| {
            #[allow(deprecated)]
            {
                device.name().unwrap_or_else(|_| "未知设备".to_string())
            }
        })
}

fn device_detail(device: &cpal::Device) -> Option<String> {
    let name = device_name(device);
    device.description().ok().and_then(|description| {
        description
            .driver()
            .or_else(|| description.manufacturer())
            .or_else(|| description.address())
            .or_else(|| {
                description.extended().iter().find_map(|line| {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed)
                    }
                })
            })
            .map(str::trim)
            .filter(|detail| !detail.is_empty() && *detail != name)
            .map(str::to_string)
    })
}

fn device_display_label(device: &cpal::Device) -> String {
    let name = device_name(device);
    if let Some(detail) = device_detail(device) {
        format!("{name} · {detail}")
    } else {
        name
    }
}

fn device_search_text(device: &cpal::Device) -> String {
    let mut parts = vec![device_name(device)];
    if let Some(detail) = device_detail(device) {
        parts.push(detail);
    }

    if let Ok(description) = device.description() {
        parts.extend(
            description
                .extended()
                .iter()
                .map(|line| line.trim())
                .filter(|line| !line.is_empty())
                .map(str::to_string),
        );
    }

    parts.join(" ").to_ascii_lowercase()
}

fn device_id_text(device: &cpal::Device) -> String {
    device
        .id()
        .map(|device_id| device_id.to_string())
        .unwrap_or_else(|_| device_name(device))
}

fn parse_device_id(selector: &str) -> Option<cpal::DeviceId> {
    cpal::DeviceId::from_str(selector).ok()
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

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn is_vbcable_package_dir(path: &Path) -> bool {
    path.is_dir()
        && (path.join("VBCABLE_Setup_x64.exe").is_file() || path.join("VBCABLE_Setup.exe").is_file())
}

fn find_vbcable_package_in(path: &Path) -> Option<PathBuf> {
    if is_vbcable_package_dir(path) {
        return Some(path.to_path_buf());
    }

    std::fs::read_dir(path).ok()?.find_map(|entry| {
        let entry = entry.ok()?;
        let child_path = entry.path();
        let child_name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        if child_name.starts_with("vbcable_driver_pack") && is_vbcable_package_dir(&child_path) {
            Some(child_path)
        } else {
            None
        }
    })
}

fn candidate_vbcable_search_dirs() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(path) = env::var("PVZ_VBCABLE_DIR") {
        candidates.push(PathBuf::from(path));
    }

    if let Ok(path) = env::current_dir() {
        candidates.push(path.clone());
        if let Some(parent) = path.parent() {
            candidates.push(parent.to_path_buf());
        }
    }

    if let Ok(path) = env::current_exe() {
        if let Some(parent) = path.parent() {
            candidates.push(parent.to_path_buf());
            if let Some(grand_parent) = parent.parent() {
                candidates.push(grand_parent.to_path_buf());
            }
        }
    }

    if let Ok(user_profile) = env::var("USERPROFILE") {
        let user_profile = PathBuf::from(user_profile);
        candidates.push(user_profile.join("Downloads"));
        candidates.push(user_profile.join("Desktop"));
        candidates.push(user_profile.join("Documents"));
    }

    let mut unique = Vec::new();
    for candidate in candidates {
        if unique.iter().any(|item: &PathBuf| item == &candidate) {
            continue;
        }
        unique.push(candidate);
    }

    unique
}

fn find_local_vbcable_package_dir() -> Option<PathBuf> {
    candidate_vbcable_search_dirs()
        .into_iter()
        .find_map(|path| find_vbcable_package_in(&path))
}

fn preferred_vbcable_setup_path(package_dir: &Path) -> Option<PathBuf> {
    let x64_setup = package_dir.join("VBCABLE_Setup_x64.exe");
    if x64_setup.is_file() {
        return Some(x64_setup);
    }

    let setup = package_dir.join("VBCABLE_Setup.exe");
    if setup.is_file() {
        return Some(setup);
    }

    None
}

fn vbcable_control_panel_path(package_dir: &Path) -> Option<PathBuf> {
    let control_panel = package_dir.join("VBCABLE_ControlPanel.exe");
    if control_panel.is_file() {
        Some(control_panel)
    } else {
        None
    }
}

fn find_vbcable_output_device(host: &cpal::Host) -> Option<cpal::Device> {
    host.output_devices()
        .ok()?
        .find(|device| looks_like_vbcable_output(&device_search_text(device)))
}

fn find_vbcable_input_device(host: &cpal::Host) -> Option<cpal::Device> {
    host.input_devices()
        .ok()?
        .find(|device| looks_like_vbcable_input(&device_search_text(device)))
}

fn build_vbcable_status() -> VBCableStatus {
    let host = cpal::default_host();
    let output_device = find_vbcable_output_device(&host);
    let input_device = find_vbcable_input_device(&host);
    let package_dir = find_local_vbcable_package_dir();
    let setup_path = package_dir
        .as_ref()
        .and_then(|path| preferred_vbcable_setup_path(path));
    let control_panel_path = package_dir
        .as_ref()
        .and_then(|path| vbcable_control_panel_path(path));

    VBCableStatus {
        installed: output_device.is_some() && input_device.is_some(),
        package_dir: package_dir.as_ref().map(|path| path_to_string(path)),
        setup_path: setup_path.as_ref().map(|path| path_to_string(path)),
        control_panel_path: control_panel_path.as_ref().map(|path| path_to_string(path)),
        input_device_id: input_device.as_ref().map(|device| device_id_text(device)),
        input_device_name: input_device.as_ref().map(|device| device_display_label(device)),
        output_device_id: output_device.as_ref().map(|device| device_id_text(device)),
        output_device_name: output_device.as_ref().map(|device| device_display_label(device)),
        official_download_url: VBCABLE_DOWNLOAD_URL.to_string(),
        integration_notice: VBCABLE_INTEGRATION_NOTICE.to_string(),
    }
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

fn find_input_device_by_selector(host: &cpal::Host, target: &str) -> Result<cpal::Device, String> {
    if let Some(device_id) = parse_device_id(target) {
        if let Some(device) = host.device_by_id(&device_id) {
            if device.supports_input() {
                return Ok(device);
            }
        }

        return Err(format!("未找到输入设备: {target}"));
    }

    let devices = host.input_devices().map_err(|err| err.to_string())?;
    devices
        .into_iter()
        .find(|device| device_name(device) == target)
        .ok_or_else(|| format!("未找到输入设备: {target}"))
}

fn find_output_device_by_selector(host: &cpal::Host, target: &str) -> Result<cpal::Device, String> {
    if let Some(device_id) = parse_device_id(target) {
        if let Some(device) = host.device_by_id(&device_id) {
            if device.supports_output() {
                return Ok(device);
            }
        }

        return Err(format!("未找到输出设备: {target}"));
    }

    let devices = host.output_devices().map_err(|err| err.to_string())?;
    devices
        .into_iter()
        .find(|device| device_name(device) == target)
        .ok_or_else(|| format!("未找到输出设备: {target}"))
}

fn preferred_virtual_output_id(host: &cpal::Host) -> Option<String> {
    host.output_devices().ok()?.find_map(|device| {
        if looks_like_virtual_microphone_output(&device_search_text(&device))
            && device_supports_rate(&device, false, MODEL_SAMPLE_RATE)
        {
            Some(device_id_text(&device))
        } else {
            None
        }
    })
}

fn preferred_monitor_output_id(host: &cpal::Host, excluded: Option<&str>) -> Option<String> {
    if let Some(default_device) = host.default_output_device() {
        let device_id = device_id_text(&default_device);
        if Some(device_id.as_str()) != excluded
            && device_supports_rate(&default_device, false, MODEL_SAMPLE_RATE)
        {
            return Some(device_id);
        }
    }

    if let Ok(devices) = host.output_devices() {
        for device in devices {
            let device_id = device_id_text(&device);
            if Some(device_id.as_str()) == excluded {
                continue;
            }
            if !device_supports_rate(&device, false, MODEL_SAMPLE_RATE) {
                continue;
            }
            if !looks_like_virtual_microphone_output(&device_search_text(&device)) {
                return Some(device_id);
            }
        }
    }

    if let Ok(devices) = host.output_devices() {
        for device in devices {
            let device_id = device_id_text(&device);
            if Some(device_id.as_str()) == excluded {
                continue;
            }
            if device_supports_rate(&device, false, MODEL_SAMPLE_RATE) {
                return Some(device_id);
            }
        }
    }

    None
}

fn resolve_input_device(host: &cpal::Host, requested: Option<&str>) -> Result<cpal::Device, String> {
    if let Some(name) = requested {
        return find_input_device_by_selector(host, name);
    }

    host.default_input_device()
        .ok_or_else(|| "未找到默认输入设备".to_string())
}

fn resolve_virtual_output_device(
    host: &cpal::Host,
    requested: Option<&str>,
) -> Result<cpal::Device, String> {
    if let Some(name) = requested {
        return find_output_device_by_selector(host, name);
    }

    if let Some(device_id) = preferred_virtual_output_id(host) {
        return find_output_device_by_selector(host, &device_id);
    }

    host.default_output_device()
        .ok_or_else(|| "未找到默认处理后输出设备".to_string())
}

fn resolve_monitor_output_device(
    host: &cpal::Host,
    requested: Option<&str>,
    excluded_id: &str,
) -> Result<Option<cpal::Device>, String> {
    if let Some(selector) = requested {
        let device = find_output_device_by_selector(host, selector)?;
        if device_id_text(&device) == excluded_id {
            return Err("监听设备不能和虚拟麦输出设备相同".to_string());
        }
        return Ok(Some(device));
    }

    if let Some(device_id) = preferred_monitor_output_id(host, Some(excluded_id)) {
        return find_output_device_by_selector(host, &device_id).map(Some);
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
        id: device_id_text(device),
        name: device_name(device),
        detail: device_detail(device),
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
        id: device_id_text(device),
        name: name.clone(),
        detail: device_detail(device),
        compatible: !supported.is_empty(),
        supported_sample_formats: supported
            .into_iter()
            .map(|cfg| sample_format_label(cfg.sample_format()))
            .collect(),
        virtual_mic_candidate: looks_like_virtual_microphone_output(&device_search_text(device)),
    }
}

#[tauri::command]
pub fn list_audio_devices() -> Result<AudioDeviceCatalog, String> {
    let host = cpal::default_host();
    let default_input_id = host.default_input_device().map(|device| device_id_text(&device));
    let default_output_id = host.default_output_device().map(|device| device_id_text(&device));
    let preferred_virtual_output_id = preferred_virtual_output_id(&host);
    let preferred_monitor_output_id = preferred_monitor_output_id(
        &host,
        preferred_virtual_output_id.as_deref(),
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
        default_input_id,
        default_output_id,
        preferred_virtual_output_id,
        preferred_monitor_output_id,
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
    input_device_id: Option<String>,
    virtual_output_device_id: Option<String>,
    monitor_output_device_id: Option<String>,
) -> Result<String, String> {
    let mut guard = audio_cell().lock().unwrap();
    if let Some(state) = &*guard {
        if state.running.load(Ordering::SeqCst) {
            return Ok("降噪引擎已经启动".into());
        }
    }

    let input_device_id = normalize_requested_device_name(input_device_id);
    let virtual_output_device_id = normalize_requested_device_name(virtual_output_device_id);
    let monitor_output_device_id = normalize_requested_device_name(monitor_output_device_id);

    let host = cpal::default_host();
    let input_device = resolve_input_device(&host, input_device_id.as_deref())?;
    let virtual_output_device =
        resolve_virtual_output_device(&host, virtual_output_device_id.as_deref())?;
    let resolved_virtual_output_id = device_id_text(&virtual_output_device);
    let resolved_virtual_output_label = device_display_label(&virtual_output_device);
    let monitor_output_device = resolve_monitor_output_device(
        &host,
        monitor_output_device_id.as_deref(),
        &resolved_virtual_output_id,
    )?;
    let resolved_monitor_output_label = monitor_output_device
        .as_ref()
        .map(device_display_label);

    let input_supported = supported_configs_for_rate(&input_device, true, MODEL_SAMPLE_RATE)?;
    let input_supported = select_supported_config(input_supported).ok_or_else(|| {
        format!(
            "输入设备 {} 不支持 {}Hz",
            device_display_label(&input_device),
            MODEL_SAMPLE_RATE
        )
    })?;

    let virtual_output_supported =
        supported_configs_for_rate(&virtual_output_device, false, MODEL_SAMPLE_RATE)?;
    let virtual_output_supported = select_supported_config(virtual_output_supported).ok_or_else(|| {
        format!(
            "输出设备 {} 不支持 {}Hz",
            resolved_virtual_output_label,
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
                device_display_label(device),
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
    // 输入环形缓冲区 
    let input_ring = HeapRb::<f32>::new(RING_CAPACITY);
    let (producer_in, mut consumer_in) = input_ring.split();
    // 虚拟输出环形缓冲区 
    let virtual_output_ring = HeapRb::<f32>::new(RING_CAPACITY);
    // 这是虚拟输出设备 
    let (mut virtual_output_producer, virtual_output_consumer) = virtual_output_ring.split();
    // 监听设备环形缓冲区 
    let monitor_output_ring = HeapRb::<f32>::new(RING_CAPACITY);
    // 这是监听设备
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
        input_device_name: device_display_label(&input_device),
        virtual_output_device_name: resolved_virtual_output_label.clone(),
        monitor_output_device_name: resolved_monitor_output_label.clone(),
        input_channels: input_config.channels,
        virtual_output_channels: virtual_output_config.channels,
        monitor_output_channels,
        input_sample_format: sample_format_label(input_sample_format),
        virtual_output_sample_format: sample_format_label(virtual_output_sample_format),
        monitor_output_sample_format,
        sample_rate: MODEL_SAMPLE_RATE,
        stats,
    });

    let monitor_summary = resolved_monitor_output_label.unwrap_or_else(|| "未配置".to_string());
    Ok(format!(
        "RNNoise 降噪引擎已启动，输入: {}，虚拟麦输出: {}，监听: {}，采样率: {}Hz",
        device_display_label(&input_device),
        resolved_virtual_output_label,
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

#[tauri::command]
pub fn on_install_vbcable() -> String {
    let status = build_vbcable_status();
    if status.installed {
        return "VB-CABLE 已安装，可直接套用推荐配置。".into();
    }

    if let Some(package_dir) = status.package_dir {
        return format!(
            "已发现本地 VB-CABLE 驱动包：{}。请以管理员身份运行官方 Setup 程序安装，并在安装后重启电脑。",
            package_dir
        );
    }

    format!(
        "未检测到 VB-CABLE。本应用不内置自动安装，请从官网下载后解压并以管理员身份运行官方 Setup：{}",
        status.official_download_url
    )
}

#[tauri::command]
pub fn get_vbcable_status() -> VBCableStatus {
    build_vbcable_status()
}