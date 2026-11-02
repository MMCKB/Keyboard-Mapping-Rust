//! 键盘测试用的 104 键布局渲染。
//!
//! 纯静态布局：每一行是一串键帽，按下的键用强调色高亮。
//! 键位按 1u = 24 DIP 排版，整块宽度 544 DIP，放进 620 宽的窗口绰绰有余。

use windows_reactor::*;

const UNIT: f64 = 24.0;
const GAP: f64 = 2.0;
const FONT: f64 = 8.0;

struct Key {
    vk: u16,
    label: &'static str,
    units: f64,
}

const fn key(vk: u16, label: &'static str, units: f64) -> Key {
    Key { vk, label, units }
}

const fn gap(units: f64) -> Key {
    Key {
        vk: 0,
        label: "",
        units,
    }
}

const ROW_F: &[Key] = &[
    key(0x1B, "Esc", 1.0),
    key(0x70, "F1", 1.0),
    key(0x71, "F2", 1.0),
    key(0x72, "F3", 1.0),
    key(0x73, "F4", 1.0),
    gap(0.25),
    key(0x74, "F5", 1.0),
    key(0x75, "F6", 1.0),
    key(0x76, "F7", 1.0),
    key(0x77, "F8", 1.0),
    gap(0.25),
    key(0x78, "F9", 1.0),
    key(0x79, "F10", 1.0),
    key(0x7A, "F11", 1.0),
    key(0x7B, "F12", 1.0),
    gap(1.5),
];

const ROW_NUMBER: &[Key] = &[
    key(0xC0, "`", 1.0),
    key(0x31, "1", 1.0),
    key(0x32, "2", 1.0),
    key(0x33, "3", 1.0),
    key(0x34, "4", 1.0),
    key(0x35, "5", 1.0),
    key(0x36, "6", 1.0),
    key(0x37, "7", 1.0),
    key(0x38, "8", 1.0),
    key(0x39, "9", 1.0),
    key(0x30, "0", 1.0),
    key(0xBD, "-", 1.0),
    key(0xBB, "=", 1.0),
    key(0x08, "⌫", 2.0),
];

const ROW_TAB: &[Key] = &[
    key(0x09, "Tab", 1.5),
    key(0x51, "Q", 1.0),
    key(0x57, "W", 1.0),
    key(0x45, "E", 1.0),
    key(0x52, "R", 1.0),
    key(0x54, "T", 1.0),
    key(0x59, "Y", 1.0),
    key(0x55, "U", 1.0),
    key(0x49, "I", 1.0),
    key(0x4F, "O", 1.0),
    key(0x50, "P", 1.0),
    key(0xDB, "[", 1.0),
    key(0xDD, "]", 1.0),
    key(0xDC, "\\", 1.5),
];

const ROW_CAPS: &[Key] = &[
    key(0x14, "Caps", 1.75),
    key(0x41, "A", 1.0),
    key(0x53, "S", 1.0),
    key(0x44, "D", 1.0),
    key(0x46, "F", 1.0),
    key(0x47, "G", 1.0),
    key(0x48, "H", 1.0),
    key(0x4A, "J", 1.0),
    key(0x4B, "K", 1.0),
    key(0x4C, "L", 1.0),
    key(0xBA, ";", 1.0),
    key(0xDE, "'", 1.0),
    key(0x0D, "Enter", 2.25),
];

const ROW_SHIFT: &[Key] = &[
    key(0xA0, "Shift", 2.25),
    key(0x5A, "Z", 1.0),
    key(0x58, "X", 1.0),
    key(0x43, "C", 1.0),
    key(0x56, "V", 1.0),
    key(0x42, "B", 1.0),
    key(0x4E, "N", 1.0),
    key(0x4D, "M", 1.0),
    key(0xBC, ",", 1.0),
    key(0xBE, ".", 1.0),
    key(0xBF, "/", 1.0),
    key(0xA1, "Shift", 2.75),
];

const ROW_BOTTOM: &[Key] = &[
    key(0xA2, "Ctrl", 1.25),
    key(0x5B, "Win", 1.25),
    key(0xA4, "Alt", 1.25),
    key(0x20, "Space", 6.25),
    key(0xA5, "Alt", 1.25),
    key(0x5C, "Win", 1.25),
    key(0x5D, "Menu", 1.25),
    key(0xA3, "Ctrl", 1.25),
];

const NAV_ROWS: &[&[Key]] = &[
    &[
        key(0x2C, "Prt", 1.0),
        key(0x91, "Scr", 1.0),
        key(0x13, "Pse", 1.0),
    ],
    &[
        key(0x2D, "Ins", 1.0),
        key(0x24, "Home", 1.0),
        key(0x21, "PgUp", 1.0),
    ],
    &[
        key(0x2E, "Del", 1.0),
        key(0x23, "End", 1.0),
        key(0x22, "PgDn", 1.0),
    ],
    &[gap(1.0), key(0x26, "↑", 1.0), gap(1.0)],
    &[
        key(0x25, "←", 1.0),
        key(0x28, "↓", 1.0),
        key(0x27, "→", 1.0),
    ],
];

const NUMPAD_ROWS: &[&[Key]] = &[
    &[
        key(0x90, "Num", 1.0),
        key(0x6F, "/", 1.0),
        key(0x6A, "*", 1.0),
        key(0x6D, "-", 1.0),
    ],
    &[
        key(0x67, "7", 1.0),
        key(0x68, "8", 1.0),
        key(0x69, "9", 1.0),
        key(0x6B, "+", 1.0),
    ],
    &[
        key(0x64, "4", 1.0),
        key(0x65, "5", 1.0),
        key(0x66, "6", 1.0),
        gap(1.0),
    ],
    &[
        key(0x61, "1", 1.0),
        key(0x62, "2", 1.0),
        key(0x63, "3", 1.0),
        key(0x0D, "⏎", 1.0),
    ],
    &[key(0x60, "0", 2.0), key(0x6E, ".", 1.0), gap(1.0)],
];

/// 生成整块键盘。`pressed` 里是当前按下的虚拟键码（原始值，左右修饰键区分）。
pub fn keyboard(pressed: &[u16]) -> View {
    let main = StackPanel::new().spacing(GAP).children((
        row(ROW_F, pressed),
        row(ROW_NUMBER, pressed),
        row(ROW_TAB, pressed),
        row(ROW_CAPS, pressed),
        row(ROW_SHIFT, pressed),
        row(ROW_BOTTOM, pressed),
    ));

    let mut nav_children: Vec<(u32, View)> = Vec::new();
    for (index, keys) in NAV_ROWS.iter().copied().enumerate() {
        nav_children.push((index as u32, row(keys, pressed)));
    }
    for index in NAV_ROWS.len()..6 {
        nav_children.push((index as u32, row(&[], pressed)));
    }
    let nav = StackPanel::new().spacing(GAP).keyed_children(nav_children);

    let mut numpad_children: Vec<(u32, View)> = Vec::new();
    numpad_children.push((0, row(&[], pressed)));
    for (index, keys) in NUMPAD_ROWS.iter().copied().enumerate() {
        numpad_children.push((index as u32 + 1, row(keys, pressed)));
    }
    let numpad = StackPanel::new().spacing(GAP).keyed_children(numpad_children);

    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(8.0)
        .children((main, nav, numpad))
}

fn row(keys: &[Key], pressed: &[u16]) -> View {
    if keys.is_empty() {
        return Border::new()
            .height(UNIT - GAP)
            .width(UNIT * 3.0 - GAP)
            .into();
    }

    let mut children: Vec<(u32, View)> = Vec::with_capacity(keys.len());
    for (index, key) in keys.iter().enumerate() {
        children.push((index as u32, keycap(key, pressed)));
    }
    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(GAP)
        .keyed_children(children)
}

fn keycap(key: &Key, pressed: &[u16]) -> View {
    let active = key.vk != 0 && pressed.contains(&key.vk);

    let border = Border::new()
        .width(key.units * UNIT - GAP)
        .height(UNIT - GAP)
        .corner_radius(CornerRadius::uniform(3.0))
        .border_thickness(Thickness::uniform(1.0))
        .border_brush(ThemeBrush::CardStroke)
        .background(if active {
            ThemeBrush::Accent
        } else {
            ThemeBrush::CardBackground
        });

    if key.label.is_empty() {
        return border.into();
    }

    let text = TextBlock::new()
        .text(key.label)
        .font_size(FONT)
        .horizontal_alignment(HorizontalAlignment::Center)
        .vertical_alignment(VerticalAlignment::Center)
        .foreground(if active {
            ThemeBrush::AccentText
        } else {
            ThemeBrush::PrimaryText
        });

    border.content(text)
}
