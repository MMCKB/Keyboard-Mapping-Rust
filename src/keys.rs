//! 虚拟键码的归一化与中文显示名。

/// 左右 Ctrl / Shift / Alt 归一化为同一个键，便于映射与命名。
pub fn normalize(vk: u16) -> u16 {
    match vk {
        0xA0 | 0xA1 => 0x10, // Shift
        0xA2 | 0xA3 => 0x11, // Ctrl
        0xA4 | 0xA5 => 0x12, // Alt
        _ => vk,
    }
}

/// 返回按键的中文（或通用）名称。
pub fn name(vk: u16) -> String {
    let vk = normalize(vk);

    if (0x30..=0x39).contains(&vk) || (0x41..=0x5A).contains(&vk) {
        return char::from(vk as u8).to_string();
    }
    if (0x70..=0x87).contains(&vk) {
        return format!("F{}", vk - 0x6F);
    }
    if (0x60..=0x69).contains(&vk) {
        return format!("小键盘{}", vk - 0x60);
    }

    let text = match vk {
        0x08 => "Backspace",
        0x09 => "Tab",
        0x0D => "回车",
        0x10 => "Shift",
        0x11 => "Ctrl",
        0x12 => "Alt",
        0x13 => "Pause",
        0x14 => "大写锁定",
        0x1B => "Esc",
        0x20 => "空格",
        0x21 => "Page Up",
        0x22 => "Page Down",
        0x23 => "End",
        0x24 => "Home",
        0x25 => "←",
        0x26 => "↑",
        0x27 => "→",
        0x28 => "↓",
        0x2C => "Print Screen",
        0x2D => "Insert",
        0x2E => "Delete",
        0x5B => "左 Win",
        0x5C => "右 Win",
        0x5D => "菜单键",
        0x6A => "小键盘*",
        0x6B => "小键盘+",
        0x6C => "小键盘回车",
        0x6D => "小键盘-",
        0x6E => "小键盘.",
        0x6F => "小键盘/",
        0x90 => "数字锁定",
        0x91 => "滚动锁定",
        0xA6 => "浏览器后退",
        0xA7 => "浏览器前进",
        0xA8 => "浏览器刷新",
        0xA9 => "浏览器停止",
        0xAA => "浏览器搜索",
        0xAB => "浏览器收藏",
        0xAC => "浏览器主页",
        0xAD => "静音",
        0xAE => "音量-",
        0xAF => "音量+",
        0xB0 => "下一曲",
        0xB1 => "上一曲",
        0xB2 => "停止播放",
        0xB3 => "播放/暂停",
        0xB4 => "邮件",
        0xB5 => "媒体播放器",
        0xB6 => "我的电脑",
        0xBA => ";",
        0xBB => "=",
        0xBC => ",",
        0xBD => "-",
        0xBE => ".",
        0xBF => "/",
        0xC0 => "`",
        0xDB => "[",
        0xDC => "\\",
        0xDD => "]",
        0xDE => "'",
        _ => return format!("VK_0x{vk:02X}"),
    };

    text.to_string()
}
