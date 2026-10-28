# Wuwa Tappo Global · Linux

This fork was developed with substantial help from an AI assistant (Claude), which wrote most of the code changes. All changes were tested on real hardware before release.

An unofficial community port of **Wuwa Tappo**, the official *Wuthering Waves* desktop pet, to Linux — for the **Global** server (EU, America, Asia, SEA, HMT accounts).

This is a fork of [observer130/wuwa-tappo-linux](https://github.com/observer130/wuwa-tappo-linux), which ports the CN client. The official frontend runs unchanged inside a native Rust + Tauri 2 + WebKitGTK backend — no Wine, no VM.

What this fork adds on top of the CN port:

- **Global server support:** Global API and WebSocket endpoints, and the Global browser login (`wuwa-buddy://` link handler).
- **Native Wayland overlay:** On compositors with `wlr-layer-shell` (Hyprland, Sway, KDE Plasma, …) the pet runs as a layer-shell surface with a real input region. Clicks next to the pet go to the windows below it, as they should.
- **One-command build** from the official installer, English scripts and messages.

> **Disclaimer:** This is an unofficial fan project. It is not affiliated with, endorsed or supported by KURO GAMES. All official art, characters, fonts and the frontend are © KURO GAMES / Irox Games. **They are not part of this repository**: you extract them locally from your own copy of the official Windows installer. Login and online features talk to the official Kuro servers with your own account. A future official update can break this port at any time. Use at your own risk.

## Features

Everything the official client does: the pet overlay, chests and rewards, inventory, gallery, synthesis, multiplayer rooms, settings, global key and click counting, autostart. Supported hotkeys: `F7` game mode (the pet ignores clicks so it does not get in the way; press `F7` again to leave it), `F8` switch monitor, `F9` show/hide the pet.

## Requirements

- x86_64 Linux
- **Recommended:** a Wayland compositor with `wlr-layer-shell` support, such as Hyprland, Sway or KDE Plasma. The pet then runs as a native overlay.
- Everywhere else (X11 sessions, GNOME), the app falls back to X11/XWayland automatically.
- Your user must be in the `input` group for global key and click counting. `install.sh` sets this up (see [Privacy](#privacy-and-permissions)).

Tested on CachyOS with Hyprland. Reports from other setups are very welcome.

### Build dependencies

Rust is installed through **rustup**; `rust-toolchain.toml` pins the version and rustup fetches it automatically.

```bash
# Arch / CachyOS / Manjaro
sudo pacman -S --needed base-devel rustup nodejs npm 7zip webkit2gtk-4.1 gtk3 gtk-layer-shell libappindicator-gtk3

# Fedora
sudo dnf install gcc make rustup nodejs npm 7zip webkit2gtk4.1-devel gtk3-devel gtk-layer-shell-devel libappindicator-gtk3-devel

# Ubuntu 24.04+ / Debian 13
sudo apt install build-essential rustup nodejs npm 7zip libwebkit2gtk-4.1-dev libgtk-3-dev libgtk-layer-shell-dev libayatana-appindicator3-dev
```

If rustup is freshly installed, run `rustup default stable` once.

## Installation

1. **Download the official Windows installer** (`WuwaTappo-global_<version>_x64-setup.exe`) from the [official Wuwa Tappo download page](https://download-wutheringwaves-wuwatappo.kurogames-global.com/download/en/). The installer is never executed: the build only reads the frontend files out of it.

2. **Clone and build:**

   ```bash
   git clone https://github.com/SerkiGohan/wuwa-tappo-global-linux.git wuwa-tappo-global-linux
   cd wuwa-tappo-global-linux
   ./build.sh ~/Downloads/WuwaTappo-global_<version>_x64-setup.exe
   ```

   `build.sh` checks your dependencies, extracts the frontend, applies the Linux patches and builds `bin/wuwatappo-linux`. The first build takes a few minutes.

3. **Install** (once):

   ```bash
   ./install.sh
   ```

   This adds you to the `input` group, creates an app menu entry "Wuwa Tappo Global" and registers the `wuwa-buddy://` login link handler.

4. **Start:**

   ```bash
   ./run.sh
   ```

   Or launch "Wuwa Tappo Global" from your app menu. To start it with your session, add `run.sh` to your autostart. On Hyprland, for example, use an `exec-once` entry.

## Logging in

Open the settings via the ☰ button next to the pet and choose to log in. Your browser opens the official Kuro login page. After logging in, the browser asks whether to open the `wuwa-buddy` link: allow it, and the app is logged in.

## Display backend

`run.sh` and the binary pick the backend automatically:

| Session | Mode |
|---|---|
| Wayland with `wlr-layer-shell` (Hyprland, Sway, KDE, …) | Native layer-shell overlay |
| Wayland without layer shell (GNOME) | X11 via XWayland |
| X11 | X11 |

Override it with `./run.sh --x11` or `./run.sh --wayland` (or set `GDK_BACKEND` yourself).

**Hyprland and X11 mode:** Hyprland currently ignores the input region of floating windows ([hyprwm/Hyprland#16300](https://github.com/hyprwm/Hyprland/issues/16300)). In X11 mode, the full-screen overlay therefore swallows clicks meant for native Wayland windows. Use the default Wayland mode on Hyprland.

The overlay uses the layer namespace `wuwa-tappo` if you want compositor layer rules for it.

## Quitting

Use the quit option in the app's settings, or run:

```bash
pkill -f wuwatappo-linux
```

## Updating

When Kuro releases a new version, download the new installer and run `./build.sh` with it again. The Linux patches look for exact places in the official frontend and **stop the build with an error** if those changed, instead of producing a half-working app. If that happens, please open an issue.

## Privacy and permissions

- **`input` group:** Global key and click counting reads `/dev/input/event*` directly. Members of the `input` group can read **all** keyboard and mouse input, so any program running as your user could, too. If you do not want that, skip `install.sh`'s group step; the app still works, only the counting stops.
- **App data** lives in `~/.local/share/dev.wuwatappo.global.linux/`:
  - `counters.json` holds your key and click counts.
  - `logs/app.log` is a rotating log; tokens, cookies and URL parameters are redacted.
  - The login session is stored there as well.

  Delete the folder to reset everything, including your login.
- This port contains **no telemetry of its own** and never uploads your data anywhere except to the official Kuro services the original client talks to.

## Troubleshooting

- **The browser login does not return to the app:** Run `./install-url-handler.sh` again and check `xdg-mime query default x-scheme-handler/wuwa-buddy`. It should print `wuwa-tappo-global-url.desktop`.
- **Clicks next to the pet do not reach other windows:** Check whether the overlay mode is active with `grep LAYER ~/.local/share/dev.wuwatappo.global.linux/logs/app.log`.
- **Key/click counts stay at zero:** Log out and back in after `install.sh`, so the `input` group applies.
- **Anything else:** Please open an issue with the relevant lines from `logs/app.log`, your distribution and your desktop/compositor.

## Known limitations

- No tray icon, as in the CN port.
- Tested mainly on Hyprland. Other compositors with layer shell should work but are less tested.

## Uninstall

```bash
./install.sh --uninstall
```

This removes the menu entry and the link handler. Remove app data with `rm -rf ~/.local/share/dev.wuwatappo.global.linux`.

## Credits and license

- Linux port and backend: [observer130/wuwa-tappo-linux](https://github.com/observer130/wuwa-tappo-linux).
- Global support and Wayland layer-shell overlay: this fork.
- The port's code is licensed under the **MIT License**; see [LICENSE](LICENSE).
- Official assets, characters and the frontend are © KURO GAMES / Irox Games and **not** covered by this license; see [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
