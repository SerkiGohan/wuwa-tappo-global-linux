//! Native menu bridge.
//!
//! The macOS build exposed an AppKit menu bar and dock menu. The Linux rebuild
//! keeps the official in-window panel and keyboard shortcuts instead, so these
//! commands are intentionally no-ops. They stay registered because the official
//! frontend still invokes them during boot and when opening the dock menu.

use serde_json::Value;
use tauri::AppHandle;

pub fn setup(_app: &AppHandle) -> Result<(), String> {
    Ok(())
}

pub fn sync(_app: &AppHandle, _args: &Value) -> Result<Value, String> {
    Ok(Value::Null)
}

pub fn popup(
    _app: &AppHandle,
    _window: &tauri::WebviewWindow,
    _x: f64,
    _y: f64,
) -> Result<Value, String> {
    Ok(Value::Null)
}
