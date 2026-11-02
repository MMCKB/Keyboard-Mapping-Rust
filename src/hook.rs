//! 全局低级键盘钩子（WH_KEYBOARD_LL）。
//!
//! 工作方式与旧版一致：后台线程安装钩子并跑消息泵，物理按键按映射表被吞掉，
//! 再用 SendInput 注入目标键。注入事件带 `LLKHF_INJECTED` 标记，会被本钩子直接放行，
//! 因此不会形成死循环，也意味着只有物理键盘的输入会被映射。
//!
//! 额外提供「捕获」模式：在添加映射的弹窗里，下一次按下的物理键会被吞掉并回报给 UI，
//! 这样连 Win 键、媒体键这类界面收不到的按键也能被捕获。

use crate::keys::normalize;
use crate::settings::KeyMapping;
use crate::win32::*;

use std::collections::{HashMap, HashSet};
use std::time::Instant;
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU32, Ordering};
use std::sync::{mpsc, Arc, Mutex, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// 按键抬起后的余辉时间，保证快速点按也能看到高亮。
const GLOW: std::time::Duration = std::time::Duration::from_millis(250);

#[derive(Default)]
struct TesterState {
    pressed: HashSet<u16>,
    last_seen: HashMap<u16, Instant>,
}

struct Shared {
    mappings: HashMap<u16, u16>,
    held: HashMap<u16, u16>,
}

pub struct HookService {
    shared: Mutex<Shared>,
    capture: Mutex<Option<mpsc::Sender<u16>>>,
    paused: AtomicBool,
    tester: AtomicBool,
    tester_state: Mutex<TesterState>,
    hook: AtomicIsize,
    thread_id: AtomicU32,
    running: AtomicBool,
    thread: Mutex<Option<JoinHandle<()>>>,
}

static INSTANCE: OnceLock<Arc<HookService>> = OnceLock::new();

impl HookService {
    pub fn global() -> Arc<HookService> {
        INSTANCE
            .get_or_init(|| {
                Arc::new(HookService {
                    shared: Mutex::new(Shared {
                        mappings: HashMap::new(),
                        held: HashMap::new(),
                    }),
                    capture: Mutex::new(None),
                    paused: AtomicBool::new(false),
                    tester: AtomicBool::new(false),
                    tester_state: Mutex::new(TesterState::default()),
                    hook: AtomicIsize::new(0),
                    thread_id: AtomicU32::new(0),
                    running: AtomicBool::new(false),
                    thread: Mutex::new(None),
                })
            })
            .clone()
    }

    pub fn set_mappings(&self, mappings: &[KeyMapping]) {
        let mut table = HashMap::new();
        for mapping in mappings {
            let from = normalize(mapping.from_vk);
            let to = normalize(mapping.to_vk);
            if from != 0 && to != 0 && from != to {
                table.insert(from, to);
            }
        }
        if let Ok(mut shared) = self.shared.lock() {
            shared.mappings = table;
        }
    }

    /// 暂停映射（例如弹窗期间）。已注入但未抬起的键会补发抬起事件。
    pub fn set_paused(&self, paused: bool) {
        self.paused.store(paused, Ordering::SeqCst);
        if paused {
            self.release_held();
        }
    }

    /// 键盘测试模式：吞掉所有按键，并把按下/抬起记录到集合里供界面轮询。
    pub fn set_tester(&self, on: bool) {
        self.tester.store(on, Ordering::SeqCst);
        if let Ok(mut state) = self.tester_state.lock() {
            state.pressed.clear();
            state.last_seen.clear();
        }
    }

    /// 当前应该高亮的键：按住中的键，加上刚刚抬起、仍在余辉窗口内的键。
    pub fn pressed_keys(&self) -> Vec<u16> {
        let Ok(state) = self.tester_state.lock() else {
            return Vec::new();
        };
        let now = Instant::now();
        let mut keys: HashSet<u16> = state.pressed.clone();
        for (vk, seen) in &state.last_seen {
            if now.duration_since(*seen) < GLOW {
                keys.insert(*vk);
            }
        }
        keys.into_iter().collect()
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst) && self.hook.load(Ordering::SeqCst) != 0
    }

    pub fn start(self: &Arc<Self>) -> bool {
        if self.is_running() {
            return true;
        }
        self.release_held();
        self.running.store(true, Ordering::SeqCst);

        let state = Arc::clone(self);
        match thread::Builder::new()
            .name("keyremapper-hook".to_string())
            .spawn(move || state.hook_thread())
        {
            Ok(handle) => {
                if let Ok(mut slot) = self.thread.lock() {
                    *slot = Some(handle);
                }
            }
            Err(_) => {
                self.running.store(false, Ordering::SeqCst);
                return false;
            }
        }

        // 等钩子线程完成安装，UI 才能拿到准确的运行状态。
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline {
            if self.hook.load(Ordering::SeqCst) != 0 {
                return true;
            }
            if !self.running.load(Ordering::SeqCst) {
                return false;
            }
            thread::sleep(Duration::from_millis(10));
        }
        self.hook.load(Ordering::SeqCst) != 0
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        let thread_id = self.thread_id.swap(0, Ordering::SeqCst);
        if thread_id != 0 {
            unsafe {
                PostThreadMessageW(thread_id, WM_QUIT, 0, 0);
            }
        }
        let handle = self.thread.lock().ok().and_then(|mut slot| slot.take());
        if let Some(handle) = handle {
            let _ = handle.join();
        }
        self.release_held();
    }

    /// 放弃当前捕获（弹窗关闭时调用），等待中的捕获会立刻返回 None。
    pub fn clear_capture(&self) {
        if let Ok(mut slot) = self.capture.lock() {
            *slot = None;
        }
    }

    /// 等待下一次物理按键；`is_cancelled` 返回 true 时放弃。
    pub fn capture_next(&self, is_cancelled: impl Fn() -> bool) -> Option<u16> {
        let (sender, receiver) = mpsc::channel();
        match self.capture.lock() {
            Ok(mut slot) => *slot = Some(sender),
            Err(_) => return None,
        }

        loop {
            if is_cancelled() {
                if let Ok(mut slot) = self.capture.lock() {
                    *slot = None;
                }
                return None;
            }
            match receiver.recv_timeout(Duration::from_millis(50)) {
                Ok(vk) => return Some(vk),
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                // 通道被新的捕获请求替换，交给新的那次捕获处理。
                Err(mpsc::RecvTimeoutError::Disconnected) => return None,
            }
        }
    }

    fn release_held(&self) {
        let targets: Vec<u16> = match self.shared.lock() {
            Ok(mut shared) => {
                let targets = shared.held.values().copied().collect();
                shared.held.clear();
                targets
            }
            Err(_) => return,
        };
        for target in targets {
            send_key(target, false);
        }
    }

    fn hook_thread(self: Arc<Self>) {
        unsafe {
            let module = GetModuleHandleW(std::ptr::null());
            let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), module, 0);
            if hook == 0 {
                self.running.store(false, Ordering::SeqCst);
                return;
            }

            self.thread_id.store(GetCurrentThreadId(), Ordering::SeqCst);
            self.hook.store(hook, Ordering::SeqCst);

            let mut message = Msg::default();
            while GetMessageW(&mut message, 0, 0, 0) > 0 {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }

            UnhookWindowsHookEx(hook);
            self.hook.store(0, Ordering::SeqCst);
            self.running.store(false, Ordering::SeqCst);
        }
    }
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let Some(state) = INSTANCE.get() else {
            return unsafe { CallNextHookEx(0, code, wparam, lparam) };
        };

        let message = wparam as u32;
        if matches!(
            message,
            WM_KEYDOWN | WM_KEYUP | WM_SYSKEYDOWN | WM_SYSKEYUP
        ) {
            let info = unsafe { &*(lparam as *const KbdLlHookStruct) };
            let injected = info.flags & (LLKHF_INJECTED | LLKHF_LOWER_IL_INJECTED) != 0;
            if !injected {
                let down = message == WM_KEYDOWN || message == WM_SYSKEYDOWN;

                // 0) 键盘测试：记录原始键码并吞掉按键，避免触发系统快捷键。
                if state.tester.load(Ordering::Relaxed) {
                    let raw = info.vk_code as u16;
                    if let Ok(mut tester) = state.tester_state.lock() {
                        tester.last_seen.insert(raw, Instant::now());
                        if down {
                            tester.pressed.insert(raw);
                        } else {
                            tester.pressed.remove(&raw);
                        }
                    }
                    return 1;
                }

                let vk = normalize(info.vk_code as u16);

                // 1) 捕获模式：吞掉这次按键并回报给 UI。
                if let Ok(mut slot) = state.capture.lock() {
                    if slot.is_some() {
                        let sender = if down { slot.take() } else { None };
                        drop(slot);
                        if let Some(sender) = sender {
                            let _ = sender.send(vk);
                        }
                        return 1;
                    }
                }

                // 2) 暂停：原样放行。
                if !state.paused.load(Ordering::Relaxed) {
                    if let Ok(mut shared) = state.shared.lock() {
                        if down {
                            // 已按住（自动重复）：继续重复注入目标键。
                            if let Some(target) = shared.held.get(&vk).copied() {
                                drop(shared);
                                send_key(target, true);
                                return 1;
                            }
                            if let Some(target) = shared.mappings.get(&vk).copied() {
                                shared.held.insert(vk, target);
                                drop(shared);
                                send_key(target, true);
                                return 1;
                            }
                        } else if let Some(target) = shared.held.remove(&vk) {
                            drop(shared);
                            send_key(target, false);
                            return 1;
                        }
                    }
                }
            }
        }
    }

    unsafe { CallNextHookEx(0, code, wparam, lparam) }
}

fn send_key(vk: u16, down: bool) {
    let mut flags = 0u32;
    if !down {
        flags |= KEYEVENTF_KEYUP;
    }
    if is_extended(vk) {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }

    let input = Input {
        kind: INPUT_KEYBOARD,
        data: InputUnion {
            ki: KbdInput {
                w_vk: vk,
                w_scan: 0,
                flags,
                time: 0,
                dw_extra_info: 0,
            },
        },
    };

    unsafe {
        SendInput(1, &input, std::mem::size_of::<Input>() as i32);
    }
}

/// 方向键、翻页、Insert/Delete、小键盘除号、右 Ctrl/Alt、媒体键等属于扩展键。
fn is_extended(vk: u16) -> bool {
    matches!(
        vk,
        0x21..=0x28 | 0x2C | 0x2D | 0x2E | 0x5B | 0x5C | 0x5D | 0x6F | 0x90 | 0x91 | 0xA3 | 0xA5
            | 0xAD..=0xB7
    )
}
