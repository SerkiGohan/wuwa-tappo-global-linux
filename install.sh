#!/usr/bin/env bash
# Set up Wuwa Tappo Global for the current user: read access to input devices
# (global key/click counting), an app menu entry and the login link handler.
#   ./install.sh              install
#   ./install.sh --uninstall  remove menu entry and link handler
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
USER_NAME="${SUDO_USER:-$USER}"
APP_ID="wuwa-tappo-global"
APPS="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
DESKTOP_FILE="$APPS/$APP_ID.desktop"
ICON_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor/256x256/apps"
ICON_FILE="$ICON_DIR/$APP_ID.png"

if [[ "${1:-}" == "--uninstall" ]]; then
  rm -f "$DESKTOP_FILE" "$ICON_FILE"
  update-desktop-database "$APPS" >/dev/null 2>&1 || true
  "$DIR/install-url-handler.sh" --uninstall
  echo "[ok] Menu entry removed. App data (login, click counts, logs) stays in"
  echo "     ~/.local/share/dev.wuwatappo.global.linux/ and can be deleted by hand."
  exit 0
fi

if [[ ! -x "$DIR/bin/wuwatappo-linux" && ! -x "$DIR/src-tauri/target/release/wuwatappo-linux" ]]; then
  echo "Binary not found. Build it first: ./build.sh /path/to/WuwaTappo-global_<version>_x64-setup.exe" >&2
  exit 1
fi
chmod +x "$DIR/run.sh" "$DIR/install-url-handler.sh"

if ! getent group input >/dev/null 2>&1; then
  echo "[..] Creating the input group"
  sudo groupadd --system input
fi
if getent group input | awk -F: '{print $4}' | tr ',' '\n' | grep -qx "$USER_NAME"; then
  echo "[ok] $USER_NAME is already in the input group"
else
  echo "[..] Adding $USER_NAME to the input group (needed for global key/click counting)"
  echo "     Note: members of this group can read all keyboard and mouse input."
  sudo usermod -aG input "$USER_NAME"
  echo "[ok] Done. ./run.sh works right away; other launchers after logging in again."
fi

mkdir -p "$APPS" "$ICON_DIR"
ICON_LINE=""
for candidate in "$DIR/src-tauri/icons/128x128@2x.png" "$DIR/src-tauri/icons/128x128.png" "$DIR/frontend/app-icon.png"; do
  if [[ -f "$candidate" ]]; then
    cp "$candidate" "$ICON_FILE"
    ICON_LINE="Icon=$ICON_FILE"
    break
  fi
done

cat > "$DESKTOP_FILE" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Wuwa Tappo Global
Comment=Unofficial Linux port of the Wuwa Tappo desktop pet (Global server)
Exec="$DIR/run.sh"
Path=$DIR
$ICON_LINE
Terminal=false
Categories=Utility;
Keywords=wuwa;wuthering waves;desktop pet;tappo;
DESKTOP
chmod +x "$DESKTOP_FILE"
update-desktop-database "$APPS" >/dev/null 2>&1 || true
echo "[ok] Menu entry \"Wuwa Tappo Global\" created"

"$DIR/install-url-handler.sh"

echo
echo "Installed. Start it with:"
echo "  $DIR/run.sh"
echo "or search for \"Wuwa Tappo Global\" in your app menu."
echo "Remove the menu entry and link handler: $DIR/install.sh --uninstall"
