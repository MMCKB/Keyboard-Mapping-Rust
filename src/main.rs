#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod autostart;
mod hook;
mod keys;
mod scanmap;
mod settings;
mod ui;
mod win32;

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();

    // UAC 提权后的子进程：执行系统级映射的写入/删除，弹框报告结果后直接退出，不打开主窗口。
    if arguments.iter().any(|argument| argument == "--apply-scanmap") {
        run_scanmap_command(true);
        return;
    }
    if arguments.iter().any(|argument| argument == "--remove-scanmap") {
        run_scanmap_command(false);
        return;
    }

    if let Err(error) = windows_reactor::App::run_component::<ui::KeyRemapperApp>(()) {
        win32::message_box(&format!("启动失败：{error}"), "键盘映射");
    }
}

fn run_scanmap_command(apply: bool) {
    let message = if apply {
        let settings = settings::load();
        if settings.mappings.is_empty() {
            "当前没有任何映射规则可写入。".to_string()
        } else {
            match scanmap::apply(&settings.mappings) {
                Ok(()) => format!(
                    "已写入 {} 条系统级映射，重启电脑后生效。\n生效后无需运行本软件，任何界面下都保持映射。",
                    settings.mappings.len()
                ),
                Err(error) => format!("操作失败：{}", error.message()),
            }
        }
    } else {
        match scanmap::remove() {
            Ok(()) => "已删除系统级映射，重启电脑后恢复系统默认的键盘行为。".to_string(),
            Err(error) => format!("操作失败：{}", error.message()),
        }
    };

    win32::message_box(&message, "键盘映射");
}
