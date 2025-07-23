// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod memory;
mod utils;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn call_rust(types: &str, text: &str) -> std::string::String {
    // println!("call_rust: {} , text: {}", types, text);
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
        _ => {
            return "hahahh".to_string();
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, call_rust])
     
        .run(tauri::generate_context!())
        .expect("启动失败");
}
