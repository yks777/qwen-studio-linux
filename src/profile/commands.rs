use tauri::{AppHandle, Emitter};
use crate::profile::{manager, Profile};

#[tauri::command]
pub fn list_profiles() -> Vec<Profile> {
    manager::load()
}

#[tauri::command]
pub fn create_profile(app: AppHandle, name: String) -> Result<Profile, String> {
    let profile = manager::create(&name)?;
    let _ = app.emit("profiles-updated", ());
    Ok(profile)
}

#[tauri::command]
pub fn rename_profile(app: AppHandle, id: String, name: String) -> Result<(), String> {
    let result = manager::rename(&id, &name);
    if result.is_ok() {
        let _ = app.emit("profiles-updated", ());
    }
    result
}

#[tauri::command]
pub fn delete_profile(app: AppHandle, id: String) -> Result<(), String> {
    let result = manager::delete(&id);
    if result.is_ok() {
        let _ = app.emit("profiles-updated", ());
    }
    result
}

#[tauri::command]
pub fn launch_profile(app: AppHandle, id: String) -> Result<(), String> {
    let profile = manager::load()
        .into_iter()
        .find(|p| p.id == id)
        .ok_or("Profile not found")?;
    crate::app::lifecycle::open_profile(&app, &profile).map_err(|e| e.to_string())
}
