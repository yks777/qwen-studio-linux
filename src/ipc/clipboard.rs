#[cfg(target_os = "linux")]
#[tauri::command]
pub async fn read_clipboard_image(app: tauri::AppHandle) -> Result<String, String> {
    use base64::Engine;
    use image::codecs::png::PngEncoder;
    use image::ColorType;
    use image::ImageEncoder;
    use log::warn;
    use tauri_plugin_clipboard_manager::ClipboardExt;

    // `read_image` must NOT run on the main thread or it can deadlock on Linux.
    let result = tauri::async_runtime::spawn_blocking(move || {
        // Prefer the Tauri clipboard plugin; fall back to arboard when it
        // fails (some X11/Wayland configs return a transient error).
        let (width, height, rgba) = match app.clipboard().read_image() {
            Ok(image) => (image.width(), image.height(), image.rgba().to_vec()),
            Err(_) => {
                let mut clipboard = arboard::Clipboard::new()
                    .map_err(|e| format!("arboard init failed: {}", e))?;
                let img = clipboard
                    .get_image()
                    .map_err(|e| format!("arboard read failed: {}", e))?;
                if img.bytes.is_empty() {
                    warn!("arboard returned an empty image — check wl-clipboard on Wayland / X11 clipboard access");
                }
                (img.width as u32, img.height as u32, img.bytes.to_vec())
            }
        };

        // Clipboard sem imagem (0×0 ou bytes vazios): retorna erro para que o
        // JS desça ao fallback de texto em vez de injetar um PNG fantasma.
        if width == 0 || height == 0 || rgba.is_empty() {
            return Err("no image in clipboard".into());
        }

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
        }
        Ok::<Vec<u8>, String>(png)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;

    Ok(base64::engine::general_purpose::STANDARD.encode(&result))
}

/// Lê a imagem do clipboard e salva em /tmp para o fluxo explícito.
/// Usado quando WebKit <2.50.2 não expõe image/* no paste: a injeção via
/// input `change` é ignorada pelo site, então salvamos o PNG e orientamos
/// o usuário a arrastar/anexar manualmente.
#[cfg(target_os = "linux")]
#[tauri::command]
pub async fn save_clipboard_image_to_file(app: tauri::AppHandle) -> Result<String, String> {
    use base64::Engine;
    // Reusa a mesma lógica de leitura (evita duplicar spawn_blocking)
    let b64 = read_clipboard_image(app).await?;
    let png = base64::engine::general_purpose::STANDARD
        .decode(&b64)
        .map_err(|e| format!("base64 decode failed: {}", e))?;
    let mut path = std::env::temp_dir();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    path.push(format!("qwen-clipboard-{}.png", ts));
    std::fs::write(&path, &png).map_err(|e| format!("write temp file failed: {}", e))?;
    path.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "invalid temp path".to_string())
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
