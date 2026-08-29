<<<<<<< HEAD
<<<<<<< HEAD
=======
#![allow(dead_code)]

<<<<<<< HEAD
>>>>>>> c0c2f30 (Fix: Upload medias e username)
=======
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
=======
>>>>>>> c0c2f30 (Fix: Upload medias e username)
use tauri::{Emitter, Listener};
use tokio::sync::broadcast;

const ALLOWED_EVENT_TYPES: &[&str] = &[
    "theme_changed",
    "language_changed",
    "system_theme_changed",
    "webview-loaded",
];

pub fn setup_event_forwarding(app: &tauri::AppHandle) {
    let handle = app.clone();
    app.listen_any("event_to_main", move |event| {
        let payload_str = event.payload();
        let Ok(payload) = serde_json::from_str::<serde_json::Value>(payload_str) else {
            return;
        };
        // Validate event type to prevent JS from broadcasting arbitrary events
        if let Some(t) = payload.get("type").and_then(|v| v.as_str()) {
            if !ALLOWED_EVENT_TYPES.contains(&t) {
                log::warn!("[EventBus] blocked unknown event type: {}", t);
                return;
            }
        }
        let _ = handle.emit("event_from_main", payload);
    });
}
