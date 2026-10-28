//! Wayland-Overlay fuer das Hauptfenster (das Pet).
//!
//! Hyprland beachtet die Eingabeflaeche schwebender Fenster nicht zuverlaessig
//! (hyprwm/Hyprland#16300) und damit auch nicht die XShape-Eingabeform eines
//! XWayland-Fensters: Das monitorgrosse Overlay schluckt dann alle Klicks.
//! Layer-Shell-Flaechen (wie Waybar oder Quickshell-Widgets) bekommen ihre
//! Eingabeflaeche dagegen korrekt beachtet.
//!
//! Laeuft GTK nativ auf Wayland, wird der Webview des Hauptfensters daher aus
//! dem Tauri-Fenster herausgeloest und in ein Layer-Shell-Fenster umgehaengt.
//! Die Klickbereiche des Pets landen als GTK-Eingabeform auf diesem Fenster.
//!
//! Alle Funktionen hier muessen auf dem GTK-Hauptthread laufen.

use gtk::prelude::*;
use gtk_layer_shell::{Edge, Layer, LayerShell};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

thread_local! {
    static OVERLAY: RefCell<Option<gtk::ApplicationWindow>> = const { RefCell::new(None) };
    /// Index des Monitors, auf den wir das Overlay zuletzt gesetzt haben.
    static MONITOR: std::cell::Cell<Option<i32>> = const { std::cell::Cell::new(None) };
    /// Zuletzt gesetzte Eingabeflaeche: (ganzes Fenster?, Rechtecke).
    static INPUT: RefCell<(bool, Vec<(i32, i32, i32, i32)>)> = const { RefCell::new((false, Vec::new())) };
}

/// Ob das Overlay aktiv ist, lesbar auch vom Eingabe-Thread (input.rs).
static ACTIVE: AtomicBool = AtomicBool::new(false);
/// Letzte Zeigerposition ueber dem Overlay in CSS-Pixeln, `None` = nicht ueber uns.
static CURSOR: Mutex<Option<(f64, f64)>> = Mutex::new(None);

/// Laeuft GTK in diesem Prozess nativ auf Wayland (nicht ueber XWayland)?
pub fn is_wayland() -> bool {
    gtk::gdk::Display::default()
        .map(|display| display.type_().name() == "GdkWaylandDisplay")
        .unwrap_or(false)
}

/// Ist das Layer-Shell-Overlay aktiv?
pub fn is_active() -> bool {
    OVERLAY.with(|overlay| overlay.borrow().is_some())
}

/// Haengt den Webview des Hauptfensters in ein Layer-Shell-Overlay um, das den
/// ganzen Monitor bedeckt. Das urspruengliche Tauri-Fenster bleibt unsichtbar.
pub fn convert(window: &tauri::WebviewWindow) -> Result<(), String> {
    if !gtk_layer_shell::is_supported() {
        return Err("Compositor unterstuetzt wlr-layer-shell nicht".into());
    }
    let old = window.gtk_window().map_err(|error| error.to_string())?;
    let app = old
        .application()
        .ok_or("GTK-Fenster hat keine Application")?;
    let vbox = window.default_vbox().map_err(|error| error.to_string())?;

    old.hide();
    old.remove(&vbox);
    // Tauri oder das Frontend duerfen das leere Originalfenster nicht wieder
    // zeigen: Ein sichtbares leeres Fenster wuerde sonst wieder Klicks schlucken.
    old.connect_show(|w| {
        let w = w.clone();
        gtk::glib::idle_add_local_once(move || w.hide());
    });

    let overlay = gtk::ApplicationWindow::new(&app);
    overlay.set_title("Wuwa Tappo Global Linux");
    overlay.set_decorated(false);
    overlay.set_app_paintable(true);
    if let Some(visual) = WidgetExt::screen(&overlay).and_then(|screen| screen.rgba_visual()) {
        overlay.set_visual(Some(&visual));
    }
    overlay.add(&vbox);

    // Muss vor dem ersten Anzeigen passieren.
    overlay.init_layer_shell();
    overlay.set_namespace("wuwa-tappo");
    // Top: ueber normalen Fenstern, aber unter Vollbild-Spielen.
    overlay.set_layer(Layer::Top);
    for edge in [Edge::Left, Edge::Right, Edge::Top, Edge::Bottom] {
        overlay.set_anchor(edge, true);
    }

    // Tauri liest Groesse und Position des Hauptfensters aus dessen
    // configure-Events. Das Originalfenster bleibt aber versteckt, also leiten
    // wir die Events des Overlays an es weiter. So stimmen innerSize(),
    // scaleFactor() & Co. im Frontend und es bekommt Resize-Events.
    let old_for_configure = old.clone();
    overlay.connect_configure_event(move |_, event| {
        let event: gtk::gdk::Event = (**event).clone();
        let _ = old_for_configure.emit_by_name::<bool>("configure-event", &[&event]);
        false
    });

    // Beim Monitorwechsel bildet gtk-layer-shell die Flaeche neu ab, und GDK
    // erzeugt dafuer eine neue Wayland-Surface, die die alte Eingabeflaeche
    // nicht uebernimmt. Nach jedem Abbilden setzen wir sie deshalb neu.
    overlay.connect_map(|window| {
        let window = window.clone();
        gtk::glib::idle_add_local_once(move || reapply_input(&window));
    });

    // Bis das Frontend seine Klickbereiche meldet: komplett durchklickbar.
    overlay.input_shape_combine_region(Some(&gtk::cairo::Region::create()));
    overlay.show_all();

    start_pointer_tracking(&overlay);
    OVERLAY.with(|slot| *slot.borrow_mut() = Some(overlay));
    ACTIVE.store(true, Ordering::Release);
    Ok(())
}

/// Zeigerposition fuer input.rs: `None` = kein Overlay aktiv (X11-Weg nutzen),
/// `Some(None)` = Zeiger ist nicht ueber dem Pet, `Some(Some((x, y)))` = Position
/// in CSS-Pixeln relativ zum Overlay.
pub fn cursor() -> Option<Option<(f64, f64)>> {
    if !ACTIVE.load(Ordering::Acquire) {
        return None;
    }
    Some(CURSOR.lock().ok().and_then(|cursor| *cursor))
}

/// Unter Wayland gibt es keine globale Zeigerposition. Wir bekommen den Zeiger
/// aber immer dann, wenn er ueber der Eingabeflaeche des Pets ist, und genau
/// dort braucht das Frontend ihn. Ausserhalb melden wir "nicht ueber uns".
fn start_pointer_tracking(overlay: &gtk::ApplicationWindow) {
    let overlay = overlay.clone();
    gtk::glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
        let position = pointer_over(&overlay);
        if let Ok(mut cursor) = CURSOR.lock() {
            *cursor = position;
        }
        gtk::glib::ControlFlow::Continue
    });
}

fn pointer_over(overlay: &gtk::ApplicationWindow) -> Option<(f64, f64)> {
    let own = overlay.window()?;
    let pointer = WidgetExt::display(overlay).default_seat()?.pointer()?;
    let (under, _, _) = pointer.window_at_position();
    if under?.toplevel() != own {
        return None;
    }
    let (_, x, y, _) = own.device_position_double(&pointer);
    Some((x, y))
}

/// Setzt die Eingabeflaeche des Overlays. `full` = ganzes Fenster klickbar,
/// sonst nur die uebergebenen Rechtecke (x, y, Breite, Hoehe in CSS-Pixeln).
/// Gibt `false` zurueck, wenn kein Overlay aktiv ist.
pub fn apply_input(full: bool, rects: &[(i32, i32, i32, i32)]) -> bool {
    INPUT.with(|input| *input.borrow_mut() = (full, rects.to_vec()));
    OVERLAY.with(|slot| {
        let slot = slot.borrow();
        let Some(overlay) = slot.as_ref() else {
            return false;
        };
        reapply_input(overlay);
        true
    })
}

/// Setzt die zuletzt gemeldete Eingabeflaeche (erneut) auf das Overlay.
fn reapply_input(overlay: &gtk::ApplicationWindow) {
    INPUT.with(|input| {
        let (full, rects) = &*input.borrow();
        if *full {
            overlay.input_shape_combine_region(None);
        } else {
            let region = gtk::cairo::Region::create();
            for &(x, y, width, height) in rects {
                let _ = region.union_rectangle(&gtk::cairo::RectangleInt::new(x, y, width, height));
            }
            overlay.input_shape_combine_region(Some(&region));
        }
    });
}

/// F8: Verschiebt das Overlay auf den naechsten Monitor (bei zweien also hin
/// und her). Das Frontend kann den aktuellen Monitor im Overlay-Modus nicht
/// selbst bestimmen, weil das versteckte Tauri-Fenster keine echte Position
/// hat. Deshalb merken wir uns hier, wo das Overlay gerade ist, und zaehlen
/// einfach weiter. Gibt `false` zurueck, wenn kein Overlay aktiv ist.
pub fn move_to_next_monitor() -> bool {
    OVERLAY.with(|slot| {
        let slot = slot.borrow();
        let Some(overlay) = slot.as_ref() else {
            return false;
        };
        let display = WidgetExt::display(overlay);
        // Nur echte Bildschirme: Virtuelle Ausgaben (z.B. Headless-Outputs fuer
        // Streaming) haben kein Modell und liegen oft weit ausserhalb des
        // sichtbaren Bereichs, dort waere das Pet unsichtbar.
        let mut candidates: Vec<i32> = (0..display.n_monitors())
            .filter(|&i| display.monitor(i).and_then(|m| m.model()).is_some())
            .collect();
        if candidates.is_empty() {
            candidates = (0..display.n_monitors()).collect();
        }
        if candidates.len() < 2 {
            return true;
        }
        // Beim ersten Wechsel: den Monitor erkennen, den Hyprland beim Start
        // gewaehlt hat. Danach den gemerkten Index nehmen.
        let current = MONITOR.with(|monitor| monitor.get()).unwrap_or_else(|| {
            overlay
                .window()
                .and_then(|window| display.monitor_at_window(&window))
                .and_then(|found| {
                    (0..display.n_monitors()).find(|&i| display.monitor(i).as_ref() == Some(&found))
                })
                .unwrap_or(candidates[0])
        });
        let position = candidates.iter().position(|&i| i == current).unwrap_or(0);
        let next = candidates[(position + 1) % candidates.len()];
        if let Some(monitor) = display.monitor(next) {
            // gtk-layer-shell bildet die Flaeche dafuer neu ab; die neue Groesse
            // kommt ueber die configure-Weiterleitung beim Frontend an.
            overlay.set_monitor(&monitor);
            MONITOR.with(|slot| slot.set(Some(next)));
        }
        true
    })
}
