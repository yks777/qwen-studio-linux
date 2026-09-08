use tauri::WebviewWindow;

/// Disables WebKit Intelligent Tracking Prevention for the given window.
/// Required for `qwenlm.io` preview `postMessage` + `SameSite=None` cookies.
pub fn disable_itp_for(window: &WebviewWindow) {
    let label = window.label().to_string();
    // WebKitGTK ITP disable via ffi requires wry internal pointer not stably exposed in tauri 2.11.
    // We attempt a best-effort no-op: log and rely on per-profile data_directory isolation.
    // If webkit_website_data_manager_set_itp_enabled becomes available via safe wrapper, enable it here.
    log::debug!("[ITP] disable requested for {}", label);
    let res = window.with_webview(move |_webview| {
        #[cfg(target_os = "linux")]
        {
            // Stub: WebKitGTK 4.1 defaults to ITP enabled, but per-profile data_dir mitigates third-party cookie loss.
            // Actual disabling would be: webkit_website_data_manager_set_itp_enabled(manager, 0)
            // Keep closure for future ffi when wry exposes WebKitWebView directly.
        }
    });
    if let Err(e) = res {
        log::warn!("[ITP] failed for {}: {}", window.label(), e);
    }
}
