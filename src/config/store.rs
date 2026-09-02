use serde_json::Value;
use std::fs;

use super::paths;
use super::schema::Settings;

pub fn save(settings: &Settings) -> Result<(), String> {
    let path = paths::settings_file();
    let content = serde_json::to_string(settings).map_err(|e| e.to_string())?;
    if let Ok(existing) = fs::read_to_string(&path) {
        if existing == content {
            return Ok(());
        }
    }
<<<<<<< HEAD
    let tmp = path.with_file_name(format!(
        "{}.tmp",
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("settings.json")
    ));
=======
    let tmp = path.with_extension("json.tmp");
>>>>>>> 0f81055 (Melhorias)
    fs::write(&tmp, &content).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())
}

pub fn load() -> Settings {
    let path = paths::settings_file();
    match fs::read_to_string(&path) {
        Ok(c) => match serde_json::from_str(&c) {
            Ok(v) => v,
            Err(e) => {
                log::warn!("[config] corrupted settings.json, using defaults: {}", e);
                Settings::default()
            }
        },
        Err(_) => Settings::default(),
    }
}

pub fn load_raw() -> Value {
    let path = paths::settings_file();
    fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or(Value::Object(serde_json::Map::new()))
}

pub fn save_raw(value: &Value) -> Result<(), String> {
    let path = paths::settings_file();
    let content = serde_json::to_string(value).map_err(|e| e.to_string())?;
    if let Ok(existing) = fs::read_to_string(&path) {
        if existing == content {
            return Ok(());
        }
    }
<<<<<<< HEAD
    let tmp = path.with_file_name(format!(
        "{}.tmp",
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("settings.json")
    ));
=======
    let tmp = path.with_extension("json.tmp");
>>>>>>> 0f81055 (Melhorias)
    fs::write(&tmp, &content).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())
}
