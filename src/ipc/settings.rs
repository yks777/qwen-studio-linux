use crate::config;

const ALLOWED_SETTINGS_KEYS: &[&str] = &[
    "general",
    "theme",
    "language",
    "check_updates",
    "checkUpdates",
    "zoom",
    "notifications",
];

#[tauri::command]
pub async fn get_setting(
    _app: tauri::AppHandle,
    key: String,
) -> Result<Option<serde_json::Value>, String> {
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
    if key.len() > 128 || key.is_empty() {
        return Err("Invalid key".into());
    }
=======
>>>>>>> c0c2f30 (Fix: Upload medias e username)
=======
    if key.len() > 128 || key.is_empty() {
        return Err("Invalid key".into());
    }
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
=======
>>>>>>> c0c2f30 (Fix: Upload medias e username)
=======
    if key.len() > 128 || key.is_empty() {
        return Err("Invalid key".into());
    }
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
    let raw = config::store::load_raw();
    Ok(raw.get(&key).cloned())
}

#[tauri::command]
pub async fn set_setting(
    _app: tauri::AppHandle,
    key: String,
    value: serde_json::Value,
) -> Result<(), String> {
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
=======
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
=======
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
    if key.len() > 128 || key.is_empty() {
        return Err("Invalid key".into());
    }
    // Block direct overwrite of mcpServers via generic settings API — use dedicated command
    if key == "mcpServers" || key == "mcp_servers" {
        return Err("Use mcp_client_update_config for MCP servers".into());
    }
    if !ALLOWED_SETTINGS_KEYS.contains(&key.as_str()) && !key.starts_with("general.") {
        // Allow but log; keep permissive for forward-compat but prevent mcpServers hijack
        log::warn!("[settings] unknown key: {}", key);
    }
    // Value size cap 1 MB
    let val_str = serde_json::to_string(&value).map_err(|e| e.to_string())?;
    if val_str.len() > 1024 * 1024 {
        return Err("Value too large".into());
    }
<<<<<<< HEAD
<<<<<<< HEAD
=======
>>>>>>> c0c2f30 (Fix: Upload medias e username)
=======
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
=======
>>>>>>> c0c2f30 (Fix: Upload medias e username)
=======
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
    let mut raw = config::store::load_raw();
    if let Some(obj) = raw.as_object_mut() {
        obj.insert(key, value);
    }
    config::store::save_raw(&raw)
}
