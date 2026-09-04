#[cfg(target_os = "linux")]
fn read_clipboard_png_inner(app: tauri::AppHandle) -> Result<Vec<u8>, String> {
    use image::codecs::png::PngEncoder;
    use image::ColorType;
    use image::ImageEncoder;
    use log::warn;
    use tauri_plugin_clipboard_manager::ClipboardExt;

    let (width, height, rgba) = match app.clipboard().read_image() {
        Ok(image) => (image.width(), image.height(), image.rgba().to_vec()),
        Err(_) => {
            let mut clipboard =
                arboard::Clipboard::new().map_err(|e| format!("arboard init failed: {}", e))?;
            let img = clipboard
                .get_image()
                .map_err(|e| format!("arboard read failed: {}", e))?;
            if img.bytes.is_empty() {
                warn!("arboard returned an empty image — check wl-clipboard on Wayland / X11 clipboard access");
            }
            (img.width as u32, img.height as u32, img.bytes.to_vec())
        }
    };
    if width == 0 || height == 0 || rgba.is_empty() {
        return Err("no image in clipboard".into());
    }
    if rgba.len() > 16 * 1024 * 1024 || (width as u64) * (height as u64) > 3840 * 2160 {
        return Err("clipboard image too large".into());
    }
    let mut png = Vec::new();
    let encoder = PngEncoder::new(&mut png);
    encoder
        .write_image(&rgba, width, height, ColorType::Rgba8.into())
        .map_err(|e| format!("PNG encode failed: {}", e))?;
    Ok(png)
}

<<<<<<< HEAD
        // Limite anti-freeze: evita travar ao ler screenshot 4K gigante
        if rgba.len() > 16 * 1024 * 1024 || (width as u64) * (height as u64) > 3840 * 2160 {
            return Err("clipboard image too large".into());
        }

        let mut png = Vec::new();
        {
            let encoder = PngEncoder::new(&mut png);
            encoder
                .write_image(&rgba, width, height, ColorType::Rgba8.into())
                .map_err(|e| format!("PNG encode failed: {}", e))?;
=======
#[cfg(target_os = "linux")]
#[tauri::command]
pub async fn read_clipboard_image(app: tauri::AppHandle) -> Result<String, String> {
    use base64::Engine;
    use std::sync::{Mutex, OnceLock};
    use std::time::{Duration, Instant};

    static CACHE: OnceLock<Mutex<Option<(Instant, String)>>> = OnceLock::new();
    if let Some(m) = CACHE.get().and_then(|m| m.lock().ok()) {
        if let Some((ts, cached)) = m.as_ref() {
            if ts.elapsed() < Duration::from_millis(500) {
                return Ok(cached.clone());
            }
>>>>>>> dev
        }
    }

    let png = tauri::async_runtime::spawn_blocking({
        let app = app.clone();
        move || read_clipboard_png_inner(app)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;

    let b64 = base64::engine::general_purpose::STANDARD.encode(&png);
    if let Ok(mut g) = CACHE.get_or_init(|| Mutex::new(None)).lock() {
        *g = Some((Instant::now(), b64.clone()));
    }
    Ok(b64)
}

/// Lê a imagem do clipboard e salva em /tmp para o fluxo explícito.
/// Usado quando WebKit <2.50.2 não expõe image/* no paste: a injeção via
/// input `change` é ignorada pelo site, então salvamos o PNG e orientamos
/// o usuário a arrastar/anexar manualmente.
#[cfg(target_os = "linux")]
#[tauri::command]
pub async fn save_clipboard_image_to_file(app: tauri::AppHandle) -> Result<String, String> {
<<<<<<< HEAD
    // Directly get PNG bytes without base64 round-trip
    let png = read_clipboard_image_bytes(app).await?;
    let mut path = std::env::temp_dir();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let pid = std::process::id();
    path.push(format!("qwen-clipboard-{}-{}.png", pid, ts));
    // Restrict permissions to 0o600 where supported
    std::fs::write(&path, &png).map_err(|e| format!("write temp file failed: {}", e))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }
    path.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "invalid temp path".to_string())
=======
    let png = read_clipboard_image_bytes(app).await?;
    let path = tauri::async_runtime::spawn_blocking(move || -> Result<String, String> {
        let mut path = std::env::temp_dir();
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let pid = std::process::id();
        path.push(format!("qwen-clipboard-{}-{}.png", pid, ts));
        std::fs::write(&path, &png).map_err(|e| format!("write temp file failed: {}", e))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
        }
        path.to_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "invalid temp path".to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;
    Ok(path)
}

#[cfg(target_os = "linux")]
async fn read_clipboard_image_bytes(app: tauri::AppHandle) -> Result<Vec<u8>, String> {
    tauri::async_runtime::spawn_blocking(move || read_clipboard_png_inner(app))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
>>>>>>> dev
}

#[cfg(target_os = "linux")]
async fn read_clipboard_image_bytes(app: tauri::AppHandle) -> Result<Vec<u8>, String> {
    use image::codecs::png::PngEncoder;
    use image::ColorType;
    use image::ImageEncoder;
    use log::warn;
    use tauri_plugin_clipboard_manager::ClipboardExt;

    tauri::async_runtime::spawn_blocking(move || {
        let (width, height, rgba) = match app.clipboard().read_image() {
            Ok(image) => (image.width(), image.height(), image.rgba().to_vec()),
            Err(_) => {
                let mut clipboard = arboard::Clipboard::new()
                    .map_err(|e| format!("arboard init failed: {}", e))?;
                let img = clipboard
                    .get_image()
                    .map_err(|e| format!("arboard read failed: {}", e))?;
                if img.bytes.is_empty() {
                    warn!("arboard returned an empty image");
                }
                (img.width as u32, img.height as u32, img.bytes.to_vec())
            }
        };
        if width == 0 || height == 0 || rgba.is_empty() {
            return Err("no image in clipboard".into());
        }
        if rgba.len() > 16 * 1024 * 1024 || (width as u64) * (height as u64) > 3840 * 2160 {
            return Err("clipboard image too large".into());
        }
        let mut png = Vec::new();
        let encoder = PngEncoder::new(&mut png);
        encoder
            .write_image(&rgba, width, height, ColorType::Rgba8.into())
            .map_err(|e| format!("PNG encode failed: {}", e))?;
        Ok::<Vec<u8>, String>(png)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[cfg(not(target_os = "linux"))]
#[tauri::command]
pub async fn read_clipboard_image(_app: tauri::AppHandle) -> Result<String, String> {
    Err("Only available on Linux".into())
}

#[cfg(not(target_os = "linux"))]
#[tauri::command]
pub async fn save_clipboard_image_to_file(_app: tauri::AppHandle) -> Result<String, String> {
    Err("Only available on Linux".into())
}
