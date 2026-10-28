#!/usr/bin/env bash
# Build Wuwa Tappo Global for Linux from the official Windows installer.
#
#   ./build.sh /path/to/WuwaTappo-global_<version>_x64-setup.exe
#
# The installer is never executed: the official frontend is extracted
# statically from it and embedded into the Linux binary (bin/wuwatappo-linux).
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$DIR"

step() { printf '\n\033[1m==> %s\033[0m\n' "$*"; }
die() { printf '\033[31merror:\033[0m %s\n' "$*" >&2; exit 1; }

INSTALLER="${1:-}"
if [[ -z "$INSTALLER" || ! -f "$INSTALLER" ]]; then
  echo "Usage: ./build.sh /path/to/WuwaTappo-global_<version>_x64-setup.exe" >&2
  echo "Download the installer from the official Wuwa Tappo page first." >&2
  exit 1
fi
INSTALLER="$(realpath "$INSTALLER")"

step "Checking build dependencies"
missing=()
for tool in node npm cargo pkg-config; do
  command -v "$tool" >/dev/null 2>&1 || missing+=("$tool")
done
SEVENZIP="$(command -v 7zz || command -v 7z || true)"
[[ -n "$SEVENZIP" ]] || missing+=("7-Zip (7z or 7zz)")
if command -v pkg-config >/dev/null 2>&1; then
  for lib in webkit2gtk-4.1 gtk+-3.0 gtk-layer-shell-0 libsoup-3.0; do
    pkg-config --exists "$lib" || missing+=("$lib (development files)")
  done
fi
if ((${#missing[@]})); then
  printf '  missing: %s\n' "${missing[@]}" >&2
  cat >&2 <<'HINT'

Install them, for example:
  Arch:          sudo pacman -S --needed base-devel rustup nodejs npm 7zip webkit2gtk-4.1 gtk3 gtk-layer-shell libappindicator-gtk3
  Fedora:        sudo dnf install gcc make rustup nodejs npm 7zip webkit2gtk4.1-devel gtk3-devel gtk-layer-shell-devel libappindicator-gtk3-devel
  Ubuntu/Debian: sudo apt install build-essential rustup nodejs npm 7zip libwebkit2gtk-4.1-dev libgtk-3-dev libgtk-layer-shell-dev libayatana-appindicator3-dev
HINT
  die "missing build dependencies"
fi
# rust-toolchain.toml pins the Rust version; rustup installs it on demand.
if command -v rustup >/dev/null 2>&1; then
  rustup toolchain install >/dev/null 2>&1 || true
fi

step "Installing npm build tools"
if [[ ! -d node_modules ]]; then
  npm ci
else
  echo "  node_modules present, skipping"
fi

step "Extracting the official frontend from $(basename "$INSTALLER")"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
node scripts/extract-assets.mjs --installer "$INSTALLER" --sevenzip "$SEVENZIP" \
  --output "$TMP/frontend" > "$TMP/extract.json" \
  || die "extraction failed (is this the official Wuwa Tappo installer?)"
if ! grep -qs 'APP_REGION="GLOBAL"' "$TMP"/frontend/assets/*.js; then
  die "this is not the Global installer. For the CN version use github.com/observer130/wuwa-tappo-linux"
fi
rm -rf frontend
mv "$TMP/frontend" frontend
echo "  $(find frontend -type f | wc -l) files extracted"

step "Preparing frontend and icons"
npm run --silent icons
npm run --silent prepare:frontend

step "Building the Linux binary (the first build takes a few minutes)"
cargo build --release --manifest-path src-tauri/Cargo.toml
mkdir -p bin
cp src-tauri/target/release/wuwatappo-linux bin/

printf '\n\033[1mDone.\033[0m Next steps:\n'
echo "  ./install.sh   # input group, app menu entry, login link handler (once)"
echo "  ./run.sh       # start Wuwa Tappo"
