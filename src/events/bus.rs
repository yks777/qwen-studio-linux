#![allow(dead_code)]

use tokio::sync::broadcast;
use tauri::{Emitter, Listener};

pub struct EventBus {
    sender: broadcast::Sender<serde_json::Value>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(256);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<serde_json::Value> {
        self.sender.subscribe()
    }

    pub fn publish(&self, event: serde_json::Value) {
        let _ = self.sender.send(event);
    }
}

pub fn setup_event_forwarding(app: &tauri::AppHandle) {
    let handle = app.clone();
    app.listen_any("event_to_main", move |event| {
        let payload_str = event.payload();
        if let Ok(payload) = serde_json::from_str::<serde_json::Value>(payload_str) {
            let _ = handle.emit("event_from_main", payload);
        }
    });
}
