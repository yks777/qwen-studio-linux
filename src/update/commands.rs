use std::sync::atomic::{AtomicBool, Ordering};

use crate::app::state::AppState;
use crate::config::schema::UpdateInfo;
use tauri::{AppHandle, Emitter, Manager};

/// Guards against concurrent install runs (the update can be triggered from
/// multiple windows at once, e.g. every profile webview).
static INSTALLING: AtomicBool = AtomicBool::new(false);
static LAST_MANUAL_CHECK: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[tauri::command]
pub async fn check_for_updates(app: AppHandle, silent: bool) -> Result<UpdateInfo, String> {
    // Debounce manual checks to avoid GitHub API spam via menu (economia de rede/CPU)
    if !silent {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let last = LAST_MANUAL_CHECK.load(Ordering::Relaxed);
        if now.saturating_sub(last) < 60 && last != 0 {
            let state = app.state::<AppState>();
            let updates = state.updates.read().await;
            if let Some(cached) = updates.get_cached().cloned() {
                return Ok(cached);
            }
        }
        LAST_MANUAL_CHECK.store(now, Ordering::Relaxed);
    }
    let current = crate::config::schema::APP_VERSION;

    let (latest, notes, download_url) = match super::checker::fetch_latest_version().await {
        Ok(v) => v,
        Err(e) => {
            log::warn!("[Update] Check failed: {}", e);
            return Ok(UpdateInfo {
                current_version: current.to_string(),
                available: false,
                latest_version: current.to_string(),
                release_notes: String::new(),
                download_url: None,
            });
        }
    };

    let available = super::checker::compare_versions(current, &latest) < 0;
    let info = UpdateInfo {
        current_version: current.to_string(),
        available,
        latest_version: latest,
        release_notes: notes,
        download_url,
    };

    let state = app.state::<AppState>();
    let mut updates = state.updates.write().await;
    updates.set_last_check();
    updates.cache_info(info.clone());

    if available && !silent {
        app.emit(
            "event_from_main",
            serde_json::json!({
                "type": "update-available",
                "payload": info
            }),
        )
        .map_err(|e| e.to_string())?;

<<<<<<< HEAD
<<<<<<< HEAD
        // No auto-install: user must explicitly click "Install" in Updates tab
=======
        // Self-update: download + install automatically in the background.
        // Triggered from Rust (single source) to avoid parallel downloads
        // across the multiple webviews that inject this script.
        if let Some(url) = install_url {
            let app2 = app.clone();
            tauri::async_runtime::spawn(async move {
                match install_update_with_progress(app2.clone(), url).await {
                    Ok(s) if s != "already-updating" => {
                        let _ = app2.emit(
                            "event_from_main",
                            serde_json::json!({
                                "type": "update-installed"
                            }),
                        );
                    }
                    _ => {}
                }
            });
        }
>>>>>>> c0c2f30 (Fix: Upload medias e username)
=======
        // No auto-install: user must explicitly click "Install" in Updates tab
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
    }

    Ok(info)
}

#[tauri::command]
pub async fn install_update_with_progress(app: AppHandle, url: String) -> Result<String, String> {
    if INSTALLING.swap(true, Ordering::SeqCst) {
        return Ok("already-updating".to_string());
    }

    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            INSTALLING.store(false, Ordering::SeqCst);
        }
    }
    let _guard = Guard;

    // Sanitize url
    let parsed = url::Url::parse(&url).map_err(|e| e.to_string())?;
    if parsed.scheme() != "https" {
        return Err("Only https url allowed".into());
    }

<<<<<<< HEAD
    let client = super::checker::HTTP_CLIENT.clone();
    let resp = client
        .get(url.clone())
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .map_err(|e| e.to_string())?;
=======
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client.get(url.clone()).send().await.map_err(|e| e.to_string())?;
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
    if !resp.status().is_success() {
        return Err(format!("Download failed: {}", resp.status()));
    }
    let total = resp.content_length().unwrap_or(0);
    if total > 500 * 1024 * 1024 {
        return Err("File too large".into());
    }

    let tmp_dir = std::env::temp_dir();
    let raw_name = url.rsplit('/').next().unwrap_or("update.tmp");
    // sanitize file name: only alphanumeric + . - _
    let file_name: String = raw_name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' })
        .collect();
    let file_name = if file_name.is_empty() { "update.tmp".to_string() } else { file_name };
    let file_path = tmp_dir.join(&file_name);
    let mut file = tokio::fs::File::create(&file_path).await.map_err(|e| e.to_string())?;

    let mut downloaded = 0u64;
    let mut stream = resp.bytes_stream();
    let mut last_emit = tokio::time::Instant::now();
    let mut last_progress: u32 = 0;

    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        // Hard cap incremental
        if downloaded + chunk.len() as u64 > 500 * 1024 * 1024 {
            return Err("File too large during download".into());
        }
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;

        if total > 0 {
            let progress = (downloaded as f64 / total as f64 * 100.0) as u32;
<<<<<<< HEAD
            // throttle to 1000ms (economia de wakes: 5/s → 1/s por janela)
            if progress != last_progress
                && last_emit.elapsed() >= std::time::Duration::from_millis(1000)
=======
            // throttle to 200ms or 1% change
            if progress != last_progress
                && last_emit.elapsed() >= std::time::Duration::from_millis(200)
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
            {
                last_progress = progress;
                last_emit = tokio::time::Instant::now();
                app.emit("event_from_main", serde_json::json!({
                    "type": "update-progress",
                    "payload": { "progress": progress, "downloaded": downloaded, "total": total }
                }))
                .map_err(|e| e.to_string())?;
            }
        }
<<<<<<< HEAD
    }
    file.flush().await.map_err(|e| e.to_string())?;
    drop(file);

<<<<<<< HEAD
    let result = super::installer::install_update(file_path).await;
=======
        let tmp_dir = std::env::temp_dir();
        let file_name = url.rsplit('/').next().unwrap_or("update.tmp");
        let file_path = tmp_dir.join(file_name);
        std::fs::write(&file_path, &file_content).map_err(|e| e.to_string())?;

        super::installer::install_update(file_path).await
=======
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
    }
    file.flush().await.map_err(|e| e.to_string())?;
    drop(file);

<<<<<<< HEAD
    INSTALLING.store(false, Ordering::SeqCst);
>>>>>>> c0c2f30 (Fix: Upload medias e username)
=======
    let result = super::installer::install_update(file_path).await;
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
    result
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
