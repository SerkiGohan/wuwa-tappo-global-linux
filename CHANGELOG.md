# Changelog

本项目从 macOS 移植版 [wuwa-tappo-mac](https://github.com/Last-emo-boy/wuwa-tappo-mac) 派生，版本号跟随官方国服客户端。

## [1.0.1-linux.1] - 2026-10-27

首个 Linux 社区版本，对应官方国服 **Wuwa Tappo 1.0.1**。

### 新增

- 官方 1.0.1 静态前端完整嵌入二进制，保留桌宠主界面、背包、图鉴、合成、联机、设置、登录、关于、更新/维护提示。
- Rust + Tauri 2 + WebKitGTK 原生后端，替换 Windows WebView2/原生窗口层。
- `evdev` 全局键盘/鼠标监听，支持点击计数、鼠标穿透、联机角色命中区域。
- `tokio-tungstenite` 原生 WebSocket 代理，仅允许官方国服端点。
- 多显示器几何、工作区 insets、面板折叠、登录窗口就绪握手。
- 点击计数原子持久化与日志脱敏/滚动。
- Linux 启动、桌面菜单安装、tar.gz 打包脚本与 GitHub Actions CI/Release/上游同步。
- 官方 1.0.1 新增的 `set_main_skip_taskbar` 与 `exit_maintenance_app` 命令适配。
- 新增内部命令 `report_webview_metrics`，由注入脚本上报 DPR 与 CSS 视口。
- 关于页标注“Linux 社区移植版”，并区分原 Windows 配置要求。

### 修复

- 适配 GTK 分数缩放：以 WebKit 实际上报的 `devicePixelRatio` 换算命中区域、原生鼠标事件、面板/登录/关于窗口尺寸与居中位置，修复 1.25/1.5 缩放下“桌宠可见但按钮点不中”。
- 任务栏/work area insets 改为 CSS 像素返回。
- 修复辅助窗口（关于/面板）命中区域与主窗口 CSS 坐标空间不一致的问题：打开关于或面板后，原生点击不再被误判为落在辅助窗口而丢弃。
- 原生鼠标事件新增 `[MOUSE]` 诊断日志（按键、payload、CSS 坐标、是否命中辅助窗口）。
- 主窗口鼠标穿透改为 XShape 输入区域（命中区域并集），不再轮询光标；修复 Wayland/XWayland 下透明覆盖层全屏穿透时 X11 指针坐标冻结、点击落到下层应用的问题。
- 全局指针位置改为通过独立 XCB 连接查询 X11 根窗口，修复后台线程中 `cursor_position()` 返回固定坐标、所有点击都落在同一点的问题。
- 命中区域同步新增诊断日志（`[WEBVIEW]`、`[HIT]`），便于反馈缩放问题。
- 修复通行证登录成功后登录态立即丢失、重启后菜单仍显示未登录的问题：WebKitGTK 不向 `tauri://` 来源暴露 JavaScript Cookie，现将 `infoToken` 等 Cookie 镜像写入同源 LocalStorage，并在读取时回退。
- 输入设备热插拔、混合 DPI、网络监视线程退出延迟等健壮性修复。

### 变更

- 项目从 `full-port/` 子目录提升为仓库根目录。
- 启动/安装脚本更名为 `run.sh` / `install.sh`；移除全部 macOS 专用代码、脚本、测试、Info.plist 与 CI，保留上游出处说明。
- 输入状态事件由 `mac-input-status` 更名为 `wuwa-input-status`；注入脚本更名为 `platform.js` / `websocket.js`。
- 前端补丁集中到 `scripts/frontend-patches.mjs`，锚点不匹配时构建失败。
- `upstream-cn.json` 更新到 1.0.1，并重新生成对应契约。

### 已知限制

- 未实现原生菜单栏/托盘与内置更新器。
- 纯 Wayland 未验证，依赖 XWayland。
- 预编译包要求 glibc ≥ 2.39。
