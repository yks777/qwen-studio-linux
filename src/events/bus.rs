use tauri::{Emitter, Listener};

const ALLOWED_EVENT_TYPES: &[&str] = &[
    "theme_changed",
    "language_changed",
    "system_theme_changed",
    "webview-loaded",
];

pub fn setup_event_forwarding(app: &tauri::AppHandle) {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    let handle = app.clone();
    let last_emit: Arc<Mutex<HashMap<String, Instant>>> = Arc::new(Mutex::new(HashMap::new()));
    app.listen_any("event_to_main", move |event| {
        let payload_str = event.payload();
        let Ok(payload) = serde_json::from_str::<serde_json::Value>(payload_str) else {
            return;
        };
<<<<<<< HEAD
        // Validate event type to prevent JS from broadcasting arbitrary events
=======
>>>>>>> dev
        if let Some(t) = payload.get("type").and_then(|v| v.as_str()) {
            if !ALLOWED_EVENT_TYPES.contains(&t) {
                log::warn!("[EventBus] blocked unknown event type: {}", t);
                return;
            }
<<<<<<< HEAD
=======
            // Throttle high-frequency events (e.g., webview-loaded spam) — 300ms per type
            if t == "webview-loaded" {
                if let Ok(mut m) = last_emit.lock() {
                    let now = Instant::now();
                    if let Some(prev) = m.get(t) {
                        if now.duration_since(*prev) < Duration::from_millis(300) {
                            return;
                        }
                    }
                    m.insert((*t).to_string(), now);
                }
            }
>>>>>>> dev
        }
        let _ = handle.emit("event_from_main", payload);
    });
}
