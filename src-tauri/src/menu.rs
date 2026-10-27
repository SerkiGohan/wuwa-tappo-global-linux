//! Native menu bridge.

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
