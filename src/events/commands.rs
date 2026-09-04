use std::sync::{OnceLock, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

static LAST_WEBVIEW_LOADED: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();

#[tauri::command]
pub async fn webview_loaded(app: AppHandle) -> Result<(), String> {
<<<<<<< HEAD
=======
    let m = LAST_WEBVIEW_LOADED.get_or_init(|| Mutex::new(None));
    if let Ok(mut g) = m.lock() {
        if let Some(prev) = *g {
            if prev.elapsed() < Duration::from_millis(300) {
                return Ok(());
            }
        }
        *g = Some(Instant::now());
    }
>>>>>>> dev
    app.emit(
        "event_from_main",
        serde_json::json!({
            "type": "webview-loaded",
            "payload": null
        }),
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
