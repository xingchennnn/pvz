// use crate::memory::error::MemoryError;
use windows::{core::Array, Win32::{
    // Foundation::HANDLE,
    Foundation::{CloseHandle, HANDLE},
    System::{
        Diagnostics::Debug::*,
        Threading::{OpenProcess, PROCESS_ALL_ACCESS},
    },
}};

pub struct MemoryReader {
    handle: HANDLE,
}

impl MemoryReader {
    pub fn new(pid: u32) -> Result<Self, windows::core::Error> {
        // 打开进程句柄（这里需要实现具体逻辑）
        let handle = unsafe {
            OpenProcess(
                PROCESS_ALL_ACCESS, // 至少需要读权限
                false,
                pid,
            )?
        };

        Ok(Self { handle })
    }

    pub fn read<T: Copy>(&self, address: u64) -> Result<T, windows::core::Error> {
        let size = std::mem::size_of::<T>();
        let mut buffer = vec![0u8; size];
        let mut bytes_read = 0;

        unsafe {
            ReadProcessMemory(
                self.handle,
                address as *const _,
                buffer.as_mut_ptr() as _,
                size,
                Some(&mut bytes_read),
            )?;
        }

        if bytes_read != size {
            return Err(windows::core::Error::new(
                windows::core::HRESULT(0x400), // STATUS_BAD_LENGTH
                "读取长度不完整".into(),
            ));
        }

        // 安全转换
        let ptr = buffer.as_ptr() as *const T;
        Ok(unsafe { std::ptr::read_unaligned(ptr) })
    }

    // 可选的写入方法
    pub fn write<T: Copy>(&self, address: u64, value: T) -> Result<(), windows::core::Error> {
        let size = std::mem::size_of::<T>();
        let value_ptr = &value as *const T;

        let mut bytes_written = 0;
        unsafe {
            WriteProcessMemory(
                self.handle,
                address as *mut _,
                value_ptr as _,
                size,
                Some(&mut bytes_written),
            )?;
        }

        if bytes_written != size {
            return Err(windows::core::Error::new(
                windows::core::HRESULT(0x400), // STATUS_BAD_LENGTH
                "写入长度不完整".into(),
            ));
        }

        Ok(())
    }

    // 冷却写入方法
    pub fn write_cold(&self, address: u64, data: [i32;2]) -> Result<(), windows::core::Error> {
        let mut bytes_written = 0;

        unsafe {
            WriteProcessMemory(
                self.handle,
                address as *mut _,
                data.as_ptr() as _,
                data.len(),
                Some(&mut bytes_written),
            )
        }?;

        if bytes_written != data.len() {
            return Err(windows::core::Error::new(
                windows::core::HRESULT(0x400), // STATUS_BAD_LENGTH
                "写入长度不完整".into(),
            ));
        }


        Ok(())
    }

    // 关闭进程句柄
    pub fn close(&self) -> Result<(), windows::core::Error> {
        let _result = unsafe { CloseHandle(self.handle) };
        // _result.map_err(|e| {
        //     return windows::core::Error::new(windows::core::HRESULT(0x600), "关闭句柄失败".into());
        // });

        Ok(())
    }
}

// 实现 Drop 确保资源自动释放
impl Drop for MemoryReader {
    fn drop(&mut self) {
        if !self.handle.is_invalid() {
            // println!("MemoryReader drop");
            let _ = unsafe { CloseHandle(self.handle) };
        }
    }
}
