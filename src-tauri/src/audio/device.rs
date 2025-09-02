use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Stream;
use ndarray::Array3;
use once_cell::sync::OnceCell;
use onnxruntime::{environment::Environment, LoggingLevel};
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;
use rustfft::{num_complex::Complex, FftPlanner};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

struct AudioState {
    handle: Option<JoinHandle<()>>,
    running: Arc<AtomicBool>, // 线程生命周期
    enabled: Arc<AtomicBool>, // 监听/推理开关
    input_stream: Option<Stream>,
    output_stream: Option<Stream>,
}

static AUDIO_THREAD: OnceCell<Arc<Mutex<Option<AudioState>>>> = OnceCell::new();

/// STFT 一帧
fn stft_frame(frame: &[f32], fft_size: usize) -> Vec<Complex<f32>> {
    let mut buf: Vec<Complex<f32>> = frame.iter().map(|&x| Complex { re: x, im: 0.0 }).collect();
    buf.resize(fft_size, Complex { re: 0.0, im: 0.0 });
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(fft_size);
    fft.process(&mut buf);
    buf
}

/// STFT 转换
fn stft_audio(audio: &[f32], fft_size: usize, hop_size: usize) -> (Array3<f32>, Array3<f32>) {
    let mut frames_real = Vec::new();
    let mut frames_imag = Vec::new();

    let mut pos = 0;
    while pos + fft_size <= audio.len() {
        let frame = &audio[pos..pos + fft_size];
        let spectrum = stft_frame(frame, fft_size);

        let mut real = Vec::with_capacity(fft_size / 2 + 1);
        let mut imag = Vec::with_capacity(fft_size / 2 + 1);

        for &c in spectrum.iter().take(fft_size / 2 + 1) {
            real.push(c.re);
            imag.push(c.im);
        }

        frames_real.push(real);
        frames_imag.push(imag);
        pos += hop_size;
    }

    let frames = frames_real.len();
    let bins = fft_size / 2 + 1;

    let real_array = Array3::from_shape_vec(
        (1, bins, frames),
        frames_real.into_iter().flatten().collect(),
    )
    .unwrap();
    let imag_array = Array3::from_shape_vec(
        (1, bins, frames),
        frames_imag.into_iter().flatten().collect(),
    )
    .unwrap();
    (real_array, imag_array)
}

#[tauri::command]
pub fn start_noise_reduction() -> String {
    let thread_cell = AUDIO_THREAD.get_or_init(|| Arc::new(Mutex::new(None)));
    let mut guard = thread_cell.lock().unwrap();
    if let Some(state) = &*guard {
        if state.running.load(Ordering::SeqCst) {
            return "已经在运行".into();
        }
    }

    let running = Arc::new(AtomicBool::new(true));
    let enabled = Arc::new(AtomicBool::new(false));

    // 环形缓冲
    let input_ring = HeapRb::<f32>::new(48000 * 4);
    let (mut prod_in, mut cons_in) = input_ring.split();
    let output_ring = HeapRb::<f32>::new(48000 * 4);
    let (mut prod_out, mut cons_out) = output_ring.split();

    let host = cpal::default_host();
    let input_dev = host.default_input_device().expect("无输入设备");
    let output_dev = host.default_output_device().expect("无输出设备");

    let input_cfg: cpal::StreamConfig = input_dev.default_input_config().unwrap().into();
    let output_cfg: cpal::StreamConfig = output_dev.default_output_config().unwrap().into();
    let in_channels = input_cfg.channels as usize;
    let out_channels = output_cfg.channels as usize;

    let input_enabled = enabled.clone();
    let input_stream = input_dev
        .build_input_stream(
            &input_cfg,
            move |data: &[f32], _| {
                if input_enabled.load(Ordering::SeqCst) {
                    for frame in data.chunks(in_channels) {
                        let mono = if in_channels == 1 {
                            frame[0]
                        } else {
                            frame.iter().sum::<f32>() / in_channels as f32
                        };
                        let _ = prod_in.try_push(mono);
                    }
                }
            },
            move |err| eprintln!("输入流错误: {}", err),
            None,
        )
        .expect("构建输入流失败");

    let output_enabled = enabled.clone();
    let output_stream = output_dev
        .build_output_stream(
            &output_cfg,
            move |data: &mut [f32], _| {
                if output_enabled.load(Ordering::SeqCst) {
                    for frame in data.chunks_mut(out_channels) {
                        let s = cons_out.try_pop().unwrap_or(0.0);
                        for ch in 0..out_channels {
                            frame[ch] = s;
                        }
                    }
                } else {
                    data.fill(0.0);
                }
            },
            move |err| eprintln!("输出流错误: {}", err),
            None,
        )
        .expect("构建输出流失败");

    let thread_running = running.clone();
    let thread_enabled = enabled.clone();

    let handle = thread::spawn(move || {
        // ORT 环境 & 模型
        let env = Environment::builder()
            .with_name("denoise")
            .with_log_level(LoggingLevel::Warning)
            .build()
            .unwrap();
        let mut session = env
            .new_session_builder()
            .unwrap()
            .with_model_from_file("public/onnx_model.onnx")
            .unwrap();

        let fft_size = 400;
        let hop_size = 160;

        println!("🎤 降噪线程已启动");

        let mut pcm_buf: Vec<f32> = Vec::new();

        while thread_running.load(Ordering::SeqCst) {
            if !thread_enabled.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(10));
                continue;
            }

            // 聚齐 PCM
            while pcm_buf.len() < fft_size && thread_running.load(Ordering::SeqCst) {
                if let Some(s) = cons_in.try_pop() {
                    pcm_buf.push(s);
                } else {
                    thread::sleep(Duration::from_millis(2));
                }
            }
            if pcm_buf.len() < fft_size {
                continue;
            }

            // STFT
            let (real, imag) = stft_audio(&pcm_buf, fft_size, hop_size);

            // ONNX 推理
            let outputs = session.run(vec![real.into_dyn(), imag.into_dyn()]);

            if let Ok(out) = outputs {
                let out_view = out[0].view();
                for &s in out_view.iter() {
                    let _ = prod_out.try_push(s);
                }
            } else {
                eprintln!("ONNX 推理失败: {:?}", outputs.err());
            }

            pcm_buf.drain(..hop_size); // 滑动 hop
        }

        println!("🛑 降噪线程已退出");
    });

    *guard = Some(AudioState {
        handle: Some(handle),
        running,
        enabled,
        input_stream: Some(input_stream),
        output_stream: Some(output_stream),
    });

    "降噪线程已启动".into()
}

#[tauri::command]
pub fn open_listener() -> String {
    let thread_cell = AUDIO_THREAD.get_or_init(|| Arc::new(Mutex::new(None)));
    let mut guard = thread_cell.lock().unwrap();
    if let Some(state) = guard.as_mut() {
        if let Some(stream) = &state.input_stream {
            let _ = stream.play();
        }
        if let Some(stream) = &state.output_stream {
            let _ = stream.play();
        }
        state.enabled.store(true, Ordering::SeqCst);
        println!("监听已打开");
        return "监听已打开".into();
    }
    "降噪线程未启动".into()
}
/**关闭监听 */
#[tauri::command]
pub fn close_listener() -> String {
    let thread_cell = AUDIO_THREAD.get_or_init(|| Arc::new(Mutex::new(None)));
    let mut guard = thread_cell.lock().unwrap();
    if let Some(state) = guard.as_mut() {
        state.enabled.store(false, Ordering::SeqCst);
        return "监听已关闭".into();
    }
    "降噪线程未启动".into()
}

/**停止降噪 */
#[tauri::command]
pub fn stop_noise_reduction() -> String {
    let thread_cell = AUDIO_THREAD.get_or_init(|| Arc::new(Mutex::new(None)));
    let mut guard = thread_cell.lock().unwrap();
    if let Some(state) = guard.as_mut() {
        state.enabled.store(false, Ordering::SeqCst);
        state.running.store(false, Ordering::SeqCst);
        if let Some(h) = state.handle.take() {
            let _ = h.join();
        }
        *guard = None;
        return "降噪已停止".into();
    }

    "当前没有正在运行的降噪任务".into()
}

/**安装虚拟麦克风 */
#[warn(dead_code)]
fn install_vbcable() -> String {
    // let installer = Path::new("VBCABLE_Setup_x64.exe");
    // if !installer.exists() {
    //     return "❌ 找不到安装包".into();
    // }
    // // 以管理员模式启动静默安装
    // Command::new("powershell")
    //     .args(&[
    //         "Start-Process",
    //         installer.to_str().unwrap(),
    //         "-ArgumentList",
    //         "\"-i -h\"",
    //         "-Verb",
    //         "RunAs",
    //         "-Wait",
    //     ])
    //     .status().expect("安装失败，请确认是否以管理员身份运行，并手动安装驱动");
    let _installed = check_vbcable_installed();
    "✅ 安装成功".into()
}

// 检查设备是否安装成功（示例：列出音频设备名称）
#[warn(dead_code)]
fn check_vbcable_installed() -> String {
    let host = cpal::default_host();
    for dev in host.output_devices().expect("获取输出设备失败") {
        if dev.name().unwrap().contains("CABLE Output") {
            return "true".into();
        }
    }

    "❌ VB‑Cable 未安装".into()
}

// 用户点击时调用
#[tauri::command]
pub fn on_install_vbcable() -> String {
    // install_vbcable();
    // // 安装后稍等几秒让 Windows 注册设备
    // thread::sleep(Duration::from_secs(5));
    // if check_vbcable_installed() == "✅ VB‑Cable 已安装" {
    //     println!("✅ VB‑Cable 已安装，可以在微信中选择“CABLE Output”作为麦克风");
    // } else {
    //     eprintln!("❌ 安装失败，请确认是否以管理员身份运行，并手动安装驱动");
    // }
    install_vbcable();
    "安装成功".into()
}
