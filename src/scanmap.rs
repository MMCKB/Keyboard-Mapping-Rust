//! 系统级映射：写入 `HKLM\SYSTEM\CurrentControlSet\Control\Keyboard Layout` 的
//! `Scancode Map` 值（键盘驱动层重映射，与 SharpKeys 同款机制）。
//!
//! 写入/删除后需重启电脑生效；生效后完全不依赖本软件运行，也不需要开机自启。

use crate::settings::KeyMapping;
use crate::win32::*;

const KEY_PATH: &str = r"SYSTEM\CurrentControlSet\Control\Keyboard Layout";
const VALUE_NAME: &str = "Scancode Map";

#[derive(Debug)]
pub enum ScanmapError {
    /// 缺少管理员权限，可调用 `relaunch_elevated` 走 UAC。
    NeedAdmin,
    Failed(String),
}

impl ScanmapError {
    pub fn message(&self) -> String {
        match self {
            ScanmapError::NeedAdmin => "需要管理员权限。".to_string(),
            ScanmapError::Failed(text) => text.clone(),
        }
    }
}

pub fn exists() -> bool {
    let path = wide(KEY_PATH);
    let name = wide(VALUE_NAME);

    unsafe {
        let mut key: HKEY = 0;
        if RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            path.as_ptr(),
            0,
            KEY_QUERY_VALUE,
            &mut key,
        ) != ERROR_SUCCESS
        {
            return false;
        }

        let mut kind = 0u32;
        let mut length = 0u32;
        let rc = RegQueryValueExW(
            key,
            name.as_ptr(),
            std::ptr::null_mut(),
            &mut kind,
            std::ptr::null_mut(),
            &mut length,
        );
        RegCloseKey(key);

        rc == ERROR_SUCCESS && kind == REG_BINARY && length > 12
    }
}

pub fn apply(mappings: &[KeyMapping]) -> Result<(), ScanmapError> {
    let data = build(mappings)?;
    let path = wide(KEY_PATH);
    let name = wide(VALUE_NAME);

    unsafe {
        let mut key: HKEY = 0;
        let rc = RegCreateKeyExW(
            HKEY_LOCAL_MACHINE,
            path.as_ptr(),
            0,
            std::ptr::null(),
            0,
            KEY_SET_VALUE,
            std::ptr::null(),
            &mut key,
            std::ptr::null_mut(),
        );
        if rc == ERROR_ACCESS_DENIED {
            return Err(ScanmapError::NeedAdmin);
        }
        if rc != ERROR_SUCCESS {
            return Err(ScanmapError::Failed(format!("打开注册表失败（错误码 {rc}）。")));
        }

        let rc = RegSetValueExW(
            key,
            name.as_ptr(),
            0,
            REG_BINARY,
            data.as_ptr(),
            data.len() as u32,
        );
        RegCloseKey(key);

        if rc == ERROR_ACCESS_DENIED {
            return Err(ScanmapError::NeedAdmin);
        }
        if rc != ERROR_SUCCESS {
            return Err(ScanmapError::Failed(format!("写入注册表失败（错误码 {rc}）。")));
        }
    }

    Ok(())
}

pub fn remove() -> Result<(), ScanmapError> {
    let path = wide(KEY_PATH);
    let name = wide(VALUE_NAME);

    unsafe {
        let mut key: HKEY = 0;
        let rc = RegOpenKeyExW(HKEY_LOCAL_MACHINE, path.as_ptr(), 0, KEY_SET_VALUE, &mut key);
        if rc == ERROR_ACCESS_DENIED {
            return Err(ScanmapError::NeedAdmin);
        }
        if rc == ERROR_FILE_NOT_FOUND {
            return Ok(());
        }
        if rc != ERROR_SUCCESS {
            return Err(ScanmapError::Failed(format!("打开注册表失败（错误码 {rc}）。")));
        }

        let rc = RegDeleteValueW(key, name.as_ptr());
        RegCloseKey(key);

        if rc == ERROR_ACCESS_DENIED {
            return Err(ScanmapError::NeedAdmin);
        }
        if rc != ERROR_SUCCESS && rc != ERROR_FILE_NOT_FOUND {
            return Err(ScanmapError::Failed(format!("删除注册表值失败（错误码 {rc}）。")));
        }
    }

    Ok(())
}

/// 以管理员身份重新启动本程序执行指定操作（触发 UAC），
/// 由命令行参数分支完成实际写入。
pub fn relaunch_elevated(arguments: &str) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
    let file = wide(&exe.to_string_lossy());
    let operation = wide("runas");
    let parameters = wide(arguments);

    let result = unsafe {
        ShellExecuteW(
            0,
            operation.as_ptr(),
            file.as_ptr(),
            parameters.as_ptr(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    };

    if result > 32 {
        Ok(())
    } else {
        Err(format!("未能获得管理员权限（错误码 {result}）。"))
    }
}

/// Scancode Map 布局：8 字节全 0 头 + 4 字节条目数（含结束条目）
/// + N×4 字节映射条目 + 4 字节 0 结束符。
/// 每个条目低 16 位 = 映射后的扫描码，高 16 位 = 原始扫描码。
fn build(mappings: &[KeyMapping]) -> Result<Vec<u8>, ScanmapError> {
    let mut pairs: Vec<(u16, u16)> = Vec::new();

    for mapping in mappings {
        let from = map_vk_to_scan_code(mapping.from_vk);
        let to = map_vk_to_scan_code(mapping.to_vk);
        if from == 0 || to == 0 {
            return Err(ScanmapError::Failed(format!(
                "按键「{}」或「{}」没有对应的物理扫描码，无法用于系统级映射。",
                mapping.from_name, mapping.to_name
            )));
        }
        pairs.push((to, from));
    }

    let mut data = vec![0u8; 16 + pairs.len() * 4];
    let count = pairs.len() as u32 + 1;
    data[8..12].copy_from_slice(&count.to_le_bytes());

    let mut offset = 12;
    for (to, from) in pairs {
        data[offset..offset + 2].copy_from_slice(&to.to_le_bytes());
        data[offset + 2..offset + 4].copy_from_slice(&from.to_le_bytes());
        offset += 4;
    }

    Ok(data)
}

fn map_vk_to_scan_code(vk: u16) -> u16 {
    let code = unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC_EX) };
    (code & 0xFFFF) as u16
}
