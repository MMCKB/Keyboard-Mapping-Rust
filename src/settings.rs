//! 配置模型与持久化：`%LOCALAPPDATA%\KeyRemapper\settings.json`。
//!
//! JSON 字段名与旧版（C# / System.Text.Json 默认 PascalCase）保持一致，
//! 因此两个版本可以共用同一份配置文件。

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 一条映射规则：把 `from_vk` 替换成 `to_vk`。
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct KeyMapping {
    pub from_vk: u16,
    pub from_name: String,
    pub to_vk: u16,
    pub to_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct AppSettings {
    pub enabled: bool,
    pub mappings: Vec<KeyMapping>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            mappings: Vec::new(),
        }
    }
}

fn directory() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("KeyRemapper")
}

fn settings_path() -> PathBuf {
    directory().join("settings.json")
}

pub fn load() -> AppSettings {
    match std::fs::read_to_string(settings_path()) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
        Err(_) => AppSettings::default(),
    }
}

pub fn save(settings: &AppSettings) {
    let directory = directory();
    if std::fs::create_dir_all(&directory).is_err() {
        return;
    }
    if let Ok(text) = serde_json::to_string_pretty(settings) {
        let _ = std::fs::write(directory.join("settings.json"), text);
    }
}
