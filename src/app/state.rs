use crate::mcp::manager::McpManager;
use crate::profile::Profile;
use crate::update::manager::UpdateManager;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AppState {
    pub mcp: Arc<RwLock<McpManager>>,
    pub updates: Arc<RwLock<UpdateManager>>,
    /// Maps a profile window label (e.g. `main-<id>`) to its profile.
    pub window_profiles: Arc<RwLock<HashMap<String, Profile>>>,
    /// Label of the currently focused window (used as the "active" window).
    pub last_focused: Arc<RwLock<Option<String>>>,
    pub session_capture_running: Arc<AtomicBool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            mcp: Arc::new(RwLock::new(McpManager::new())),
            updates: Arc::new(RwLock::new(UpdateManager::new())),
            window_profiles: Arc::new(RwLock::new(HashMap::new())),
            last_focused: Arc::new(RwLock::new(None)),
            session_capture_running: Arc::new(AtomicBool::new(false)),
        }
    }
}
