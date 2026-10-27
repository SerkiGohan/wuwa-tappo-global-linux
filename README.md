# Wuwa Tappo CN · Linux 社区移植版

[![CI](https://github.com/observer130/wuwa-tappo-linux/actions/workflows/ci.yml/badge.svg)](https://github.com/observer130/wuwa-tappo-linux/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/code-MIT-blue.svg)](LICENSE)
[![Official UI](https://img.shields.io/badge/official%20UI-1.0.1-c8a2ff.svg)](upstream-cn.json)

把《鸣潮》官方桌宠 **Wuwa Tappo 国服 1.0.1** 的完整官方 Web 界面带到 Linux。界面、角色、字体与在线协议保持官方原样，原生后端使用 **Rust + Tauri 2 + WebKitGTK** 重建，不需要 Wine、Windows 或虚拟机。

> 这是社区非官方移植，不是库洛官方 Linux 客户端。登录、抽卡、联机等在线功能仍连接 Kuro 官方服务，请使用自己的国服账号。官方美术与素材版权归 **KURO GAMES / Irox Games**，不适用本仓库的 MIT 许可，详见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。

## 功能

- **官方桌宠主界面**：透明、无边框、全屏覆盖层，保留官方角色、宝箱、点击计数、设置入口与多语言文案。
- **官方多窗口**：背包、图鉴、合成、联机、设置，登录窗口，关于、OSS 声明、更新/维护提示，以及 Toast。
- **全局键盘与鼠标**：基于 `evdev` 读取 `/dev/input/event*`，统计打字与点击；支持官方快捷键、点击穿透和按角色区域命中（联机时区分不同桌宠）。
- **原生 WebSocket 代理**：联机房间走 Rust `tokio-tungstenite`，绕开 WebKitGTK 对跨域 WebSocket 的限制，同时只允许官方国服端点。
- **多显示器与窗口几何**：工作区安全区域、任务栏 insets、显示器切换、窗口折叠、鼠标穿透命中测试。
- **本地数据**：点击计数原子写入；日志做脱敏和滚动；不内置任何账号、Cookie 或遥测上传。
- **开机自启**：复用官方设置页的“开机启动”开关（`tauri-plugin-autostart`）。

## 系统要求

- **x86_64 Linux**，X11 或 XWayland（纯 Wayland 尚未验证，见[已知限制](#已知限制)）。
- **WebKitGTK 4.1**、GTK 3、`libsoup-3.0`。
- 预编译发布包在 Ubuntu 24.04 环境构建，要求 **glibc ≥ 2.39**（Ubuntu 24.04+、Fedora 40+、Arch 等）。更旧的发行版请从源码构建。
- 读取全局键盘/鼠标需要当前用户属于 **`input` 组**。

各发行版安装运行依赖：

```bash
# Arch Linux
sudo pacman -S webkit2gtk-4.1 gtk3 libappindicator-gtk3

# Ubuntu 24.04 / Debian 13
sudo apt install libwebkit2gtk-4.1-0 libgtk-3-0 libayatana-appindicator3-1

# Fedora
sudo dnf install webkit2gtk4.1 gtk3 libappindicator-gtk3
```

## 快速开始

### 使用发布包

从 [Releases](https://github.com/observer130/wuwa-tappo-linux/releases) 下载 `wuwa-tappo-linux-<version>-x86_64.tar.gz`，校验后解压：

```bash
sha256sum -c wuwa-tappo-linux-1.0.1-x86_64.tar.gz.sha256  # 可选
tar -xzf wuwa-tappo-linux-1.0.1-x86_64.tar.gz
cd wuwa-tappo-linux-1.0.1-x86_64
./install.sh   # 加入 input 组、注册桌面菜单项
./run.sh       # 启动
```

之后也可以在应用菜单搜索 **Wuwa Tappo CN Linux**。

> **推荐通过 `run.sh` 启动。** 它会应用 `input` 组、单实例锁，并设置 `GDK_BACKEND=x11` 与 `WEBKIT_FORCE_COMPOSITING_MODE=1`。二进制本身也会在未显式指定时补齐这两个变量，但 `run.sh` 仍是经过验证的启动方式。
>
> 刚加入 `input` 组时，当前会话可用 `./run.sh`（内部通过 `newgrp` 生效）；重新登录后所有启动方式都生效。

### 从源码构建

需要 Node.js 20+、Rust 1.97.1、7-Zip，以及 WebKitGTK 4.1 开发包。官方前端需要从 Windows 安装包中静态提取，后再嵌入二进制：

```bash
npm ci
npm run upstream:update          # 下载并校验官方 1.0.1 安装包，提取 frontend/
npm run icons
npm run prepare:frontend
cargo build --release --manifest-path src-tauri/Cargo.toml
mkdir -p bin && cp src-tauri/target/release/wuwatappo-linux bin/
npm run package
```

完整步骤、手动提取、deb 打包和中国大陆镜像配置见 [docs/building.md](docs/building.md)。

### 登录

1. 打开设置入口，选择“通行证登录”或“游戏 UID 登录”。
2. 使用自己的国服账号完成登录。登录态由 WebKitGTK 的 Cookie/LocalStorage 保存，不会上传到本仓库。
3. 未登录时部分面板会停留在加载态，登录后由官方前端恢复。

## 快捷键

以下快捷键由官方前端注册（需窗口获得焦点/输入监听可用）：

| 按键 | 行为 |
| --- | --- |
| `F7` | 切换游戏模式（临时禁用与桌宠的交互） |
| `F8` | 在多显示器之间切换主窗口 |
| `F9` | 显示 / 隐藏桌宠 |

界面内还可点击右下角宝箱领取奖励、点击设置按钮打开背包 / 图鉴 / 合成 / 联机 / 设置。

## 数据与隐私

- 应用数据：`~/.local/share/dev.w33d.wuwatappo.cn.linux/`
  - `counters.json`：按键与点击计数（按账号/环境区分）。
  - `logs/app.log`：脱敏后的运行日志，超过 2 MiB 自动滚动。
  - WebKit 的 Cookie / LocalStorage 也位于该应用数据目录下。
- 日志会对 URL 查询参数、token、Cookie、Authorization 等做脱敏；不会主动上传。
- 重置全部本地状态：先退出应用，再删除上述目录。这会同时清除登录态与点击计数。

## 工作原理

官方前端（Vue）被打包进二进制，由 Tauri 在 WebKitGTK 中加载；Rust 后端负责窗口、全局输入、原生 WebSocket 代理、计数器与日志。启动时向每个窗口注入两个脚本：

- `scripts/platform.js`：错误上报脱敏、输入状态提示、原生输入事件抑制。
- `scripts/websocket.js`：仅对官方 Socket.IO 端点把 `WebSocket` 替换为原生桥接。

前端补丁在 `scripts/frontend-patches.mjs` 中集中维护。补丁锚点一旦与新版官方前端不匹配，构建会直接失败，避免产出半损坏的包。

更完整的进程、窗口、输入与 IPC 说明见 [docs/architecture.md](docs/architecture.md)。

## 目录结构

```
src-tauri/                 Rust 原生后端
  src/lib.rs               应用入口与命令分发
  src/windows.rs           主窗口/面板/登录/关于等窗口与命中区域
  src/input.rs             evdev 全局键鼠监听
  src/socket.rs            官方 WebSocket 原生代理
  src/storage.rs           点击计数持久化
  src/system.rs            日志、诊断、自启动
  src/menu.rs              原生菜单占位（Linux 无菜单栏）
  tauri.conf.json          Tauri 配置与 deb 元数据
scripts/                   提取、补丁、上游同步与打包脚本
tests/                     Node 契约测试与工具测试
docs/                      架构、构建、故障排查
upstream-cn.json           官方版本、下载地址、SHA-256 与签名锁定
```

## 已知限制

- **没有原生菜单栏/托盘**：macOS 版使用 NSMenu；Linux 版保留官方界面内设置与快捷键。`menu.rs` 是明确的空实现。
- **没有内置更新器**：官方更新清单只提供 Windows 包；请从本仓库 Releases 更新。
- **纯 Wayland 未验证**：主窗口通过 XWayland 运行。GNOME Wayland 下若输入焦点异常，请确认使用 `run.sh`（`GDK_BACKEND=x11`）。
- **多显示器混合缩放**：单显示器的 1.25/1.5/2.0 分数缩放已按 WebKit 实际上报的 DPR 适配；跨显示器不同缩放切换后，面板可能需要重新打开或重新居中。
- **登录态保存**：官方前端把 `infoToken` 写在 Cookie 中，但 WebKitGTK 不向 `tauri://` 来源暴露 JavaScript Cookie；构建时已将 Cookie 读写镜像到同源 LocalStorage，使登录态可在重启后恢复。不使用系统密钥环，token 以明文保存在应用数据目录。
- **仅验证国服 1.0.1**：国际服或后续版本需要重新提取资源、更新补丁锚点并重新生成契约。
- **预编译包 glibc 要求**：在 Ubuntu 24.04 构建，glibc < 2.39 的系统请自行编译。

更多问题见 [docs/troubleshooting.md](docs/troubleshooting.md)。

## 参与贡献

欢迎提交发行版适配、Wayland 修复、文档与测试。开始前请阅读 [CONTRIBUTING.md](CONTRIBUTING.md)。请不要在 Issue 或 PR 中附带账号、Cookie、token 或未脱敏日志。

## 致谢与许可

- 移植思路、资源提取脚本与 Rust 后端大量基于 [Last-emo-boy/wuwa-tappo-mac](https://github.com/Last-emo-boy/wuwa-tappo-mac)（MIT）。
- 官方界面、角色、美术、音频、字体等资源版权归 **KURO GAMES** 与 **Irox Games** 所有，不因随包分发而转为 MIT；仅用于个人桌面美化。
- 本仓库原创的 Linux 适配代码采用 MIT 许可；详见 [LICENSE](LICENSE) 与 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。
