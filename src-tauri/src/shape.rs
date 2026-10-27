//! X11 input-shape management for the main overlay.
//!
//! Wayland compositors do not expose a global pointer position to XWayland
//! clients when the overlay is click-through. Polling the cursor therefore
//! cannot decide when to re-enable input. Instead, publish the union of the
//! official hit regions as the window's XShape input region: the compositor
//! routes pointer events inside the pet regions to us, and everything else
//! passes through without any polling.

use gdkx11::X11Window;
use gtk::glib::Cast;
use gtk::prelude::*;
use x11rb::connection::Connection;
use x11rb::protocol::shape::{ConnectionExt as ShapeExt, SK, SO};
use x11rb::protocol::xproto::{ClipOrdering, Rectangle};
use x11rb::rust_connection::RustConnection;

pub struct InputShape {
    connection: RustConnection,
    window: u32,
}

impl InputShape {
    pub fn for_window(window: &tauri::WebviewWindow) -> Result<Self, String> {
        let gtk_window = window.gtk_window().map_err(|error| error.to_string())?;
        let gdk_window = gtk_window
            .window()
            .ok_or("The GTK window is not realized yet")?;
        let xid = gdk_window
            .downcast_ref::<X11Window>()
            .ok_or("The GDK window is not an X11 window")?
            .xid() as u32;
        let (connection, _) = x11rb::connect(None)
            .map_err(|error| format!("X11 shape connection failed: {error}"))?;
        Ok(Self {
            connection,
            window: xid,
        })
    }

    pub fn apply(&self, rectangles: &[Rectangle]) -> Result<(), String> {
        self.connection
            .shape_rectangles(
                SO::SET,
                SK::INPUT,
                ClipOrdering::UNSORTED,
                self.window,
                0,
                0,
                rectangles,
            )
            .map_err(|error| format!("XShape request failed: {error}"))?
            .check()
            .map_err(|error| format!("XShape request was rejected: {error}"))?;
        self.connection
            .flush()
            .map_err(|error| format!("XShape flush failed: {error}"))
    }
}
