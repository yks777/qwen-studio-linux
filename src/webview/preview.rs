use std::sync::atomic::{AtomicU32, Ordering};

use crate::webview::user_agent::USER_AGENT;
use tauri::{Manager, WebviewUrl};

static PREVIEW_COUNTER: AtomicU32 = AtomicU32::new(1);

#[tauri::command]
pub async fn open_preview_window(app: tauri::AppHandle, url: String) -> Result<bool, String> {
    if !crate::webview::navigation::is_preview_url(&url) {
        return Err("URL is not a preview URL".into());
    }
    let parsed: url::Url = url::Url::parse(&url).map_err(|e| e.to_string())?;
    if parsed.host_str().is_none() {
        return Err("Invalid URL host".into());
    }

    // Reuse existing preview window if present
    let existing_label = "preview-1".to_string();
    if let Some(existing) = app.get_webview_window(&existing_label) {
        let js_url = serde_json::to_string(&url).map_err(|e| e.to_string())?;
        let _ = existing.eval(format!("window.location.replace({});", js_url));
        let _ = existing.show();
        let _ = existing.set_focus();
        return Ok(true);
    }

    let label = format!("preview-{}", PREVIEW_COUNTER.fetch_add(1, Ordering::SeqCst));
    // Ensure unique if preview-1 already taken and we incremented
    let label = if app.get_webview_window(&label).is_some() {
        format!("preview-{}", PREVIEW_COUNTER.fetch_add(1, Ordering::SeqCst))
    } else {
        label
    };

    let external_url = url.parse().map_err(|e: url::ParseError| e.to_string())?;
    let script = crate::webview::js_injector::build_init_script();

    let mut builder = tauri::WebviewWindowBuilder::new(&app, &label, WebviewUrl::External(external_url))
        .title("Qwen Preview")
        .inner_size(1280.0, 900.0)
        .min_inner_size(800.0, 600.0)
        .center()
        .resizable(true)
        .visible(true)
        .focused(true)
        .user_agent(USER_AGENT)
        .initialization_script(&script)
        .enable_clipboard_access()
        .on_navigation(|url| crate::webview::navigation::should_allow_inline(url.as_ref()));

    if let Some(profile) = crate::app::window_utils::focused_profile_async(&app).await {
        builder = builder.data_directory(crate::profile::manager::data_dir_for(&profile.id));
    }

    let window = builder.build().map_err(|e| e.to_string())?;
    crate::webview::itp::disable_itp_for(&window);
    crate::app::window_utils::attach_file_drop_handler(&window);

    Ok(true)
}
