<<<<<<< HEAD
<<<<<<< HEAD
=======
use crate::events::bus::EventBus;
>>>>>>> c0c2f30 (Fix: Upload medias e username)
=======
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
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
    /// Last session hash per profile (cookies len + localStorage hash) to skip IPC when unchanged.
    pub last_session_hash: Arc<tokio::sync::Mutex<HashMap<String, u64>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            mcp: Arc::new(RwLock::new(McpManager::new())),
            updates: Arc::new(RwLock::new(UpdateManager::new())),
            window_profiles: Arc::new(RwLock::new(HashMap::new())),
            last_focused: Arc::new(RwLock::new(None)),
            session_capture_running: Arc::new(AtomicBool::new(false)),
            last_session_hash: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }
}
