use base64::Engine;
use serde::Deserialize;
use tauri::{AppHandle, Listener, Manager, WebviewWindow};

/// Maximum dropped file size we forward to the web app (50 MiB). Larger files
/// would blow up the base64 payload sent over `eval` and risk freezing the UI.
const MAX_DROP_SIZE: usize = 50 * 1024 * 1024;

/// Attaches a file-drop handler that forwards dropped files into the web app
/// exactly like a real browser would — the file becomes an attachment via the
/// composer's file input. Handling the drop in Rust (instead of letting the
/// webview receive the OS drop) prevents the site from breaking, since
/// WebKitGTK does not deliver `File` objects to the page reliably.
pub fn attach_file_drop_handler(window: &WebviewWindow) {
    let win = window.clone();
    let _ = window.listen("tauri://drag-drop", move |event| {
        #[derive(Deserialize)]
        struct DragDropPayload {
            paths: Vec<std::path::PathBuf>,
        }

        let payload: DragDropPayload = match serde_json::from_str(event.payload()) {
            Ok(p) => p,
            Err(_) => return,
        };

        for path in payload.paths {
            let bytes = match std::fs::read(&path) {
                Ok(b) => b,
                Err(_) => continue,
            };
            if bytes.len() > MAX_DROP_SIZE {
                continue;
            }
            let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
            let mime = mime_for_path(&path);
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "file".to_string());
            let js = format!(
                "window.__qwenInjectFile({:?}, {:?}, {:?})",
                b64, mime, name
            );
            let _ = win.eval(&js);
        }
    });
}

fn mime_for_path(path: &std::path::Path) -> String {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("bmp") => "image/bmp",
        Some("svg") => "image/svg+xml",
        Some("pdf") => "application/pdf",
        Some("txt") => "text/plain",
        Some("md") => "text/markdown",
        Some("csv") => "text/csv",
        Some("json") => "application/json",
        Some("xml") => "application/xml",
        Some("html") => "text/html",
        Some("doc") => "application/msword",
        Some("docx") => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        Some("xls") => "application/vnd.ms-excel",
        Some("xlsx") => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        Some("ppt") => "application/vnd.ms-powerpoint",
        Some("pptx") => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        Some("zip") => "application/zip",
        Some("mp3") => "audio/mpeg",
        Some("mp4") => "video/mp4",
        Some("wav") => "audio/wav",
        _ => "application/octet-stream",
    }
    .to_string()
}

/// Returns the most relevant window to act on for window-scoped commands
/// (zoom, devtools, reload, minimize, etc.).
///
/// Priority: the last focused window (if it still exists), otherwise the
/// first open profile window (`main-*`), otherwise any webview window.
///
/// Sync version — safe to call from GTK/main thread and from `on_menu_event`.
/// Uses `try_read` instead of `blocking_read` so it never panics when called
/// from a `tokio-rt-worker`.
pub fn active_webview_window(app: &AppHandle) -> Option<WebviewWindow> {
    if let Some(state) = app.try_state::<crate::app::state::AppState>() {
        if let Ok(last) = state.last_focused.try_read() {
            if let Some(label) = last.as_ref() {
                if let Some(w) = app.get_webview_window(label) {
                    return Some(w);
                }
            }
        }
    }

    if let Some(w) = first_profile_window(app) {
        return Some(w);
    }

    app.webview_windows().into_values().next()
}

/// Async variant for Tauri commands (tokio runtime). Never panics.
pub async fn active_webview_window_async(app: &AppHandle) -> Option<WebviewWindow> {
    if let Some(state) = app.try_state::<crate::app::state::AppState>() {
        let last = state.last_focused.read().await;
        if let Some(label) = last.as_ref() {
            if let Some(w) = app.get_webview_window(label) {
                return Some(w);
            }
        }
    }

    if let Some(w) = first_profile_window(app) {
        return Some(w);
    }

    app.webview_windows().into_values().next()
}

/// Returns the first open profile window (`main-*`), if any.
pub fn first_profile_window(app: &AppHandle) -> Option<WebviewWindow> {
    app.webview_windows()
        .into_values()
        .find(|w| w.label().starts_with("main-"))
}

/// Returns the profile of the currently focused window, if any (sync, non-panicking).
#[allow(dead_code)]
pub fn focused_profile(app: &AppHandle) -> Option<crate::profile::Profile> {
    let state = app.try_state::<crate::app::state::AppState>()?;
    let last = state.last_focused.try_read().ok()?;
    let label = last.as_ref()?.clone();
    drop(last);
    let guard = state.window_profiles.try_read().ok()?;
    guard.get(&label).cloned()
}

/// Async variant — use from `#[tauri::command] async fn`.
pub async fn focused_profile_async(app: &AppHandle) -> Option<crate::profile::Profile> {
    let state = app.try_state::<crate::app::state::AppState>()?;
    let label = state.last_focused.read().await.clone()?;
    let guard = state.window_profiles.read().await;
    guard.get(&label).cloned()
}
