use crate::config;

#[tauri::command]
pub async fn get_setting(
    _app: tauri::AppHandle,
    key: String,
) -> Result<Option<serde_json::Value>, String> {
    let raw = config::store::load_raw();
    Ok(raw.get(&key).cloned())
}

#[tauri::command]
pub async fn set_setting(
    _app: tauri::AppHandle,
    key: String,
    value: serde_json::Value,
) -> Result<(), String> {
    let mut raw = config::store::load_raw();
    if let Some(obj) = raw.as_object_mut() {
        obj.insert(key, value);
    }
    config::store::save_raw(&raw)
}
