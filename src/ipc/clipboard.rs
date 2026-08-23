#[cfg(target_os = "linux")]
#[tauri::command]
pub async fn read_clipboard_image() -> Result<String, String> {
    use base64::Engine;
    let (tx, rx) = std::sync::mpsc::channel::<Result<Vec<u8>, String>>();

    glib::MainContext::default().invoke(move || {
        let result = (|| -> Result<Vec<u8>, String> {
            let clipboard = gtk::Clipboard::get(&gtk::gdk::Atom::intern("CLIPBOARD"));
            if !clipboard.wait_is_image_available() {
                return Err("No image in clipboard".into());
            }
            let pixbuf = clipboard.wait_for_image()
                .ok_or_else(|| "No image in clipboard".to_string())?;
            pixbuf.save_to_bufferv("png", &[])
                .map_err(|e| format!("PNG save failed: {}", e))
        })();
        let _ = tx.send(result);
    });

    match tokio::time::timeout(
        std::time::Duration::from_secs(2),
        tokio::task::spawn_blocking(move || rx.recv().map_err(|e| e.to_string())?),
    ).await {
        Ok(Ok(Ok(bytes))) => Ok(base64::engine::general_purpose::STANDARD.encode(&bytes)),
        Ok(Ok(Err(e))) => Err(e),
        Ok(Err(e)) => Err(format!("Task join error: {}", e)),
        Err(_) => Err("Timeout reading clipboard image".into()),
    }
}

#[cfg(not(target_os = "linux"))]
#[tauri::command]
pub async fn read_clipboard_image() -> Result<String, String> {
    Err("Only available on Linux".into())
}
