use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn webview_loaded(app: AppHandle) -> Result<(), String> {
    app.emit("event_from_main", serde_json::json!({
        "type": "webview-loaded",
        "payload": null
    })).map_err(|e| e.to_string())?;
    Ok(())
}
