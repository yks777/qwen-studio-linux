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
    if key.len() > 128 || key.is_empty() {
        return Err("Invalid key".into());
    }
=======
>>>>>>> c0c2f30 (Fix: Upload medias e username)
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
=======
>>>>>>> c0c2f30 (Fix: Upload medias e username)
    let mut raw = config::store::load_raw();
    if let Some(obj) = raw.as_object_mut() {
        obj.insert(key, value);
    }
    config::store::save_raw(&raw)
}
