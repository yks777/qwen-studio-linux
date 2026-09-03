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
            .on_navigation(|url| crate::webview::navigation::is_allowed(url.as_ref()));

    if let Some(profile) = crate::app::window_utils::focused_profile_async(&app).await {
        builder = builder.data_directory(crate::profile::manager::data_dir_for(&profile.id));
    }

    let window = builder.build().map_err(|e| e.to_string())?;

    // Aplica zoom persistido
    {
        let zoom = crate::config::store::load().general.zoom;
        if zoom != 0.0 && (zoom - 1.0).abs() > f64::EPSILON {
            let js = format!(
                "document.documentElement.style.zoom='{}'; document.body.style.zoom='{}';",
                zoom, zoom
            );
            let _ = window.eval(js);
        }
    }

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
    // Aceita http(s) + mailto/blob/data para paridade navegador; javascript: continua bloqueado
    let is_http = url.starts_with("http://") || url.starts_with("https://");
    let is_mailto = url.starts_with("mailto:");
    let is_blob_data = url.starts_with("blob:") || url.starts_with("data:");
    if !is_http && !is_mailto && !is_blob_data {
        return Ok(false);
    }
    // Validação de host para http(s); mailto/blob/data não exigem host
    if is_http {
        let parsed = url::Url::parse(&url).map_err(|e| e.to_string())?;
        if parsed.host_str().is_none() {
            return Ok(false);
        }
    } else if is_blob_data {
        // blob:/data: já validados pelo prefixo; deixa o navegador/OS decidir
        // Se vier de window.open, abre no OS browser como fallback
        let _ = open::that(&url);
        return Ok(true);
    } else if is_mailto {
        let _ = open::that(&url);
        return Ok(true);
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
