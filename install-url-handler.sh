#!/usr/bin/env bash
# Register the wuwa-buddy:// link handler used by the Global login.
# After logging in, the browser opens wuwa-buddy://auth/callback?... and starts
# the binary with that link. The single-instance plugin hands it to the running
# app. The handler starts the binary directly (not run.sh), because run.sh's
# lock would turn the second instance away before it can pass the link on.
#   ./install-url-handler.sh             register
#   ./install-url-handler.sh --uninstall remove
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APPS="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
FILE="$APPS/wuwa-tappo-global-url.desktop"

if [[ "${1:-}" == "--uninstall" ]]; then
  rm -f "$FILE"
  update-desktop-database "$APPS" >/dev/null 2>&1 || true
  echo "[ok] wuwa-buddy:// handler removed"
  exit 0
fi

BIN="$DIR/bin/wuwatappo-linux"
[[ -x "$BIN" ]] || BIN="$DIR/src-tauri/target/release/wuwatappo-linux"
if [[ ! -x "$BIN" ]]; then
  echo "Binary not found. Build it first: ./build.sh /path/to/installer.exe" >&2
  exit 1
fi

mkdir -p "$APPS"
cat > "$FILE" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Wuwa Tappo Global (login callback)
Exec="$BIN" %u
NoDisplay=true
MimeType=x-scheme-handler/wuwa-buddy;
DESKTOP

update-desktop-database "$APPS" >/dev/null 2>&1 || true
xdg-mime default wuwa-tappo-global-url.desktop x-scheme-handler/wuwa-buddy
echo "[ok] wuwa-buddy:// -> $(xdg-mime query default x-scheme-handler/wuwa-buddy)"
