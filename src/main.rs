#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod autostart;
mod hook;
mod keys;
mod log;
mod scanmap;
mod settings;
mod ui;
mod win32;

fn main() {
    log::init();
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    log::write(&format!(
        "start args={arguments:?} exe={}",
        std::env::current_exe()
            .map(|path| path.display().to_string())
            .unwrap_or_default()
    ));

    // UAC 提权后的子进程：执行系统级映射的写入/删除，弹框报告结果后直接退出，不打开主窗口。
    if arguments.iter().any(|argument| argument == "--apply-scanmap") {
        run_scanmap_command(true);
        return;
    }
    if arguments.iter().any(|argument| argument == "--remove-scanmap") {
        run_scanmap_command(false);
        return;
    }

    let minimal = arguments.iter().any(|argument| argument == "--minimal");
    log::write(if minimal {
        "run_component: minimal"
    } else {
        "run_component: main"
    });

    let result = if minimal {
        windows_reactor::App::run_component::<ui::MinimalApp>(())
    } else {
        windows_reactor::App::run_component::<ui::KeyRemapperApp>(())
    };

    match &result {
        Ok(()) => log::write("run_component: Ok"),
        Err(error) => log::write(&format!("run_component: Err {error}")),
    }

    if let Err(error) = result {
        win32::message_box(&format!("启动失败：{error}"), "键盘映射");
    }
}

fn run_scanmap_command(apply: bool) {
    log::write(&format!("scanmap command: apply={apply}"));

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

    log::write(&format!("scanmap command result: {message}"));
    win32::message_box(&message, "键盘映射");
}
