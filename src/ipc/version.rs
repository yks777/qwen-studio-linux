#[tauri::command]
pub async fn get_app_version() -> Result<String, String> {
    Ok(crate::config::schema::APP_VERSION.to_string())
}

#[tauri::command]
pub async fn get_platform_info() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH
    }))
}
