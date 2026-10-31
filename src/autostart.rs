//! 开机自启：写 `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`。

use crate::win32::*;

const RUN_KEY_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE_NAME: &str = "KeyRemapper";

pub fn is_enabled() -> bool {
    let path = wide(RUN_KEY_PATH);
    let name = wide(VALUE_NAME);

    unsafe {
        let mut key: HKEY = 0;
        if RegOpenKeyExW(HKEY_CURRENT_USER, path.as_ptr(), 0, KEY_QUERY_VALUE, &mut key)
            != ERROR_SUCCESS
        {
            return false;
        }

        let mut length = 0u32;
        let rc = RegQueryValueExW(
            key,
            name.as_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut length,
        );
        RegCloseKey(key);

        rc == ERROR_SUCCESS && length > 0
    }
}

pub fn set(enabled: bool) -> Result<(), String> {
    let path = wide(RUN_KEY_PATH);
    let name = wide(VALUE_NAME);

    unsafe {
        let mut key: HKEY = 0;
        let rc = RegCreateKeyExW(
            HKEY_CURRENT_USER,
            path.as_ptr(),
            0,
            std::ptr::null(),
            0,
            KEY_SET_VALUE,
            std::ptr::null(),
            &mut key,
            std::ptr::null_mut(),
        );
        if rc != ERROR_SUCCESS {
            return Err(format!("打开注册表失败（错误码 {rc}）。"));
        }

        let rc = if enabled {
            let exe = std::env::current_exe().map_err(|error| error.to_string())?;
            let command = format!("\"{}\"", exe.display());
            let value: Vec<u16> = command.encode_utf16().chain(std::iter::once(0)).collect();
            RegSetValueExW(
                key,
                name.as_ptr(),
                0,
                REG_SZ,
                value.as_ptr() as *const u8,
                (value.len() * 2) as u32,
            )
        } else {
            RegDeleteValueW(key, name.as_ptr())
        };
        RegCloseKey(key);

        if rc != ERROR_SUCCESS && rc != ERROR_FILE_NOT_FOUND {
            return Err(format!("写入注册表失败（错误码 {rc}）。"));
        }
    }

    Ok(())
}
