fn persist_zoom(zoom: f64) {
    let mut s = crate::config::store::load();
    let clamped = zoom.clamp(0.5, 3.0);
    s.general.zoom = clamped;
    let _ = crate::config::store::save(&s);
}

fn apply_zoom(app: &tauri::AppHandle, script: &str) {
    if let Some(w) = crate::app::window_utils::active_webview_window(app) {
        if let Err(e) = w.eval(script) {
            log::warn!("[Shortcuts] zoom eval failed: {}", e);
        }
    }
}

fn current_zoom() -> f64 {
    let s = crate::config::store::load();
    if s.general.zoom == 0.0 {
        1.0
    } else {
        s.general.zoom.clamp(0.5, 3.0)
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
            }
        }
        "hard_reload" => {
            if let Some(w) = crate::app::window_utils::active_webview_window(&app) {
                // hard reload sem cache (WebKit ignora cache)
                if let Err(e) = w.eval("location.reload(true);") {
                    log::warn!("[Shortcuts] hard_reload eval failed: {}", e);
                }
            }
        }
        "go_back" => {
            if let Some(w) = crate::app::window_utils::active_webview_window(&app) {
                let _ = w.eval("history.back();");
            }
        }
        "go_forward" => {
            if let Some(w) = crate::app::window_utils::active_webview_window(&app) {
                let _ = w.eval("history.forward();");
            }
        }
        "find" => {
            if let Some(w) = crate::app::window_utils::active_webview_window(&app) {
                let _ = w.eval("window.__qwenFindOpen && window.__qwenFindOpen();");
            }
        }
        "print" => {
            if let Some(w) = crate::app::window_utils::active_webview_window(&app) {
                let _ = w.eval("window.print();");
            }
        }
        "devtools" => {
            let _ = super::window::toggle_hidden_devtools(app).await;
        }
        "zoom_in" => {
<<<<<<< HEAD
            let z = (current_zoom() + 0.1).clamp(0.5, 3.0);
            persist_zoom(z);
            apply_zoom(
                &app,
                &format!("document.documentElement.style.zoom = '{}'; document.body.style.zoom = '{}';", z, z),
            );
        }
        "zoom_out" => {
            let z = (current_zoom() - 0.1).clamp(0.5, 3.0);
            persist_zoom(z);
            apply_zoom(
                &app,
                &format!("document.documentElement.style.zoom = '{}'; document.body.style.zoom = '{}';", z, z),
            );
        }
        "zoom_reset" => {
            persist_zoom(1.0);
            apply_zoom(&app, "document.documentElement.style.zoom = '1.0'; document.body.style.zoom = '1.0';");
=======
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
>>>>>>> ce2f600 (optimization)
        }
        _ => {
            log::warn!("[Shortcuts] Unknown action: {}", action);
        }
    }
    Ok(())
}
