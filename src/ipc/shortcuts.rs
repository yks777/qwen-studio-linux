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
                let _ = w.eval(
                    "window.__qwenSetZoom && window.__qwenSetZoom(Math.min(2.0, (window.__qwenCurrentZoom||1)+0.1));",
                );
            }
        }
        "zoom_out" => {
            if let Some(w) = crate::app::window_utils::active_webview_window(&app) {
                let _ = w.eval(
                    "window.__qwenSetZoom && window.__qwenSetZoom(Math.max(0.5, (window.__qwenCurrentZoom||1)-0.1));",
                );
            }
        }
        "zoom_reset" => {
            if let Some(w) = crate::app::window_utils::active_webview_window(&app) {
                let _ = w.eval("window.__qwenSetZoom && window.__qwenSetZoom(1.0);");
            }
        }
        _ => {
            log::warn!("[Shortcuts] Unknown action: {}", action);
        }
    }
    Ok(())
}
