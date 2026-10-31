//! WinUI 3 界面（windows-reactor 声明式组件）。
//!
//! 组件持有全部状态，`update` 处理消息，`view` 生成视图树；
//! Reactor 负责把视图树协调到原生 WinUI 控件上。

use std::sync::Arc;

use windows_reactor::*;

use crate::hook::HookService;
use crate::scanmap::{self, ScanmapError};
use crate::settings::{self, AppSettings, KeyMapping};
use crate::{autostart, keys, log};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    From,
    To,
}

#[derive(Clone)]
pub enum Msg {
    EnableToggled(bool),
    AutoStartToggled(bool),
    AddClicked,
    Delete(u16),
    DialogClosed(ContentDialogResult),
    ArmCapture(Target),
    Captured(Target, Option<u16>),
    ApplyScanmap,
    RemoveScanmap,
}

struct Status {
    title: String,
    message: String,
    severity: InfoBarSeverity,
}

pub struct KeyRemapperApp {
    mappings: Vec<KeyMapping>,
    enabled: bool,
    autostart: bool,
    dialog_open: bool,
    from_vk: Option<u16>,
    to_vk: Option<u16>,
    capturing: Option<Target>,
    status: Option<Status>,
    scanmap_exists: bool,
    hook: Arc<HookService>,
    hook_running: bool,
}

impl Component for KeyRemapperApp {
    type Input = ();
    type Message = Msg;

    fn create(_input: &(), _context: &ComponentContext<Self>) -> Self {
        log::write("app create: begin");
        let settings = settings::load();
        let hook = HookService::global();
        hook.set_mappings(&settings.mappings);

        let mut running = false;
        if settings.enabled && !settings.mappings.is_empty() {
            running = hook.start();
        }

        let mut app = Self {
            enabled: settings.enabled,
            mappings: settings.mappings,
            autostart: autostart::is_enabled(),
            dialog_open: false,
            from_vk: None,
            to_vk: None,
            capturing: None,
            status: None,
            scanmap_exists: scanmap::exists(),
            hook,
            hook_running: running,
        };

        if app.enabled && !app.mappings.is_empty() && !app.hook_running {
            app.set_status("键盘钩子安装失败，映射未生效。", InfoBarSeverity::Error);
        }

        log::write(&format!(
            "app create: done mappings={} enabled={} hook_running={}",
            app.mappings.len(),
            app.enabled,
            app.hook_running
        ));
        app
    }

    fn update(&mut self, message: Msg, context: &ComponentContext<Self>) {
        match message {
            Msg::EnableToggled(enabled) => {
                self.enabled = enabled;
                self.apply_all();
                if enabled && self.mappings.is_empty() {
                    self.set_status(
                        "已启用映射，但还没有任何规则，请先添加。",
                        InfoBarSeverity::Informational,
                    );
                }
            }
            Msg::AutoStartToggled(enabled) => {
                self.autostart = enabled;
                if let Err(error) = autostart::set(enabled) {
                    self.set_status(
                        format!("设置开机自启失败：{error}"),
                        InfoBarSeverity::Error,
                    );
                }
            }
            Msg::AddClicked => {
                self.dialog_open = true;
                self.from_vk = None;
                self.to_vk = None;
                self.capturing = None;
                // 捕获依赖低级键盘钩子，即使当前没有规则也要先把钩子装上。
                if !self.hook.start() {
                    self.set_status("键盘钩子安装失败，无法捕获按键。", InfoBarSeverity::Error);
                }
                // 弹窗期间暂停映射，保证捕获到的是物理按键。
                self.hook.set_paused(true);
            }
            Msg::Delete(from_vk) => {
                self.mappings.retain(|mapping| mapping.from_vk != from_vk);
                self.apply_all();
            }
            Msg::DialogClosed(result) => {
                self.dialog_open = false;
                self.capturing = None;
                self.hook.clear_capture();
                self.hook.set_paused(false);
                self.refresh_hook();

                if matches!(result, ContentDialogResult::Primary) {
                    match (self.from_vk, self.to_vk) {
                        (Some(from_vk), Some(to_vk)) if from_vk == to_vk => {
                            self.set_status(
                                "原始按键和目标按键相同，无需映射。",
                                InfoBarSeverity::Error,
                            );
                        }
                        (Some(from_vk), Some(to_vk)) => {
                            self.mappings.retain(|mapping| mapping.from_vk != from_vk);
                            self.mappings.push(KeyMapping {
                                from_vk,
                                from_name: keys::name(from_vk),
                                to_vk,
                                to_name: keys::name(to_vk),
                            });
                            self.apply_all();
                            self.set_status("已添加映射，立即生效。", InfoBarSeverity::Success);
                        }
                        _ => self.set_status(
                            "请先分别捕获原始按键和目标按键。",
                            InfoBarSeverity::Error,
                        ),
                    }
                }
            }
            Msg::ArmCapture(target) => {
                self.capturing = Some(target);
                let hook = Arc::clone(&self.hook);
                let _ = context.spawn_background(move |cancel| {
                    let captured = hook.capture_next(|| cancel.is_cancelled());
                    Msg::Captured(target, captured)
                });
            }
            Msg::Captured(target, captured) => {
                if let Some(vk) = captured {
                    match target {
                        Target::From => self.from_vk = Some(vk),
                        Target::To => self.to_vk = Some(vk),
                    }
                }
                if self.capturing == Some(target) {
                    self.capturing = None;
                }
            }
            Msg::ApplyScanmap => {
                if self.mappings.is_empty() {
                    self.set_status(
                        "没有映射规则可写入，请先在上方添加映射。",
                        InfoBarSeverity::Error,
                    );
                } else {
                    match scanmap::apply(&self.mappings) {
                        Ok(()) => {
                            self.scanmap_exists = true;
                            self.set_status(
                                "系统级映射已写入，重启电脑后生效；此后不开机自启、不运行本软件也保持生效。",
                                InfoBarSeverity::Success,
                            );
                        }
                        Err(ScanmapError::NeedAdmin) => {
                            match scanmap::relaunch_elevated("--apply-scanmap") {
                                Ok(()) => self.set_status(
                                    "请在弹出的管理员确认框中选择「是」，写入完成后会弹出结果。",
                                    InfoBarSeverity::Informational,
                                ),
                                Err(error) => self.set_status(
                                    format!("未获得管理员权限，未写入。（{error}）"),
                                    InfoBarSeverity::Error,
                                ),
                            }
                        }
                        Err(error) => self.set_status(
                            format!("写入失败：{}", error.message()),
                            InfoBarSeverity::Error,
                        ),
                    }
                }
            }
            Msg::RemoveScanmap => match scanmap::remove() {
                Ok(()) => {
                    self.scanmap_exists = false;
                    self.set_status(
                        "已删除系统级映射，重启电脑后恢复系统默认的键盘行为。",
                        InfoBarSeverity::Success,
                    );
                }
                Err(ScanmapError::NeedAdmin) => match scanmap::relaunch_elevated("--remove-scanmap") {
                    Ok(()) => self.set_status(
                        "请在弹出的管理员确认框中选择「是」，删除完成后会弹出结果。",
                        InfoBarSeverity::Informational,
                    ),
                    Err(error) => self.set_status(
                        format!("未获得管理员权限，未删除。（{error}）"),
                        InfoBarSeverity::Error,
                    ),
                },
                Err(error) => self.set_status(
                    format!("删除失败：{}", error.message()),
                    InfoBarSeverity::Error,
                ),
            },
        }
    }

    fn view(&self, _input: &(), context: &mut ViewContext<Self>) -> View {
        log_view();
        context.window_title("键盘映射");

        let mut visuals = WindowVisuals::new()
            .backdrop(WindowBackdrop::Mica)
            .client_size(620.0, 700.0);
        if let Some(icon) = app_icon() {
            visuals = visuals.icon(icon);
        }
        context.window_visuals(visuals);

        // ---- 顶栏：标题 + 启用开关 ----
        let title = TextBlock::new()
            .text("键盘映射")
            .font_size(28.0)
            .font_weight(FontWeight::SEMI_BOLD);
        let subtitle = TextBlock::new()
            .text("把某个按键即时替换成另一个按键，如 Caps Lock → Esc。")
            .font_size(13.0)
            .opacity(0.75)
            .text_wrapping(TextWrapping::Wrap);
        let title_block = StackPanel::new().spacing(2.0).children((title, subtitle));

        let enable_switch = StackPanel::new()
            .orientation(Orientation::Horizontal)
            .spacing(8.0)
            .vertical_alignment(VerticalAlignment::Center)
            .children((
                TextBlock::new().text("启用映射"),
                ToggleSwitch::new()
                    .is_on(self.enabled)
                    .on_toggled(context.callback(Msg::EnableToggled))
                    .slots([
                        SlotView::new(ToggleSwitchSlot::OnContent, ""),
                        SlotView::new(ToggleSwitchSlot::OffContent, ""),
                    ]),
            ));

        let header = Border::new()
            .grid_row(0)
            .padding(Thickness::new(24.0, 20.0, 24.0, 0.0))
            .content(
                Grid::new()
                    .columns([GridLength::Star(1.0), GridLength::Auto])
                    .column_spacing(12.0)
                    .children((
                        Border::new().grid_column(0).content(title_block),
                        Border::new().grid_column(1).content(enable_switch),
                    )),
            );

        // ---- 状态 / 提示条 ----
        let mut bars: Vec<(u32, View)> = Vec::new();
        if let Some(status) = &self.status {
            bars.push((
                0,
                InfoBar::new()
                    .is_open(true)
                    .is_closable(true)
                    .severity(status.severity)
                    .title(status.title.clone())
                    .message(status.message.clone())
                    .into(),
            ));
        }
        bars.push((
            1,
            InfoBar::new()
                .is_open(true)
                .is_closable(true)
                .severity(InfoBarSeverity::Informational)
                .title("提示")
                .message("要让映射在管理员权限的程序（如任务管理器）中也生效，请以管理员身份运行本软件。")
                .into(),
        ));
        if self.scanmap_exists {
            bars.push((
                2,
                InfoBar::new()
                    .is_open(true)
                    .is_closable(true)
                    .severity(InfoBarSeverity::Warning)
                    .title("系统级映射已存在")
                    .message("重启电脑后即使不运行本软件也会保持映射；它与即时映射相互独立，注意不要对同一物理按键设置会相互干扰的规则。")
                    .into(),
            ));
        }
        let bars_area = Border::new()
            .grid_row(1)
            .padding(Thickness::new(24.0, 12.0, 24.0, 0.0))
            .content(StackPanel::new().spacing(8.0).keyed_children(bars));

        // ---- 映射列表 ----
        let list: View = if self.mappings.is_empty() {
            Border::new()
                .grid_row(2)
                .padding(Thickness::new(24.0, 12.0, 24.0, 0.0))
                .content(
                    TextBlock::new()
                        .text("还没有映射规则。点击下方「添加映射」：先按要被替换的键，再按想要的键，映射立即生效、无需重启。")
                        .text_wrapping(TextWrapping::Wrap)
                        .horizontal_alignment(HorizontalAlignment::Center)
                        .vertical_alignment(VerticalAlignment::Center)
                        .opacity(0.7),
                )
        } else {
            let mut rows: Vec<(u32, View)> = Vec::new();
            for mapping in &self.mappings {
                rows.push((mapping.from_vk as u32, mapping_row(mapping, context)));
            }
            Border::new()
                .grid_row(2)
                .padding(Thickness::new(24.0, 12.0, 24.0, 0.0))
                .content(
                    ScrollViewer::new()
                        .vertical_scroll_bar_visibility(ScrollBarVisibility::Auto)
                        .content(StackPanel::new().spacing(0.0).keyed_children(rows)),
                )
        };

        // ---- 系统级映射 ----
        let expander_header = TextBlock::new()
            .text("系统级映射（写入系统，无需运行本软件）")
            .font_weight(FontWeight::SEMI_BOLD);

        let expander_content = StackPanel::new().spacing(12.0).children((
            TextBlock::new()
                .text("写入键盘驱动层（注册表 Scancode Map），重启电脑后永久生效；之后无需开机自启、不运行任何软件也保持映射。需要管理员权限。")
                .font_size(12.0)
                .opacity(0.75)
                .text_wrapping(TextWrapping::Wrap),
            StackPanel::new()
                .orientation(Orientation::Horizontal)
                .spacing(8.0)
                .children((
                    Button::new()
                        .style(ButtonStyle::Accent)
                        .on_click(context.message(Msg::ApplyScanmap))
                        .content("写入系统映射…"),
                    Button::new()
                        .style(ButtonStyle::Subtle)
                        .on_click(context.message(Msg::RemoveScanmap))
                        .content("恢复系统默认"),
                )),
            TextBlock::new()
                .text(if self.scanmap_exists {
                    "当前状态：注册表中已写入系统级映射（若刚写入，重启电脑后生效）。"
                } else {
                    "当前状态：未写入，系统键盘行为未被修改。"
                })
                .font_size(12.0)
                .opacity(0.75)
                .text_wrapping(TextWrapping::Wrap),
            TextBlock::new()
                .text("说明：系统级映射直接改写物理按键的扫描码，一次写入、重启后永久有效；即时映射则在本软件关闭后失效。两者可并存，即时映射收到的是系统级映射之后的按键。")
                .font_size(12.0)
                .opacity(0.6)
                .text_wrapping(TextWrapping::Wrap),
        ));

        let expander = Expander::new()
            .grid_row(3)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .margin(Thickness::new(24.0, 12.0, 24.0, 0.0))
            .is_expanded(false)
            .slots([
                SlotView::new(ExpanderSlot::Header, expander_header),
                SlotView::new(ExpanderSlot::Content, expander_content),
            ]);

        // ---- 底部操作 ----
        let footer = Border::new()
            .grid_row(4)
            .padding(Thickness::new(24.0, 12.0, 24.0, 20.0))
            .content(
                Grid::new()
                    .columns([
                        GridLength::Auto,
                        GridLength::Star(1.0),
                        GridLength::Auto,
                    ])
                    .column_spacing(12.0)
                    .children((
                        Button::new()
                            .grid_column(0)
                            .style(ButtonStyle::Accent)
                            .on_click(context.message(Msg::AddClicked))
                            .content("添加映射"),
                        StackPanel::new()
                            .grid_column(2)
                            .orientation(Orientation::Horizontal)
                            .spacing(8.0)
                            .vertical_alignment(VerticalAlignment::Center)
                            .children((
                                TextBlock::new().text("开机自启"),
                                ToggleSwitch::new()
                                    .is_on(self.autostart)
                                    .on_toggled(context.callback(Msg::AutoStartToggled))
                                    .slots([
                                        SlotView::new(ToggleSwitchSlot::OnContent, ""),
                                        SlotView::new(ToggleSwitchSlot::OffContent, ""),
                                    ]),
                            )),
                    )),
            );

        // ---- 添加映射弹窗 ----
        let from_label = match (self.capturing, self.from_vk) {
            (Some(Target::From), _) => "请按下按键…".to_string(),
            (_, Some(vk)) => keys::name(vk),
            _ => "点击后按下按键".to_string(),
        };
        let to_label = match (self.capturing, self.to_vk) {
            (Some(Target::To), _) => "请按下按键…".to_string(),
            (_, Some(vk)) => keys::name(vk),
            _ => "点击后按下按键".to_string(),
        };

        let dialog_panel = StackPanel::new().spacing(6.0).children((
            TextBlock::new().text("原始按键（会被拦截的键）"),
            Button::new()
                .on_click(context.callback(move |_: ()| Msg::ArmCapture(Target::From)))
                .content(from_label),
            TextBlock::new()
                .text("目标按键（替换成的键）")
                .margin(Thickness::new(0.0, 12.0, 0.0, 0.0)),
            Button::new()
                .on_click(context.callback(move |_: ()| Msg::ArmCapture(Target::To)))
                .content(to_label),
            TextBlock::new()
                .text("点下方按钮后按下按键即可捕获（Esc、修饰键也可作为目标）；再点一次可重新捕获。当前版本一次映射一个键，不支持组合键。")
                .font_size(12.0)
                .opacity(0.75)
                .text_wrapping(TextWrapping::Wrap)
                .margin(Thickness::new(0.0, 12.0, 0.0, 0.0)),
        ));

        let dialog = ContentDialog::new()
            .title("添加按键映射")
            .primary_button_text("确定")
            .close_button_text("取消")
            .is_open(self.dialog_open)
            .on_closed(context.callback(Msg::DialogClosed))
            .content(dialog_panel);

        // 对话框必须作为普通容器的子节点参与协调：放进 fragment 会让 WinUI 抛异常直接退出。
        let children: Vec<(u32, View)> = vec![
            (0, header),
            (1, bars_area),
            (2, list),
            (3, expander),
            (4, footer),
            (5, dialog),
        ];

        Grid::new()
            .rows([
                GridLength::Auto,
                GridLength::Auto,
                GridLength::Star(1.0),
                GridLength::Auto,
                GridLength::Auto,
            ])
            .keyed_children(children)
    }
}

impl KeyRemapperApp {
    fn apply_all(&mut self) {
        settings::save(&AppSettings {
            enabled: self.enabled,
            mappings: self.mappings.clone(),
        });
        self.refresh_hook();
    }

    fn refresh_hook(&mut self) {
        self.hook.set_mappings(&self.mappings);

        if self.enabled && !self.mappings.is_empty() {
            self.hook_running = self.hook.start();
            if !self.hook_running {
                self.set_status("键盘钩子安装失败，映射未生效。", InfoBarSeverity::Error);
            }
        } else {
            self.hook.stop();
            self.hook_running = false;
        }
    }

    fn set_status(&mut self, message: impl Into<String>, severity: InfoBarSeverity) {
        let title = if matches!(severity, InfoBarSeverity::Error) {
            "出错了"
        } else {
            "提示"
        };
        self.status = Some(Status {
            title: title.to_string(),
            message: message.into(),
            severity,
        });
    }
}

fn mapping_row(mapping: &KeyMapping, context: &mut ViewContext<KeyRemapperApp>) -> View {
    let from_badge = Border::new()
        .background(ThemeBrush::CardBackground)
        .corner_radius(CornerRadius::uniform(6.0))
        .padding(Thickness::xy(10.0, 3.0))
        .content(
            TextBlock::new()
                .text(mapping.from_name.clone())
                .font_weight(FontWeight::SEMI_BOLD),
        );

    let arrow = TextBlock::new().text("→").font_size(16.0).opacity(0.6);

    let to_badge = Border::new()
        .background(ThemeBrush::Accent)
        .corner_radius(CornerRadius::uniform(6.0))
        .padding(Thickness::xy(10.0, 3.0))
        .content(
            TextBlock::new()
                .text(mapping.to_name.clone())
                .font_weight(FontWeight::SEMI_BOLD),
        );

    let from_vk = mapping.from_vk;
    let delete = Button::new()
        .style(ButtonStyle::Subtle)
        .vertical_alignment(VerticalAlignment::Center)
        .on_click(context.callback(move |_: ()| Msg::Delete(from_vk)))
        .content("删除");

    Border::new()
        .background(ThemeBrush::CardBackground)
        .border_brush(ThemeBrush::CardStroke)
        .border_thickness(Thickness::uniform(1.0))
        .corner_radius(CornerRadius::uniform(8.0))
        .padding(Thickness::new(16.0, 10.0, 16.0, 10.0))
        .margin(Thickness::new(0.0, 3.0, 0.0, 3.0))
        .content(
            Grid::new()
                .columns([GridLength::Star(1.0), GridLength::Auto])
                .column_spacing(8.0)
                .children((
                    Border::new()
                        .grid_column(0)
                        .vertical_alignment(VerticalAlignment::Center)
                        .content(
                            StackPanel::new()
                                .orientation(Orientation::Horizontal)
                                .spacing(10.0)
                                .children((from_badge, arrow, to_badge)),
                        ),
                    Border::new().grid_column(1).content(delete),
                )),
        )
}

/// 图标路径必须是 `&'static str`，这里把 exe 同级的绝对路径泄漏一次。
fn app_icon() -> Option<&'static str> {
    use std::sync::OnceLock;
    static ICON: OnceLock<Option<&'static str>> = OnceLock::new();
    *ICON.get_or_init(|| {
        let executable = std::env::current_exe().ok()?;
        let directory = executable.parent()?;
        let path = directory.join("Assets").join("app.ico");
        if !path.is_file() {
            return None;
        }
        let text = path.to_string_lossy().into_owned();
        Some(Box::leak(text.into_boxed_str()))
    })
}

fn log_view() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNT: AtomicUsize = AtomicUsize::new(0);
    let count = COUNT.fetch_add(1, Ordering::Relaxed) + 1;
    if count <= 3 || count % 100 == 0 {
        log::write(&format!("app view: publication {count}"));
    }
}

/// 诊断用最小界面：只放一个 TextBlock，用来区分「运行时环境问题」和「界面构建问题」。
pub struct MinimalApp;

impl Component for MinimalApp {
    type Input = ();
    type Message = ();

    fn create(_input: &(), _context: &ComponentContext<Self>) -> Self {
        log::write("minimal create");
        Self
    }

    fn view(&self, _input: &(), context: &mut ViewContext<Self>) -> View {
        log::write("minimal view");
        context.window_title("键盘映射（诊断）");
        context.window_visuals(WindowVisuals::new().client_size(480.0, 220.0));
        TextBlock::new()
            .text("最小界面：看到这个窗口说明 WinUI 运行时正常。")
            .into()
    }
}
