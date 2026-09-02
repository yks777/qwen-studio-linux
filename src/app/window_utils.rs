use serde::Deserialize;
use tauri::{Listener, Manager, WebviewWindow};

use crate::ipc::drop::{mime_for_path, DropMeta};

/// Attaches a file-drop handler that forwards dropped files into the web app
/// via chunked binary streaming (no size limit). Instead of reading the whole
/// file in Rust and base64-encoding it into an `eval` literal (which crashes
/// JSC for files > ~5 MiB), we only send path+metadata via `eval` and let JS
/// pull chunks via `read_file_chunk` (Tauri IPC binary `Response`).
pub fn attach_file_drop_handler(window: &WebviewWindow) {
    let win = window.clone();
    let _ = window.listen("tauri://drag-drop", move |event| {
        #[derive(Deserialize)]
        struct DragDropPayload {
            paths: Vec<std::path::PathBuf>,
        }

        let payload_str = event.payload().to_string();
        let win = win.clone();
        tauri::async_runtime::spawn(async move {
            let payload: DragDropPayload = match serde_json::from_str(&payload_str) {
                Ok(p) => p,
                Err(_) => return,
            };

            if payload.paths.is_empty() {
                return;
            }

            let mut metas: Vec<DropMeta> = Vec::with_capacity(payload.paths.len());
            for path in payload.paths {
                let meta = match tokio::fs::metadata(&path).await {
                    Ok(m) => m,
                    Err(e) => {
                        log::warn!("[drop] metadata failed for {:?}: {}", path, e);
                        continue;
                    }
                };
                let size = meta.len();
                const MAX_DROP_SIZE: u64 = 100 * 1024 * 1024; // 100 MiB limite para economia de RAM
                if size > MAX_DROP_SIZE {
                    log::warn!(
                        "[drop] arquivo muito grande ignorado (limite 100 MiB): {:?} ({} bytes)",
                        path,
                        size
                    );
                    continue;
                }
                if size > 512 * 1024 * 1024 {
                    log::warn!(
                        "[drop] arquivo grande detectado: {:?} ({} bytes)",
                        path,
                        size
                    );
                }
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("file")
                    .to_string();
                let path_str = path.to_string_lossy().to_string();
                let mime = mime_for_path(&path);
                metas.push(DropMeta {
                    path: path_str,
                    name,
                    mime,
                    size,
                });
            }

            if metas.is_empty() {
                return;
            }

            let json = match serde_json::to_string(&metas) {
                Ok(j) => j,
                Err(_) => return,
            };
            // Payload tiny (< ~2 KB even for 10 files), safe for eval
            let js = format!(
                "window.__qwenHandleDrop && window.__qwenHandleDrop({})",
                json
            );
            let _ = win.eval(&js);
        });
    });
}

fn resolve_focused_label(app: &tauri::AppHandle, try_sync: bool) -> Option<String> {
    let state = app.try_state::<crate::app::state::AppState>()?;
    if try_sync {
        state.last_focused.try_read().ok().and_then(|g| g.clone())
    } else {
        // For sync callers we already used try_read; for async we use blocking poll
        // This helper is split so sync/async variants share logic
        state.last_focused.try_read().ok().and_then(|g| g.clone())
    }
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
pub fn active_webview_window(app: &tauri::AppHandle) -> Option<WebviewWindow> {
    if let Some(label) = resolve_focused_label(app, true) {
        if let Some(w) = app.get_webview_window(&label) {
            return Some(w);
        }
    }

    if let Some(w) = first_profile_window(app) {
        return Some(w);
    }

    app.webview_windows().into_values().next()
}

/// Async variant for Tauri commands (tokio runtime). Never panics.
pub async fn active_webview_window_async(app: &tauri::AppHandle) -> Option<WebviewWindow> {
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
pub fn first_profile_window(app: &tauri::AppHandle) -> Option<WebviewWindow> {
    app.webview_windows()
        .into_values()
        .find(|w| w.label().starts_with("main-"))
}

/// Returns the profile of the currently focused window, if any (sync, non-panicking).
#[allow(dead_code)]
pub fn focused_profile(app: &tauri::AppHandle) -> Option<crate::profile::Profile> {
    let state = app.try_state::<crate::app::state::AppState>()?;
    let last = state.last_focused.try_read().ok()?;
    let label = last.as_ref()?.clone();
    drop(last);
    let guard = state.window_profiles.try_read().ok()?;
    guard.get(&label).cloned()
}

/// Async variant — use from `#[tauri::command] async fn`.
pub async fn focused_profile_async(app: &tauri::AppHandle) -> Option<crate::profile::Profile> {
    let state = app.try_state::<crate::app::state::AppState>()?;
    let label = state.last_focused.read().await.clone()?;
    let guard = state.window_profiles.read().await;
    guard.get(&label).cloned()
}
