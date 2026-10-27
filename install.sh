#!/usr/bin/env bash
# Register Wuwa Tappo in the desktop menu and grant read access to input devices.
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
USER_NAME="${SUDO_USER:-$USER}"
APP_ID="wuwa-tappo-linux"
DESKTOP_FILE="$HOME/.local/share/applications/$APP_ID.desktop"
ICON_DIR="$HOME/.local/share/icons/hicolor/256x256/apps"
ICON_FILE="$ICON_DIR/$APP_ID.png"

if [[ "${1:-}" == "--uninstall" ]]; then
  rm -f "$DESKTOP_FILE" "$ICON_FILE"
  update-desktop-database "$HOME/.local/share/applications" >/dev/null 2>&1 || true
  echo "已移除桌面菜单项。应用数据（登录状态、点击计数、日志）仍在用户目录中，可手动删除。"
  exit 0
fi

if [[ ! -x "$DIR/run.sh" || ! -x "$DIR/bin/wuwatappo-linux" ]] \
  && [[ ! -x "$DIR/src-tauri/target/release/wuwatappo-linux" ]]; then
  echo "未找到可执行文件。请先下载发布包，或运行 'npm run package' 构建。" >&2
  exit 1
fi
chmod +x "$DIR/run.sh" "$DIR/bin/wuwatappo-linux" 2>/dev/null || true

if ! getent group input >/dev/null 2>&1; then
  echo "[..] 创建 input 组"
  sudo groupadd --system input
fi

if getent group input | awk -F: '{print $4}' | tr ',' '\n' | grep -qx "$USER_NAME"; then
  echo "[ok] $USER_NAME 已在 input 组中"
else
  echo "[..] 将 $USER_NAME 加入 input 组（读取全局键盘/鼠标需要）"
  sudo usermod -aG input "$USER_NAME"
  echo "[ok] 已加入；当前会话可直接用 ./run.sh 启动，重新登录后所有启动方式生效"
fi

mkdir -p "$HOME/.local/share/applications" "$ICON_DIR"
SOURCE_ICON=""
for candidate in "$DIR/assets/icon.png" "$DIR/src-tauri/icons/128x128.png" "$DIR/frontend/app-icon.png"; do
  if [[ -f "$candidate" ]]; then SOURCE_ICON="$candidate"; break; fi
done
ICON_LINE=""
if [[ -n "$SOURCE_ICON" ]]; then
  cp "$SOURCE_ICON" "$ICON_FILE"
  ICON_LINE="Icon=$ICON_FILE"
fi

cat > "$DESKTOP_FILE" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Wuwa Tappo CN Linux
Comment=Unofficial Linux port of the Wuwa Tappo CN desktop pet
Exec="$DIR/run.sh"
Path=$DIR
$ICON_LINE
Terminal=false
Categories=Utility;
Keywords=wuwa;wuthering waves;desktop pet;
StartupWMClass=wuwatappo-linux
DESKTOP
chmod +x "$DESKTOP_FILE"
update-desktop-database "$HOME/.local/share/applications" >/dev/null 2>&1 || true

echo
echo "安装完成。启动："
echo "  $DIR/run.sh"
echo "或在应用菜单中搜索 “Wuwa Tappo CN Linux”。"
echo "卸载菜单项：$DIR/install.sh --uninstall"
