// src/memory/error.rs
use std::fmt;

#[derive(Debug)]
pub enum MemoryError {
    ProcessIdNotFound,
    WindowNotFound,
    ReaderCreationFailed(String),
    MemoryReadFailed(String),
    MemoryWriteFailed(String),
    Unknown,
}

impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MemoryError::ProcessIdNotFound => write!(f, "未获取到窗口进程ID"),
            MemoryError::WindowNotFound => write!(f, "未找到目标窗口"),
            MemoryError::ReaderCreationFailed(msg) => write!(f, "创建内存读取器失败: {}", msg),
            MemoryError::MemoryReadFailed(msg) => write!(f, "内存读取失败: {}", msg),
            MemoryError::MemoryWriteFailed(msg) => write!(f, "内存写入失败: {}", msg),
            MemoryError::Unknown => write!(f, "未知错误"),
        }
    }
}

impl std::error::Error for MemoryError {}