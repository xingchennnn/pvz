use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread::{self, JoinHandle};
use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}, Stream};
use ringbuf::{traits::{Consumer, Producer, Split}, HeapRb};
use once_cell::sync::OnceCell;

struct AudioState {
    handle: Option<JoinHandle<()>>,
    running: Arc<AtomicBool>,
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

    let running = Arc::new(AtomicBool::new(false));

    let ring = HeapRb::<f32>::new(48000 * 2);
    let (mut prod, mut cons) = ring.split();

    let host = cpal::default_host();
    // 获取所有设备
    // let devices = host.devices();
    // println!("设备数量: {}  设备列表: ", devices );
    // 获取默认设备
    let input = host.default_input_device().expect("无输入设备");
    let output = host.default_output_device().expect("无输出设备");

    let input_config = input.default_input_config().unwrap().into();
    let output_config = output.default_output_config().unwrap().into();

    // 每个闭包中都克隆 running
    let input_running = running.clone();
    let input_stream = input.build_input_stream(
        &input_config,
        move |data: &[f32], _| {
            if input_running.load(Ordering::SeqCst) {
                for &sample in data {
                    let _ = prod.try_push(sample);
                }
            }
        },
        move |err| eprintln!("输入流错误: {}", err),
        None,
    ).unwrap();

    let output_running = running.clone();
    let output_stream = output.build_output_stream(
        &output_config,
        move |data: &mut [f32], _| {
            if output_running.load(Ordering::SeqCst) {
                for frame in data.chunks_mut(2) {
                    let sample = cons.try_pop().unwrap_or(0.0);
                    frame[0] = sample;
                    frame[1] = sample;
                }
            } else {
                for frame in data.chunks_mut(2) {
                    frame[0] = 0.0;
                    frame[1] = 0.0;
                }
            }
        },
        move |err| eprintln!("输出流错误: {}", err),
        None,
    ).unwrap();

    // 线程也要独立克隆
    let thread_running = running.clone();
    let handle = thread::spawn(move || {
        println!("🎤 降噪线程已启动");
        while thread_running.load(Ordering::SeqCst) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        // println!("🛑 降噪线程已退出");
    });

    *guard = Some(AudioState {
        handle: Some(handle),
        running,
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
        state.running.store(false, Ordering::SeqCst);

        if let Some(handle) = state.handle.take() {
            handle.join().ok(); // 等待线程退出
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
        state.running.store(true, Ordering::SeqCst);
        return "监听已打开".into();
    }

    "降噪线程未启动".into()
}

#[tauri::command]
pub fn close_listener() -> String {
    let thread_cell = AUDIO_THREAD.get_or_init(|| Arc::new(Mutex::new(None)));
    let mut guard = thread_cell.lock().unwrap();

    if let Some(state) = guard.as_mut() {
        state.running.store(false, Ordering::SeqCst);
        return "监听已关闭".into();
    }

    "降噪线程未启动".into()
}


