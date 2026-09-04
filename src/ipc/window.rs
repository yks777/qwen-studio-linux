use crate::webview::user_agent::USER_AGENT;
use std::sync::atomic::{AtomicU32, Ordering};
use tauri::Emitter;

static WINDOW_COUNTER: AtomicU32 = AtomicU32::new(1);

#[tauri::command]
pub async fn create_new_window(app: tauri::AppHandle) -> Result<String, String> {
    let id = WINDOW_COUNTER.fetch_add(1, Ordering::SeqCst);
    let label = format!("window-{}", id);
    let url = "https://chat.qwen.ai"
        .parse()
        .map_err(|e: url::ParseError| e.to_string())?;
    let script = crate::webview::js_injector::build_init_script();

    let mut builder =
        tauri::WebviewWindowBuilder::new(&app, &label, tauri::WebviewUrl::External(url))
            .title("Qwem Studio Linux")
            .inner_size(1280.0, 840.0)
            .min_inner_size(400.0, 600.0)
            .center()
            .resizable(true)
            .user_agent(USER_AGENT)
            .initialization_script(&script)
            .enable_clipboard_access()
            .on_navigation(|url| {
                let s = url.as_ref();
                if crate::webview::navigation::is_allowed(s) {
                    return true;
                }
                if s.starts_with("http://") || s.starts_with("https://") {
                    let _ = open::that(s);
                    return false;
                }
                false
            });

    if let Some(profile) = crate::app::window_utils::focused_profile_async(&app).await {
        builder = builder.data_directory(crate::profile::manager::data_dir_for(&profile.id));
    }

    let window = builder.build().map_err(|e| e.to_string())?;

    crate::app::window_utils::attach_file_drop_handler(&window);

    Ok(label)
}

#[tauri::command]
pub async fn minimize_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(w) = crate::app::window_utils::active_webview_window_async(&app).await {
        w.minimize().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn maximize_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(w) = crate::app::window_utils::active_webview_window_async(&app).await {
        if w.is_maximized().unwrap_or(false) {
            w.unmaximize().map_err(|e| e.to_string())?;
        } else {
            w.maximize().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn close_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(w) = crate::app::window_utils::active_webview_window_async(&app).await {
        w.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn open_devtool(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(w) = crate::app::window_utils::active_webview_window_async(&app).await {
        w.open_devtools();
    }
    Ok(())
}

#[tauri::command]
pub async fn toggle_hidden_devtools(app: tauri::AppHandle) -> Result<bool, String> {
    let w = crate::app::window_utils::active_webview_window_async(&app)
        .await
        .ok_or("No main window")?;
    if w.is_devtools_open() {
        w.close_devtools();
        Ok(false)
    } else {
        w.open_devtools();
        Ok(true)
    }
}

#[tauri::command]
pub async fn open_external_link(app: tauri::AppHandle, url: String) -> Result<bool, String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Ok(false);
    }
    // Validate URL parses and host is not empty to avoid javascript: or data: tricks
    let parsed = url::Url::parse(&url).map_err(|e| e.to_string())?;
    if parsed.host_str().is_none() {
        return Ok(false);
    }
    if crate::auth::domains::is_auth_url(&url) {
        if let Some(w) = crate::app::window_utils::active_webview_window_async(&app).await {
            let js_url = serde_json::to_string(&url).map_err(|e| e.to_string())?;
            let _ = w.eval(format!("window.location.href = {};", js_url));
            return Ok(true);
        }
    }
    open::that(&url).map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub async fn switch_theme(app: tauri::AppHandle, theme: String) -> Result<(), String> {
    app.emit(
        "event_from_main",
        serde_json::json!({
            "type": "theme_changed", "payload": theme
        }),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn switch_ln(app: tauri::AppHandle, ln: String) -> Result<(), String> {
    app.emit(
        "event_from_main",
        serde_json::json!({
            "type": "language_changed", "payload": ln
        }),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_title_bar_for_system_theme(
    app: tauri::AppHandle,
    is_dark: bool,
) -> Result<(), String> {
    app.emit(
        "event_from_main",
        serde_json::json!({
            "type": "system_theme_changed", "payload": is_dark
        }),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_language() -> Result<String, String> {
    Ok("en-US".into())
}
