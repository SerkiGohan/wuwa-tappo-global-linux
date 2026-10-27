#!/usr/bin/env bash
# Launch Wuwa Tappo. The binary sets the transparent-overlay environment
# defaults itself; this script adds the input group, single-instance lock and
# X11 backend preference used by the tested configuration.
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN="$DIR/bin/wuwatappo-linux"
if [[ ! -x "$BIN" ]]; then
  BIN="$DIR/src-tauri/target/release/wuwatappo-linux"
fi
if [[ ! -x "$BIN" ]]; then
  echo "wuwa-tappo: binary not found. Download a release, or build with 'npm run package'." >&2
  exit 1
fi

export GDK_BACKEND=x11
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
  if newgrp input -c "cd \"$DIR\" && GDK_BACKEND=x11 WEBKIT_FORCE_COMPOSITING_MODE=1 exec \"$BIN\""; then
    exit 0
  fi
  echo "wuwa-tappo: 无法通过 newgrp 应用 input 组，将以当前权限启动。" >&2
  echo "如果键鼠计数不可用，请重新登录，或在终端执行 'newgrp input' 后再运行。" >&2
fi
exec "$BIN"
