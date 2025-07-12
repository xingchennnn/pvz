use crate::memory::error::MemoryError; // 导入自定义的错误类型MemoryError
use crate::utils::memory_reader::MemoryReader;
use crate::utils::util::str_to_pcwstr;
use serde::{Deserialize, Serialize};
use serde_json::Value; // 导入serde_json
use std::ffi::c_void;
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{CloseHandle, HANDLE},
        System::{
            Diagnostics::Debug::ReadProcessMemory,
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, Module32First, Module32Next, MODULEENTRY32,
                TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32,
            },
            Threading::{
                OpenProcess, PROCESS_ALL_ACCESS, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
            },
        },
        UI::WindowsAndMessaging::{FindWindowW, GetWindowThreadProcessId},
    },
};

#[derive(Serialize, Deserialize)]
struct DataStruct {
    pid: u32,
    sun_value: u32,
}

/** 增加阳光 */
pub fn add_sun(text: &str) -> Result<String, MemoryError> {
    println!("add_sun 接收到的参数{}", text);
    let pid = get_pid();
    // 处理无效的PID情况
    match pid {
        0 | 3 => return Err(MemoryError::ProcessIdNotFound),
        2 => return Err(MemoryError::WindowNotFound),
        _ => {} // PID有效，继续执行
    }

    // 创建内存读取器
    let reader =
        MemoryReader::new(pid).map_err(|e| MemoryError::ReaderCreationFailed(e.to_string()))?;

    // 定义指针链偏移量
    let offsets = vec![0x768, 0x5560];

    // 读取阳光值
    let (sun_value, _address):  (u32, u64)= read_pointer_chain(&reader, 0x006A9F38, &offsets, true)?;
    // println!("阳光值：{}", sun_value);

    // 修改阳光值（可选）
    let new_sun_value = sun_value + 5000;
    println!("修改后的阳光值：{}",new_sun_value);
    reader
        .write(_address, new_sun_value)
        .map_err(|e| MemoryError::MemoryWriteFailed(e.to_string()))?;

    // 返回成功消息
    Ok(format!("成功! 进程ID: {}, 阳光值: {}", pid, sun_value))
}

/** 读取阳光值 */
pub fn read_sun_value() -> Result<String, MemoryError> {
    // println!("add_sun 接收到的参数{}", text);
    let pid = get_pid();
    // 处理无效的PID情况
    match pid {
        0 | 3 => return Err(MemoryError::ProcessIdNotFound),
        2 => return Err(MemoryError::WindowNotFound),
        _ => {} // PID有效，继续执行
    }

    // 创建内存读取器
    let reader =
        MemoryReader::new(pid).map_err(|e| MemoryError::ReaderCreationFailed(e.to_string()))?;

    // 定义指针链偏移量
    let offsets = vec![0x768, 0x5560];

    // 读取阳光值
    let (sun_value, _address): (u32, u64) = read_pointer_chain(&reader, 0x006A9F38, &offsets, true)?;

    // 当字段名和变量名相同时可以简写
    let data = DataStruct { pid, sun_value };

    let s: String = serde_json::to_string(&data).unwrap();
    // 返回成功消息
    Ok(s)
}

/** 获取植物大战僵尸杂交版v3.9的进程id */
fn get_pid() -> u32 {
    let find_name = "植物大战僵尸杂交版v3.9              ";
    let hwnd;
    // findWindowW 找到窗口句柄
    unsafe {
        hwnd = FindWindowW(
            PCWSTR::from_raw(str_to_pcwstr("MainWindow").as_ptr()),
            PCWSTR::from_raw(str_to_pcwstr(find_name).as_ptr()),
        );
        if hwnd.0 == 0 {
            println!("未找到窗口");
            return 2;
        }
    };

    // 获取窗口的进程id
    let mut process_id = 0;

    unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));
    }

    // 如果进程id为0，说明未获取到窗口进程id
    if process_id == 0 {
        println!("未获取到窗口进程id");
        return 3;
    }
    // println!("窗口的进程id为：{}", process_id);

    return process_id;
}

/// 读取多级指针链
pub fn read_pointer_chain<T: Copy>(
    reader: &MemoryReader,
    base_address: u64,
    offsets: &[u64],
    is_32bit: bool, // 新增参数
) -> Result<(T, u64), MemoryError> {
    let mut current_address = base_address;

    // 遍历所有偏移量（除了最后一个）
    for (i, offset) in offsets.iter().enumerate() {
        // println!("第{}级指针地址：0x{:X}", i + 1, current_address);
        // 根据架构读取指针值
        let ptr_value = if is_32bit {
            // 32位进程读取u32后扩展为u64
            reader
                .read::<u32>(current_address)
                .map(|v| v as u64)
                .map_err(|e| {
                    MemoryError::MemoryReadFailed(format!(
                        "第{}级指针读取失败 (0x{:X}): {}",
                        i + 1,
                        current_address,
                        e
                    ))
                })?
        } else {
            // 64位进程直接读取u64
            reader.read::<u64>(current_address).map_err(|e| {
                MemoryError::MemoryReadFailed(format!(
                    "第{}级指针读取失败 (0x{:X}): {}",
                    i + 1,
                    current_address,
                    e
                ))
            })?
        };

        // 计算下一级地址 = 指针值 + 当前偏移
        current_address = ptr_value + offset;

        // println!("第{}级指针读数：0x{:X} ， 偏移量：0x{:X} ， 计算后地址：0x{:X}",
        //     i + 1, ptr_value, offset, current_address);

        // +5560算完了才是指针地址，所以不用去掉最后一个偏移量 current_address 就是取阳光的地址了 ， 下面直接读 就是阳光的值了
    }

    // 读取最终值
    let sun_value = reader.read::<T>(current_address).map_err(|e| {
        MemoryError::MemoryReadFailed(format!("最终值读取失败 (0x{:X}): {}", current_address, e))
    })?;

    //  返回两个参数 阳光值 和 地址
    Ok((sun_value, current_address))
}
