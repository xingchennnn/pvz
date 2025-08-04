// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod memory;
mod utils;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
// use cpal::{Data, FromSample, Sample, SampleFormat, SizedSample};
use ringbuf::{
    traits::{Consumer, Producer, Split},
    HeapRb,
};
// use std::sync::{Arc, Mutex};
mod audio;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn call_rust(types: &str, text: &str) -> std::string::String {
    println!("call_rust: {} , text: {}", types, text);
    match types {
        "AddSun" => {
            // 使用match来处理Result，将错误转换为字符串
            match memory::operat::add_sun(text) {
                Ok(result) => result,
                Err(e) => format!("错误: {}", e),
            }
        }
        "GetSun" => {
            // 调用读取阳光值
            match memory::operat::read_sun_value() {
                Ok(result) => result,
                Err(e) => format!("错误: {}", e),
            }
        }
        "cooling" => {
            // 调用冷却
            match memory::operat::cooling() {
                Ok(result) => result,
                Err(e) => format!("错误: {}", e),
            }
        }
        "getDefaultAudioDevice" => {
            //  获取默认host
            let host = cpal::default_host();
            //  获取默认输出设备
            let output_device = host.default_output_device().expect("没有默认输出设备");
            println!(
                "默认音频输出设备: {}",
                output_device.clone().name().unwrap()
            );
            let output_config = output_device
                .default_output_config()
                .expect("无法获取输出配置");

            //  获取默认输入设备
            let input_device = host.default_input_device().expect("没有默认输入设备");
            println!("默认音频输入设备: {}", input_device.clone().name().unwrap());
            //  获取默认输入配置
            let input_config = input_device
                .default_input_config()
                .expect("无法获取输入配置");

            //  获取默认输出配置
            // let sample_format = input_config.sample_format();
            let input_config: cpal::StreamConfig = input_config.into();
            let output_config: cpal::StreamConfig = output_config.into();
            // 电音处理
            let input_sample_rate = input_config.sample_rate;
            let output_config = cpal::StreamConfig {
                channels: output_config.channels,
                sample_rate: input_sample_rate, // 强制一致
                buffer_size: cpal::BufferSize::Default,
            };

            // 创建一个缓冲区：线程安全共享数据
            // let audio_buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));

            // 创建一个环形缓冲区
            let ring = HeapRb::<f32>::new(48000 * 2);
            let (mut prod, mut cons) = ring.split();

            // 启动输入流（麦克风）
            let input_stream = input_device
                .build_input_stream(
                    &input_config,
                    move |data: &[f32], _| {
                        for &sample in data {
                            let _ = prod.try_push(sample);
                        }
                    },
                    move |err| eprintln!("输入错误: {}", err),
                    None,
                )
                .expect("创建输入流失败");

            let output_stream = output_device
                .build_output_stream(
                    &output_config,
                    move |data: &mut [f32], _| {
                        for frame in data.chunks_mut(2) {
                            let sample = cons.try_pop().unwrap_or(0.0);
                            frame[0] = sample;
                            frame[1] = sample;
                        }
                    },
                    move |err| eprintln!("输出错误: {}", err),
                    None,
                )
                .expect("创建输出流失败");

            println!("输入设备格式: {:?}", input_config);
            println!("输出设备格式: {:?}", output_config);

            input_stream.play().expect("无法播放输入流");
            output_stream.play().expect("无法播放输出流");

            println!("🎤 正在实时传输音频数据（麦克风 → 扬声器）...");

            // 等待 ctrl+c
            // std::thread::park();

            "默认音频设备".to_string()
        }
        _ => {
            let mut _string = String::from("你输入的是:");
            _string.push_str(text);
            _string
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            call_rust,
            audio::device::start_noise_reduction,
            audio::device::stop_noise_reduction,
            audio::device::open_listener,
            audio::device::close_listener,
            // audio::device::get_listener_status,
        ])
        .run(tauri::generate_context!())
        .expect("启动失败");
}
