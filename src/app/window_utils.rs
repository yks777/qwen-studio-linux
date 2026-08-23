use tauri::{AppHandle, Manager, WebviewWindow};

/// Returns the most relevant window to act on for window-scoped commands
/// (zoom, devtools, reload, minimize, etc.).
///
/// Priority: the last focused window (if it still exists), otherwise the
/// first open profile window (`main-*`), otherwise any webview window.
pub fn active_webview_window(app: &AppHandle) -> Option<WebviewWindow> {
    if let Some(state) = app.try_state::<crate::app::state::AppState>() {
        let last = state.last_focused.blocking_read();
        if let Some(label) = last.as_ref() {
            if let Some(w) = app.get_webview_window(label) {
                return Some(w);
            }
        }
    }

    if let Some(w) = first_profile_window(app) {
        return Some(w);
    }

    app.webview_windows()
        .into_values()
        .next()
}

/// Returns the first open profile window (`main-*`), if any.
pub fn first_profile_window(app: &AppHandle) -> Option<WebviewWindow> {
    app.webview_windows()
        .into_values()
        .find(|w| w.label().starts_with("main-"))
}

/// Returns the profile of the currently focused window, if any.
pub fn focused_profile(app: &AppHandle) -> Option<crate::profile::Profile> {
    let state = app.try_state::<crate::app::state::AppState>()?;
    let last = state.last_focused.blocking_read();
    let label = last.as_ref()?;
    let guard = state.window_profiles.blocking_read();
    guard.get(label).cloned()
}
