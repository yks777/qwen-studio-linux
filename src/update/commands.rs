use tauri::{AppHandle, Emitter, Manager};
use crate::app::state::AppState;
use crate::config::schema::UpdateInfo;

#[tauri::command]
pub async fn check_for_updates(app: AppHandle, silent: bool) -> Result<UpdateInfo, String> {
    let current = crate::config::schema::APP_VERSION;

    let (latest, notes) = match super::checker::fetch_latest_version().await {
        Ok(v) => v,
        Err(e) => {
            log::warn!("[Update] Check failed: {}", e);
            return Ok(UpdateInfo {
                current_version: current.to_string(),
                available: false,
                latest_version: current.to_string(),
                release_notes: String::new(),
            });
        }
    };

    let available = super::checker::compare_versions(current, &latest) < 0;

    let info = UpdateInfo {
        current_version: current.to_string(),
        available,
        latest_version: latest,
        release_notes: notes,
    };

    let state = app.state::<AppState>();
    let mut updates = state.updates.write().await;
    updates.set_last_check();
    updates.cache_info(info.clone());

    if available && !silent {
        app.emit("event_from_main", serde_json::json!({
            "type": "update-available",
            "payload": info
        })).map_err(|e| e.to_string())?;
    }

    Ok(info)
}

#[tauri::command]
pub async fn install_update_with_progress(app: AppHandle, url: String) -> Result<String, String> {
    let client = reqwest::Client::new();
    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let total = resp.content_length().unwrap_or(0);

    let mut downloaded = 0u64;
    let mut stream = resp.bytes_stream();
    let mut file_content = Vec::new();

    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        file_content.extend_from_slice(&chunk);
        downloaded += chunk.len() as u64;

        if total > 0 {
            let progress = (downloaded as f64 / total as f64 * 100.0) as u32;
            app.emit("event_from_main", serde_json::json!({
                "type": "update-progress",
                "payload": { "progress": progress, "downloaded": downloaded, "total": total }
            })).map_err(|e| e.to_string())?;
        }
    }

    let tmp_dir = std::env::temp_dir();
    let file_name = url.rsplit('/').next().unwrap_or("update.tmp");
    let file_path = tmp_dir.join(file_name);
    std::fs::write(&file_path, &file_content).map_err(|e| e.to_string())?;

    super::installer::install_update(file_path).await
}

#[tauri::command]
pub async fn restart_app(app: AppHandle) -> Result<(), String> {
    app.restart();
}

#[tauri::command]
pub async fn get_update_info(app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    let state = app.state::<AppState>();
    let updates = state.updates.read().await;
    Ok(updates.get_cached().cloned())
}
