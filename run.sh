#!/usr/bin/env bash
# Launch Wuwa Tappo. The binary picks the display backend itself (native
# Wayland layer-shell overlay where supported, X11/XWayland otherwise) and sets
# the transparent-overlay defaults; this script adds the input group and a
# single-instance lock. Force a backend with e.g. GDK_BACKEND=x11 ./run.sh
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN="$DIR/bin/wuwatappo-linux"
if [[ ! -x "$BIN" ]]; then
  BIN="$DIR/src-tauri/target/release/wuwatappo-linux"
fi
if [[ ! -x "$BIN" ]]; then
  echo "wuwa-tappo: binary not found. Build it first: ./build.sh /path/to/WuwaTappo-global_<version>_x64-setup.exe" >&2
  exit 1
fi

export WEBKIT_FORCE_COMPOSITING_MODE=1

LOCK="${XDG_RUNTIME_DIR:-/tmp}/wuwa-tappo-linux.lock"
exec 9>"$LOCK"
if ! flock -n 9; then
  echo "Wuwa Tappo is already running."
  exit 0
fi

cd "$DIR"
# Reading /dev/input/event* needs the input group. newgrp applies the group to
# this process without requiring a full logout when the user was just added.
# Use `id -nG` without a user argument: `id -nG "$USER"` queries /etc/group, so
# it reports input even when the current session has not inherited it yet.
if ! id -nG | tr ' ' '\n' | grep -qx input \
  && command -v newgrp >/dev/null 2>&1 \
  && getent group input >/dev/null 2>&1; then
  STARTED="${LOCK}.started"
  rm -f "$STARTED"
  if newgrp input -c "cd \"$DIR\" && : > \"$STARTED\" && WEBKIT_FORCE_COMPOSITING_MODE=1 exec \"$BIN\" 9>&-"; then
    rm -f "$STARTED"
    exit 0
  else
    status=$?
    # A marker means newgrp reached the app; a non-zero status is the app's
    # own exit status. Do not start a second instance without input in that
    # case, or the lock stays held after a crash or forced quit.
    if [[ -f "$STARTED" ]]; then
      rm -f "$STARTED"
      exit "$status"
    fi
  fi
  rm -f "$STARTED"
  echo "wuwa-tappo: could not apply the input group via newgrp; starting without it." >&2
  echo "If key/click counting does not work, log out and back in, or run 'newgrp input' first." >&2
fi

# Keep this shell alive as the lock owner and close the lock fd in the app.
# Exec'ing the binary leaks fd 9 into WebKit's child processes, which can keep
# the flock held after the app exits and make the next launch report "already
# running".
"$BIN" 9>&-
