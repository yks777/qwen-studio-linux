use crate::config::schema::DialogOptions;

#[tauri::command]
pub async fn show_native_dialog(
    app: tauri::AppHandle,
    options: DialogOptions,
) -> Result<String, String> {
    use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
    let kind = match options.title.to_lowercase().as_str() {
        "error" => MessageDialogKind::Error,
        "warning" => MessageDialogKind::Warning,
        _ => MessageDialogKind::Info,
    };
    let result = app
        .dialog()
        .message(&options.message)
        .title(&options.title)
        .kind(kind)
        .buttons(MessageDialogButtons::Ok)
        .blocking_show();
    Ok(if result { "ok".into() } else { "cancel".into() })
}

#[tauri::command]
pub async fn request_file_access(
    app: tauri::AppHandle,
    purpose: String,
    return_file: Option<bool>,
) -> Result<serde_json::Value, String> {
    use tauri_plugin_dialog::DialogExt;
    if purpose.len() > 256 {
        return Err("Purpose too long".into());
    }
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog().file().set_title(&purpose).pick_file(move |f| {
        let _ = tx.send(f.and_then(|f| f.as_path().map(|p| p.to_string_lossy().to_string())));
    });
    let path = rx
        .await
        .map_err(|e| e.to_string())?
        .ok_or("No file selected")?;
    let mut result = serde_json::json!({ "filePath": path });
    if return_file.unwrap_or(false) {
        let content = tokio::task::spawn_blocking({
            let p = path.clone();
            move || {
                let meta = std::fs::metadata(&p).map_err(|e| e.to_string())?;
                if meta.len() > 10 * 1024 * 1024 {
                    return Err("File too large (10 MB limit)".to_string());
                }
                std::fs::read_to_string(&p).map_err(|e| e.to_string())
            }
        })
        .await
        .map_err(|e| e.to_string())??;
        result["file"] = serde_json::Value::String(content);
    }
    Ok(result)
}
