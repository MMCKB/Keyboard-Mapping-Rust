//! 极简启动日志：`%LOCALAPPDATA%\KeyRemapper\startup.log`。
//!
//! 发布版是 `windows_subsystem = "windows"`，没有控制台，
//! 启动期一旦在 WinUI 内部抛异常就什么都看不到，因此把关键步骤落到文件里。

use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn log_path() -> Option<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")?;
    let directory = PathBuf::from(base).join("KeyRemapper");
    std::fs::create_dir_all(&directory).ok()?;
    Some(directory.join("startup.log"))
}

fn timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}.{:03}", now.as_secs(), now.subsec_millis())
}

pub fn write(message: &str) {
    let Some(path) = log_path() else { return };
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = writeln!(file, "[{}] {}", timestamp(), message);
    }
}

/// 启动时调用：截断过大的旧日志，并接管 panic 输出。
pub fn init() {
    if let Some(path) = log_path() {
        if let Ok(metadata) = std::fs::metadata(&path) {
            if metadata.len() > 512 * 1024 {
                let _ = std::fs::remove_file(&path);
            }
        }
    }

    std::panic::set_hook(Box::new(|info| {
        let location = info
            .location()
            .map(|location| {
                format!(
                    "{}:{}:{}",
                    location.file(),
                    location.line(),
                    location.column()
                )
            })
            .unwrap_or_else(|| "unknown".to_string());

        let message = if let Some(text) = info.payload().downcast_ref::<&str>() {
            (*text).to_string()
        } else if let Some(text) = info.payload().downcast_ref::<String>() {
            text.clone()
        } else {
            "unknown panic".to_string()
        };

        write(&format!(
            "PANIC at {location}: {message}\n{}",
            std::backtrace::Backtrace::force_capture()
        ));
    }));
}
