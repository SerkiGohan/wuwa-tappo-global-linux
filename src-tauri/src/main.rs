fn main() {
    // Hilfsmodus fuer die Backend-Wahl unten: Laeuft in einem eigenen Prozess,
    // initialisiert GTK auf Wayland und meldet per Exit-Code, ob der Compositor
    // wlr-layer-shell kann (Hyprland, Sway, KDE ja, GNOME nein).
    if std::env::args().nth(1).as_deref() == Some("--probe-layer-shell") {
        std::process::exit(if wuwatappo_linux::layer_shell_supported() { 0 } else { 1 });
    }

    // GTK/WebKitGTK need these before the first window is created. Keep values
    // supplied by the user (for example an explicit backend). Otherwise pick
    // native Wayland (click-through layer-shell overlay) when the compositor
    // supports it, and X11/XWayland everywhere else.
    if std::env::var_os("GDK_BACKEND").is_none() {
        let backend = if wayland_session() && probe_layer_shell() {
            "wayland"
        } else {
            "x11"
        };
        std::env::set_var("GDK_BACKEND", backend);
    }
    if std::env::var_os("WEBKIT_FORCE_COMPOSITING_MODE").is_none() {
        std::env::set_var("WEBKIT_FORCE_COMPOSITING_MODE", "1");
    }
    wuwatappo_linux::run();
}

fn wayland_session() -> bool {
    std::env::var_os("WAYLAND_DISPLAY").is_some()
}

/// Prueft die Layer-Shell-Unterstuetzung in einem Kindprozess. Das muss
/// getrennt passieren, weil GTK sein Backend nach der Initialisierung nicht
/// mehr wechseln kann: Faellt der Test negativ aus, starten wir sauber mit X11.
fn probe_layer_shell() -> bool {
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    std::process::Command::new(exe)
        .arg("--probe-layer-shell")
        .env("GDK_BACKEND", "wayland")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}
