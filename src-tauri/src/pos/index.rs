use windows::{
    // core::PCWSTR,
    Win32::Foundation::HWND,
    Win32::UI::WindowsAndMessaging::{
        EnumWindows, FindWindowW, GetClassNameW, GetDesktopWindow, SetWindowPos,
    },
};

// use crate::utils::util::str_to_pcwstr;

#[tauri::command]
pub fn get_all_windows() -> String {
    let all_windows;

    all_windows = enum_windows_vec();
    if all_windows.len() == 0 {
        println!("未找到窗口");
        // return 2;
    }

    println!("找到 {} 个窗口", all_windows.len());

    // String::from("获取窗口成功")
    // 当字段名和变量名相同时可以简写
    // let data = WindowInfo { pid, sun_value };
    let mut window_list = Vec::new();
    for window in all_windows {
        let data = WindowInfo {
            hwnd: window.hwnd,
            title: window.title,
            class_name: window.class_name,
        };

        // if data.class_name != "IME" && data.class_name != "MSCTFIME UI" {
        //     println!("窗口{:?}", data);
        //     let s: String = serde_json::to_string(&data).unwrap();
        // }
        window_list.push(data);
    }

    let s: String = serde_json::to_string(&window_list).unwrap();
    return s;
}

#[derive(serde::Serialize, Debug)]
struct WindowInfo {
    hwnd: isize,
    title: String,
    class_name: String,
}

fn enum_windows_vec() -> Vec<WindowInfo> {
    let mut all_top_window = Vec::new();

    unsafe extern "system" fn enum_windows_proc(
        hwnd: windows::Win32::Foundation::HWND,
        lparam: windows::Win32::Foundation::LPARAM,
    ) -> windows::Win32::Foundation::BOOL {
        // 获取窗口标题长度
        let length = windows::Win32::UI::WindowsAndMessaging::GetWindowTextLengthW(hwnd);
        if length > 0 {
            // 创建一个足够大的缓冲区来存储标题
            let mut buffer = vec![0u16; (length + 1) as usize];
            // 获取窗口标题
            windows::Win32::UI::WindowsAndMessaging::GetWindowTextW(hwnd, &mut buffer);
            let title = String::from_utf16_lossy(&buffer[..length as usize]);

            // 获取窗口类
            let class_name = GetClassNameW(hwnd, &mut buffer);

            // SAFETY: lparam is a pointer to Vec<WindowInfo>
            let all_top_window = &mut *(lparam.0 as *mut Vec<WindowInfo>);
            all_top_window.push(WindowInfo {
                hwnd: hwnd.0,
                title,
                class_name: String::from_utf16_lossy(&buffer[..class_name as usize]),
            });
        }
        // 继续枚举
        windows::Win32::Foundation::BOOL(1)
    }

    unsafe {
        let lparam = windows::Win32::Foundation::LPARAM(
            &mut all_top_window as *mut Vec<WindowInfo> as isize,
        );
        let _ = EnumWindows(Some(enum_windows_proc), lparam);
    }

    all_top_window
}

#[tauri::command]
pub fn set_window_pos_command(
    hwnd: isize,
    insert_after: isize,
    x: isize,
    y: isize,
    cx: isize,
    cy: isize,
    u_flags: u32,
) {
    println!("hwnd {}, insertafter {} x {} y {} cx {} cy {} uFlags {}", hwnd, insert_after, x, y, cx, cy, u_flags);
    unsafe {
        // 将 isize 转为 HWND，满足 SetWindowPos 的 IntoParam<HWND> 约束
        let hwnd_param = HWND(hwnd);
        let insert_after_param = HWND(insert_after);

        let _bool = SetWindowPos(
            hwnd_param,
            insert_after_param,
            x as i32,
            y as i32,
            cx as i32,
            cy as i32,
            windows::Win32::UI::WindowsAndMessaging::SET_WINDOW_POS_FLAGS(u_flags),
        );
        match _bool {
            Ok(_) => {
                println!("设置窗口位置成功");
            }
            Err(e) => {
                println!("设置窗口位置失败 {:?}", e);
            }
        };
    }
}
