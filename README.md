# 键盘映射

用 Rust 写的 Windows 按键重映射工具，界面基于 [windows-reactor](https://crates.io/crates/windows-reactor)——
微软 `windows-rs` 仓库里的声明式 WinUI 3 库。单 exe、无 .NET 依赖。

提供两种互不依赖的映射方式：

| 模式 | 原理 | 生效条件 | 适合场景 |
| --- | --- | --- | --- |
| **即时映射** | 全局低级键盘钩子（`WH_KEYBOARD_LL`）+ `SendInput` 注入 | 程序运行期间始终生效，开关即开即用 | 临时改键、随时切换 |
| **系统级映射** | 写入注册表 `Scancode Map`（键盘驱动层） | 写入后**重启电脑**生效；之后**无需运行本软件**、无需开机自启，永久生效 | 一劳永逸的固定改键 |

## 功能

- 单键 → 单键映射（字母、数字、F 键、小键盘、方向键、媒体键、修饰键、Esc 等）
- 映射列表管理：添加 / 删除，保存后即时生效
- 「启用映射」总开关；「开机自启」（写 `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`）
- 系统级映射：一键写入 / 恢复默认；缺管理员权限时自动弹 UAC 提权重启自身完成操作
- 配置持久化：`%LOCALAPPDATA%\KeyRemapper\settings.json`
- 按键捕获：弹窗期间暂停映射，用钩子捕获下一个物理按键，因此 Win 键、媒体键这类界面收不到的键也能映射

## 技术栈

- **UI**：`windows-reactor` 0.100 —— `Component` 持有状态、处理类型化消息、返回 `View`，
  由 Reactor 把视图树协调到原生控件；窗口标题、Mica 背景、窗口尺寸、图标都通过 `ViewContext::window_*` 声明。
- **系统能力**：手写的最小 Win32 FFI（`src/win32.rs`），只声明用到的 20 余个 API，
  不引入 `windows` / `windows-sys`，避免与 Reactor 内部的 `windows-core` 版本耦合。
- **配置**：`serde` + `serde_json`。

## 目录结构

```
src/
  main.rs       入口；处理 --apply-scanmap / --remove-scanmap 提权分支
  ui.rs         WinUI 3 界面（windows-reactor 组件：状态、消息、视图）
  hook.rs       WH_KEYBOARD_LL 钩子线程、按键注入、按键捕获
  scanmap.rs    系统级映射（Scancode Map 注册表读写、UAC 提权重启）
  autostart.rs  开机自启（HKCU Run）
  settings.rs   配置模型与持久化
  keys.rs       虚拟键码归一化与中文名
  log.rs        启动日志与 panic 记录
  win32.rs      最小 Win32 FFI 声明
assets/
  app.ico       窗口图标（源码里以 base64 保存，CI 打包时还原）
```

## 构建

需要 Windows + Rust 1.95 或更高版本：

```
cargo build --release
```

产物：`target\release\KeyRemapper.exe`。

推送到 `main` 后 GitHub Actions 自动构建；在 Actions 页面手动触发（Run workflow）时，
可选择同时创建 Release 并上传 `KeyRemapper-win-x64.zip`。

## 运行要求

- Windows 10 1809 及以上 / Windows 11
- 已安装 **Microsoft Windows App Runtime 2.4 或更高版本**（WinUI 3 运行时，Reactor 启动时会解析并加载该框架包）。
  未安装时程序会提示并给出下载入口：<https://learn.microsoft.com/windows/apps/windows-app-sdk/downloads>
- 发布产物只有一个 exe + `Assets\app.ico`

## 使用

**即时映射**

1. 启动程序 → 点「添加映射」→ 点第一个按钮按“要被替换的键”，点第二个按钮按“想要的键” → 确定
2. 打开「启用映射」开关即可；保持程序运行（可最小化），关闭软件即失效

**系统级映射**

1. 先添加好映射规则
2. 展开「系统级映射」→ 点「写入系统映射…」（会弹出 UAC，选“是”）
3. **重启电脑**后永久生效：此后不开机自启、不运行本软件、任何界面下都保持映射
4. 想还原：点「恢复系统默认」→ 再重启电脑即可

## 已知限制

- **即时映射**：注入的按键进不了管理员权限的程序（需以管理员身份运行本软件）；
  不支持组合键与鼠标键；左右 Ctrl/Shift/Alt 视为同一个键；部分带反作弊的游戏可能忽略 `SendInput` 注入。
- **系统级映射**：写入/还原各需重启一次电脑；左右修饰键只能映射左键（扫描码层决定）；
  仅支持有物理扫描码的按键。
- 两种模式可并存：即时映射收到的是系统级映射**之后**的按键。

## 故障排查

- 启动日志：`%LOCALAPPDATA%\KeyRemapper\startup.log`，记录启动各阶段与 panic 信息；
  程序打不开时先看这个文件。
- 只想确认运行时是否正常：运行 `KeyRemapper.exe --minimal`，它只打开一个最小窗口。
- 若提示缺少 Windows App Runtime，请安装 2.4 或更高版本后再启动。

## 许可

MIT。
