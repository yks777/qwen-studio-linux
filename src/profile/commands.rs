use tauri::{AppHandle, Emitter};
use crate::profile::{manager, Profile};

#[tauri::command]
pub fn list_profiles() -> Vec<Profile> {
    manager::load()
}

#[tauri::command]
pub fn create_profile(
    app: AppHandle,
    name: String,
    category: Option<String>,
    icon: Option<String>,
) -> Result<Profile, String> {
    let profile = manager::create(&name, category.as_deref(), icon.as_deref())?;
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

#[tauri::command]
pub fn list_categories() -> Vec<String> {
    manager::list_categories()
}

#[tauri::command]
pub fn update_profile(
    app: AppHandle,
    id: String,
    name: Option<String>,
    category: Option<String>,
    icon: Option<String>,
) -> Result<Profile, String> {
    let profile = manager::update_profile(
        &id,
        name.as_deref(),
        category.as_deref(),
        icon.as_deref(),
    )?;
    let _ = app.emit("profiles-updated", ());
    Ok(profile)
}
