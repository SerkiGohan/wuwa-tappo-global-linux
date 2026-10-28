#!/usr/bin/env bash
# Registriert das Protokoll wuwa-buddy:// fuer den Global-Login.
# Der Browser startet damit nach dem Login die App-Binary direkt (ohne run.sh,
# dessen flock-Sperre die zweite Instanz sonst abweisen wuerde). Das
# Single-Instance-Plugin reicht den Link dann an die laufende App weiter.
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN="$DIR/src-tauri/target/release/wuwatappo-linux"
if [[ -x "$DIR/bin/wuwatappo-linux" ]]; then
  BIN="$DIR/bin/wuwatappo-linux"
fi
if [[ ! -x "$BIN" ]]; then
  echo "Binary nicht gefunden - bitte zuerst bauen." >&2
  exit 1
fi

APPS="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
FILE="$APPS/wuwa-tappo-global-url.desktop"
mkdir -p "$APPS"
cat > "$FILE" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Wuwa Tappo Global (Login-Rückruf)
Exec=env GDK_BACKEND=x11 WEBKIT_FORCE_COMPOSITING_MODE=1 "$BIN" %u
NoDisplay=true
MimeType=x-scheme-handler/wuwa-buddy;
DESKTOP

update-desktop-database "$APPS" 2>/dev/null || true
xdg-mime default wuwa-tappo-global-url.desktop x-scheme-handler/wuwa-buddy
echo "wuwa-buddy:// -> $(xdg-mime query default x-scheme-handler/wuwa-buddy)"
