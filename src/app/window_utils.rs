use tauri::{AppHandle, Manager, WebviewWindow};

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
