fn main() {
    // GTK/WebKitGTK need these before the first window is created. Keep values
    // supplied by the user (for example an explicit Wayland backend), but make a
    // direct binary launch behave like run.sh for the transparent overlay.
    if std::env::var_os("GDK_BACKEND").is_none() {
        std::env::set_var("GDK_BACKEND", "x11");
    }
    if std::env::var_os("WEBKIT_FORCE_COMPOSITING_MODE").is_none() {
        std::env::set_var("WEBKIT_FORCE_COMPOSITING_MODE", "1");
    }
    wuwatappo_linux::run();
}
