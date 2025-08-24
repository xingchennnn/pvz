use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Stream,
};
use once_cell::sync::OnceCell;
use onnxruntime::ndarray::Array;
use onnxruntime::{environment::Environment, LoggingLevel};
use ringbuf::{
    traits::{Consumer, Producer, Split},
    HeapRb,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

struct AudioState {
    handle: Option<JoinHandle<()>>,
    running: Arc<AtomicBool>, // 推理线程是否存活
    enabled: Arc<AtomicBool>, // 监听/推理开关
    input_stream: Option<Stream>,
    output_stream: Option<Stream>,
}

static AUDIO_THREAD: OnceCell<Arc<Mutex<Option<AudioState>>>> = OnceCell::new();

#[tauri::command]
pub fn start_noise_reduction() -> String {
    let thread_cell = AUDIO_THREAD.get_or_init(|| Arc::new(Mutex::new(None)));
    let mut guard = thread_cell.lock().unwrap();

    if let Some(state) = &*guard {
        if state.running.load(Ordering::SeqCst) {
            return "已经在运行".into();
        }
    }

    // 线程生命周期标志
    let running = Arc::new(AtomicBool::new(true));
    let enabled = Arc::new(AtomicBool::new(false)); // 初始不推理，等待 open_listener 打开

    // 输入缓冲（麦克风 -> 模型）
    let input_ring = HeapRb::<f32>::new(48000 * 4);
    let (mut prod_in, mut cons_in) = input_ring.split();

    // 输出缓冲（模型 -> 扬声器/虚拟声卡）
    let output_ring = HeapRb::<f32>::new(48000 * 4);
    let (mut prod_out, mut cons_out) = output_ring.split();

    let host = cpal::default_host();
    let input = host.default_input_device().expect("无输入设备");
    let output = host.default_output_device().expect("无输出设备");

    let input_config: cpal::StreamConfig = input.default_input_config().unwrap().into();
    let output_config: cpal::StreamConfig = output.default_output_config().unwrap().into();

    let in_channels = input_config.channels as usize;
    let out_channels = output_config.channels as usize;

    println!(
        "输入设备: {:?}, 输出设备: {:?}",
        input_config, output_config
    );

    // 输入流：多声道 -> 下混为单声道写入 input_ring
    let input_enabled = enabled.clone();
    let input_stream = input
        .build_input_stream(
            &input_config,
            move |data: &[f32], _| {
                if input_enabled.load(Ordering::SeqCst) {
                    for frame in data.chunks(in_channels) {
                        // 简单平均混合
                        let mono = if in_channels == 1 {
                            frame[0]
                        } else {
                            let mut sum = 0.0f32;
                            for &s in frame {
                                sum += s;
                            }
                            sum / (in_channels as f32)
                        };
                        let _ = prod_in.try_push(mono);
                    }
                }
            },
            move |err| eprintln!("输入流错误: {}", err),
            None,
        )
        .expect("构建输入流失败");

    // 输出流：从 output_ring 取单声道，复制到多声道播放
    let output_enabled = enabled.clone();
    let output_stream = output
        .build_output_stream(
            &output_config,
            move |data: &mut [f32], _| {
                if output_enabled.load(Ordering::SeqCst) {
                    for frame in data.chunks_mut(out_channels) {
                        let s = cons_out.try_pop().unwrap_or(0.0);
                        for ch in 0..out_channels {
                            frame[ch] = s;
                        }
                    }
                } else {
                    for v in data.iter_mut() {
                        *v = 0.0;
                    }
                }
            },
            move |err| eprintln!("输出流错误: {}", err),
            None,
        )
        .expect("构建输出流失败");

    // 降噪线程：该线程“独占” ORT 环境与 Session（避免 Send/Sync 问题）
    let thread_running = running.clone();
    let thread_enabled = enabled.clone();

    let handle = thread::spawn(move || {
        // —— 在推理线程内部创建 ORT 环境与模型会话 ——
        let environment = Environment::builder()
            .with_name("denoise")
            .with_log_level(LoggingLevel::Warning)
            .build()
            .expect("创建 ORT 环境失败");

        // 这里根据需要设置优化等级/线程数
        let mut session = environment
            .new_session_builder()
            .expect("创建 SessionBuilder 失败")
            // .with_optimization_level(onnxruntime::GraphOptimizationLevel::All).unwrap()
            // .with_number_threads(1).unwrap()
            .with_model_from_file("public/onnx_model.onnx")
            .expect("加载 ONNX 模型失败");

        // 预取 I/O 名称
        // let input_name = session.inputs[0].name.clone();
        let _output_name = session.outputs[0].name.clone();

        // 根据你的模型输入 shape 调整
        let frame_size = 512usize;

        let mut frame_buf: Vec<f32> = Vec::with_capacity(frame_size);
        println!("🎤 降噪线程已启动");

        while thread_running.load(Ordering::SeqCst) {
            if !thread_enabled.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(10));
                continue;
            }

            // 聚齐一帧
            while frame_buf.len() < frame_size && thread_running.load(Ordering::SeqCst) {
                if let Some(s) = cons_in.try_pop() {
                    frame_buf.push(s);
                } else {
                    thread::sleep(Duration::from_millis(2));
                }
            }
            if frame_buf.len() < frame_size {
                break; // 线程准备退出
            }

            // 构造张量（示例：[1,1,frame_size]），按你的模型实际需求修改
            let input_tensor = Array::from_shape_vec((1, 1, frame_size), frame_buf.clone())
                .expect("构造输入张量失败");

            // ❗ 这里改成不带名字的输入
            // let outputs = session.run(vec![input_tensor.into_dyn()]);
            let outputs = session.run(vec![
                (session.inputs[0].name.as_str(), real.into_dyn()),
                (session.inputs[1].name.as_str(), imag.into_dyn()),
            ])?;

            if let Ok(out) = outputs {
                // ✅ 方式 A：兼容性最好
                let out_view = out[0].view();
                for &s in out_view.iter() {
                    let _ = prod_out.try_push(s);
                }
            } else {
                eprintln!("ONNX 推理失败: {:?}", outputs.err());
            }

            frame_buf.clear();
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
pub fn stop_noise_reduction() -> String {
    let thread_cell = AUDIO_THREAD.get_or_init(|| Arc::new(Mutex::new(None)));
    let mut guard = thread_cell.lock().unwrap();

    if let Some(state) = guard.as_mut() {
        state.enabled.store(false, Ordering::SeqCst);
        state.running.store(false, Ordering::SeqCst);

        if let Some(handle) = state.handle.take() {
            handle.join().ok();
        }

        *guard = None;
        return "降噪已停止".into();
    }

    "当前没有正在运行的降噪任务".into()
}

#[tauri::command]
pub fn open_listener() -> String {
    let thread_cell = AUDIO_THREAD.get_or_init(|| Arc::new(Mutex::new(None)));
    let mut guard = thread_cell.lock().unwrap();

    if let Some(state) = guard.as_mut() {
        if let Some(stream) = &state.input_stream {
            if stream.play().is_err() {
                return "输入流播放失败".into();
            }
        }
        if let Some(stream) = &state.output_stream {
            if stream.play().is_err() {
                return "输出流播放失败".into();
            }
        }
        state.enabled.store(true, Ordering::SeqCst);
        return "监听已打开".into();
    }

    "降噪线程未启动".into()
}

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

/**安装虚拟麦克风 */

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

    "✅ 安装成功".into()
}

// 检查设备是否安装成功（示例：列出音频设备名称）
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
    "安装成功".into()
}
