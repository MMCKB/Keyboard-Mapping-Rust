//! 极简 Win32 FFI 声明：只包含本项目用到的键盘钩子、输入注入与注册表 API。
//!
//! 这里手写绑定而不是引入 windows-sys，是为了让依赖面保持最小，
//! 同时避免与 windows-reactor 内部的 windows-core 版本产生耦合。

#![allow(non_snake_case, non_camel_case_types, dead_code)]

use std::ffi::c_void;

pub type BOOL = i32;
pub type HHOOK = isize;
pub type HWND = isize;
pub type HINSTANCE = isize;
pub type HKEY = isize;
pub type WPARAM = usize;
pub type LPARAM = isize;
pub type LRESULT = isize;

pub const WH_KEYBOARD_LL: i32 = 13;

pub const WM_KEYDOWN: u32 = 0x0100;
pub const WM_KEYUP: u32 = 0x0101;
pub const WM_SYSKEYDOWN: u32 = 0x0104;
pub const WM_SYSKEYUP: u32 = 0x0105;
pub const WM_QUIT: u32 = 0x0012;

pub const LLKHF_INJECTED: u32 = 0x10;
pub const LLKHF_LOWER_IL_INJECTED: u32 = 0x02;

pub const KEYEVENTF_EXTENDEDKEY: u32 = 0x0001;
pub const KEYEVENTF_KEYUP: u32 = 0x0002;
pub const INPUT_KEYBOARD: u32 = 1;

pub const HKEY_CURRENT_USER: HKEY = 0x80000001u32 as isize;
pub const HKEY_LOCAL_MACHINE: HKEY = 0x80000002u32 as isize;

pub const KEY_QUERY_VALUE: u32 = 0x0001;
pub const KEY_SET_VALUE: u32 = 0x0002;

pub const REG_SZ: u32 = 1;
pub const REG_BINARY: u32 = 3;

pub const ERROR_SUCCESS: i32 = 0;
pub const ERROR_FILE_NOT_FOUND: i32 = 2;
pub const ERROR_ACCESS_DENIED: i32 = 5;

pub const MAPVK_VK_TO_VSC_EX: u32 = 4;

pub const SW_SHOWNORMAL: i32 = 1;
pub const MB_ICONINFORMATION: u32 = 0x0000_0040;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct KbdLlHookStruct {
    pub vk_code: u32,
    pub scan_code: u32,
    pub flags: u32,
    pub time: u32,
    pub dw_extra_info: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MouseInput {
    pub dx: i32,
    pub dy: i32,
    pub mouse_data: u32,
    pub flags: u32,
    pub time: u32,
    pub dw_extra_info: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KbdInput {
    pub w_vk: u16,
    pub w_scan: u16,
    pub flags: u32,
    pub time: u32,
    pub dw_extra_info: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HardwareInput {
    pub msg: u32,
    pub param_l: u16,
    pub param_h: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union InputUnion {
    pub ki: KbdInput,
    pub mi: MouseInput,
    pub hi: HardwareInput,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Input {
    pub kind: u32,
    pub data: InputUnion,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Msg {
    pub hwnd: HWND,
    pub message: u32,
    pub w_param: WPARAM,
    pub l_param: LPARAM,
    pub time: u32,
    pub pt_x: i32,
    pub pt_y: i32,
}

impl Default for Msg {
    fn default() -> Self {
        Self {
            hwnd: 0,
            message: 0,
            w_param: 0,
            l_param: 0,
            time: 0,
            pt_x: 0,
            pt_y: 0,
        }
    }
}

pub type HookProc = Option<unsafe extern "system" fn(i32, WPARAM, LPARAM) -> LRESULT>;

#[link(name = "user32")]
unsafe extern "system" {
    pub fn SetWindowsHookExW(
        id_hook: i32,
        hook_proc: HookProc,
        module: HINSTANCE,
        thread_id: u32,
    ) -> HHOOK;
    pub fn UnhookWindowsHookEx(hook: HHOOK) -> BOOL;
    pub fn CallNextHookEx(hook: HHOOK, code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT;
    pub fn GetMessageW(msg: *mut Msg, hwnd: HWND, filter_min: u32, filter_max: u32) -> BOOL;
    pub fn TranslateMessage(msg: *const Msg) -> BOOL;
    pub fn DispatchMessageW(msg: *const Msg) -> LRESULT;
    pub fn PostThreadMessageW(thread_id: u32, msg: u32, wparam: WPARAM, lparam: LPARAM) -> BOOL;
    pub fn SendInput(count: u32, inputs: *const Input, size: i32) -> u32;
    pub fn MessageBoxW(hwnd: HWND, text: *const u16, caption: *const u16, flags: u32) -> i32;
    pub fn MapVirtualKeyW(code: u32, map_type: u32) -> u32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    pub fn GetModuleHandleW(name: *const u16) -> HINSTANCE;
    pub fn GetCurrentThreadId() -> u32;
}

#[link(name = "shell32")]
unsafe extern "system" {
    pub fn ShellExecuteW(
        hwnd: HWND,
        operation: *const u16,
        file: *const u16,
        parameters: *const u16,
        directory: *const u16,
        show: i32,
    ) -> HINSTANCE;
}

#[link(name = "advapi32")]
unsafe extern "system" {
    pub fn RegOpenKeyExW(
        key: HKEY,
        sub_key: *const u16,
        options: u32,
        sam_desired: u32,
        result: *mut HKEY,
    ) -> i32;
    pub fn RegCreateKeyExW(
        key: HKEY,
        sub_key: *const u16,
        reserved: u32,
        class: *const u16,
        options: u32,
        sam_desired: u32,
        security: *const c_void,
        result: *mut HKEY,
        disposition: *mut u32,
    ) -> i32;
    pub fn RegSetValueExW(
        key: HKEY,
        value_name: *const u16,
        reserved: u32,
        kind: u32,
        data: *const u8,
        len: u32,
    ) -> i32;
    pub fn RegDeleteValueW(key: HKEY, value_name: *const u16) -> i32;
    pub fn RegQueryValueExW(
        key: HKEY,
        value_name: *const u16,
        reserved: *mut u32,
        kind: *mut u32,
        data: *mut u8,
        len: *mut u32,
    ) -> i32;
    pub fn RegCloseKey(key: HKEY) -> i32;
}

/// 把 Rust 字符串转成以 NUL 结尾的 UTF-16 缓冲区。
pub fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

/// 简单的系统消息框（用于提权子进程回报结果）。
pub fn message_box(text: &str, caption: &str) {
    let text = wide(text);
    let caption = wide(caption);
    unsafe {
        MessageBoxW(0, text.as_ptr(), caption.as_ptr(), MB_ICONINFORMATION);
    }
}
