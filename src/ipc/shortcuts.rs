#[tauri::command]
pub async fn handle_shortcut(app: tauri::AppHandle, action: String) -> Result<(), String> {
    match action.as_str() {
        "new_window" => {
            let _ = super::window::create_new_window(app).await;
        }
        "close_window" => {
            let _ = super::window::close_window(app).await;
        }
        "reload" => {
            if let Some(w) = crate::app::window_utils::active_webview_window(&app) {
                let _ = w.eval("location.reload();");
            }
        }
        "devtools" => {
            let _ = super::window::toggle_hidden_devtools(app).await;
        }
        "zoom_in" => {
            if let Some(w) = crate::app::window_utils::active_webview_window(&app) {
                let _ = w.eval("document.body.style.zoom = Math.min(2.0, parseFloat(document.body.style.zoom||'1') + 0.1);");
            }
        }
        "zoom_out" => {
            if let Some(w) = crate::app::window_utils::active_webview_window(&app) {
                let _ = w.eval("document.body.style.zoom = Math.max(0.5, parseFloat(document.body.style.zoom||'1') - 0.1);");
            }
        }
        "zoom_reset" => {
            if let Some(w) = crate::app::window_utils::active_webview_window(&app) {
                let _ = w.eval("document.body.style.zoom = 1.0;");
            }
        }
        _ => {
            log::warn!("[Shortcuts] Unknown action: {}", action);
        }
    }
    Ok(())
}
