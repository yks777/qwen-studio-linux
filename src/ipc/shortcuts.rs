fn apply_zoom(app: &tauri::AppHandle, script: &str) {
    if let Some(w) = crate::app::window_utils::active_webview_window(app) {
        if let Err(e) = w.eval(script) {
            log::warn!("[Shortcuts] zoom eval failed: {}", e);
        }
    }
}

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
                if let Err(e) = w.eval("location.reload();") {
                    log::warn!("[Shortcuts] reload eval failed: {}", e);
                }
<<<<<<< HEAD
=======
            }
        }
        "hard_reload" => {
            if let Some(w) = crate::app::window_utils::active_webview_window(&app) {
                if let Err(e) = w.eval("location.reload(true);") {
                    log::warn!("[Shortcuts] hard_reload eval failed: {}", e);
                }
>>>>>>> dev
            }
        }
        "devtools" => {
            let _ = super::window::toggle_hidden_devtools(app).await;
        }
<<<<<<< HEAD
=======
        "fullscreen" => {
            let _ = super::window::toggle_fullscreen(app).await;
        }
        "find" => {
            if let Some(w) = crate::app::window_utils::active_webview_window(&app) {
                let _ = w.eval("window.__qwenFindOpen && window.__qwenFindOpen();");
            }
        }
>>>>>>> dev
        "zoom_in" => apply_zoom(
            &app,
            "document.body.style.zoom = Math.min(2.0, parseFloat(document.body.style.zoom||'1') + 0.1);",
        ),
        "zoom_out" => apply_zoom(
            &app,
            "document.body.style.zoom = Math.max(0.5, parseFloat(document.body.style.zoom||'1') - 0.1);",
        ),
        "zoom_reset" => apply_zoom(&app, "document.body.style.zoom = 1.0;"),
        _ => {
            log::warn!("[Shortcuts] Unknown action: {}", action);
        }
    }
    Ok(())
}
