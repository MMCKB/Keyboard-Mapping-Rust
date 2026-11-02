//! 键盘测试用的 104 键布局渲染。
//!
//! 纯静态布局，尺寸由外部传入的 `unit`（1u 的边长）决定，窗口变大时整块键盘跟着放大。
//! `pressed` 是物理按下的键（原始虚拟键码），`outputs` 是经过映射之后实际输出的键：
//! 输出键实心高亮，被映射走的物理键用描边标出。

use windows_reactor::*;

const GAP: f64 = 2.0;

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

/// 1u 对应的宽度：主键区 15u + 间距 8 + 导航区 3u + 间距 8 + 小键盘 4u。
pub fn width_for(unit: f64) -> f64 {
    unit * 22.0 + 16.0
}

/// 生成整块键盘。
pub fn keyboard(pressed: &[u16], outputs: &[u16], unit: f64) -> View {
    let main = StackPanel::new().spacing(GAP).children((
        row(ROW_F, pressed, outputs, unit),
        row(ROW_NUMBER, pressed, outputs, unit),
        row(ROW_TAB, pressed, outputs, unit),
        row(ROW_CAPS, pressed, outputs, unit),
        row(ROW_SHIFT, pressed, outputs, unit),
        row(ROW_BOTTOM, pressed, outputs, unit),
    ));

    let mut nav_children: Vec<(u32, View)> = Vec::new();
    for (index, keys) in NAV_ROWS.iter().copied().enumerate() {
        nav_children.push((index as u32, row(keys, pressed, outputs, unit)));
    }
    for index in NAV_ROWS.len()..6 {
        nav_children.push((index as u32, blank_row(unit)));
    }
    let nav = StackPanel::new().spacing(GAP).keyed_children(nav_children);

    let mut numpad_children: Vec<(u32, View)> = Vec::new();
    numpad_children.push((0, blank_row(unit)));
    for (index, keys) in NUMPAD_ROWS.iter().copied().enumerate() {
        numpad_children.push((index as u32 + 1, row(keys, pressed, outputs, unit)));
    }
    let numpad = StackPanel::new().spacing(GAP).keyed_children(numpad_children);

    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(8.0)
        .children((main, nav, numpad))
}

fn blank_row(unit: f64) -> View {
    Border::new()
        .height(unit - GAP)
        .width(unit * 3.0 - GAP)
        .into()
}

fn row(keys: &[Key], pressed: &[u16], outputs: &[u16], unit: f64) -> View {
    let mut children: Vec<(u32, View)> = Vec::with_capacity(keys.len());
    for (index, key) in keys.iter().enumerate() {
        children.push((index as u32, keycap(key, pressed, outputs, unit)));
    }
    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(GAP)
        .keyed_children(children)
}

fn keycap(key: &Key, pressed: &[u16], outputs: &[u16], unit: f64) -> View {
    let is_output = key.vk != 0 && outputs.contains(&key.vk);
    let is_physical = key.vk != 0 && pressed.contains(&key.vk);

    let background = if is_output {
        ThemeBrush::Accent
    } else {
        ThemeBrush::CardBackground
    };
    // 被映射走的物理键：描边提示「按下了，但输出不是它」。
    let border_brush = if is_output || is_physical {
        ThemeBrush::Accent
    } else {
        ThemeBrush::CardStroke
    };

    let border = Border::new()
        .width(key.units * unit - GAP)
        .height(unit - GAP)
        .corner_radius(CornerRadius::uniform(3.0))
        .border_thickness(Thickness::uniform(1.0))
        .border_brush(border_brush)
        .background(background);

    if key.label.is_empty() {
        return border.into();
    }

    let text = TextBlock::new()
        .text(key.label)
        .font_size((unit * 0.34).clamp(7.0, 11.0))
        .horizontal_alignment(HorizontalAlignment::Center)
        .vertical_alignment(VerticalAlignment::Center)
        .foreground(if is_output {
            ThemeBrush::AccentText
        } else {
            ThemeBrush::PrimaryText
        });

    border.content(text)
}
