use tauri_plugin_dialog::DialogExt;

/// Save arbitrary bytes (base64 from JS) using the native "Save As" dialog.
///
/// On Linux this goes through `xdg-desktop-portal` -> the user's default file
/// manager picker (Dolphin on KDE, Nautilus on GNOME, etc), so it works with
/// any explorer as requested.
#[tauri::command]
pub async fn save_downloaded_file(
    app: tauri::AppHandle,
    filename: String,
    mime: Option<String>,
    data_base64: String,
) -> Result<String, String> {
    // ---- validation ----
    if filename.len() > 512 {
        return Err("Filename too long".into());
    }
    if data_base64.len() > 140 * 1024 * 1024 {
        // ~100 MiB decoded (base64 ~33% overhead)
        return Err("File too large (100 MB limit)".into());
    }
    if data_base64.is_empty() {
        return Err("Empty file".into());
    }

    let sanitized = sanitize_filename(&filename);
    let bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        data_base64.trim(),
    )
    .or_else(|_| {
        base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD_NO_PAD,
            data_base64.trim(),
        )
    })
    .map_err(|e| format!("Invalid base64: {e}"))?;

    if bytes.is_empty() {
        return Err("Empty file after decode".into());
    }
    // 100 MiB decoded cap
    if bytes.len() > 100 * 1024 * 1024 {
        return Err("File too large (100 MB limit)".into());
    }

    let ext = extension_from_filename(&sanitized)
        .or_else(|| mime.as_deref().and_then(extension_from_mime))
        .unwrap_or_default();

    let filter_name = filter_name_for_ext(&ext);
    let (tx, rx) = tokio::sync::oneshot::channel();

    {
        let sanitized_clone = sanitized.clone();
        let mut builder = app.dialog().file().set_title("Salvar arquivo");
        if !ext.is_empty() {
            builder = builder.add_filter(filter_name, &[ext.as_str()]);
            // allow all files as second filter
            builder = builder.add_filter("Todos os arquivos", &["*"]);
        }
        builder = builder.set_file_name(&sanitized_clone);
        builder.save_file(move |f| {
            let _ = tx.send(f.and_then(|f| f.as_path().map(|p| p.to_string_lossy().to_string())));
        });
    }

    let path = rx
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No file selected".to_string())?;

    tokio::fs::write(&path, &bytes)
        .await
        .map_err(|e| e.to_string())?;

    Ok(path)
}

fn sanitize_filename(name: &str) -> String {
    // Keep basename only, strip path separators
    let base = std::path::Path::new(name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(name);
    let mut s: String = base
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, '-' | '_' | '.' | ' ' | '(' | ')' | '[' | ']') {
                c
            } else {
                '_'
            }
        })
        .collect();
    s = s.trim().to_string();
    if s.is_empty() {
        s = "download".to_string();
    }
    if s.len() > 200 {
        // preserve extension
        if let Some(dot) = s.rfind('.') {
            let ext = s[dot..].to_string();
            s.truncate(200 - ext.len());
            s.push_str(&ext);
        } else {
            s.truncate(200);
        }
    }
    if s.starts_with('.') {
        s = format!("download{s}");
    }
    s
}

fn extension_from_filename(name: &str) -> Option<String> {
    std::path::Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .filter(|e| !e.is_empty() && e.len() <= 10 && e.chars().all(|c| c.is_ascii_alphanumeric()))
}

fn extension_from_mime(mime: &str) -> Option<String> {
    let m = mime.to_ascii_lowercase();
    let m = m.split(';').next().unwrap_or(&m).trim();
    let ext = match m {
        "image/png" => "png",
        "image/jpeg" | "image/jpg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/svg+xml" => "svg",
        "application/pdf" => "pdf",
        "text/plain" => "txt",
        "text/csv" => "csv",
        "text/markdown" | "text/x-markdown" => "md",
        "text/html" => "html",
        "application/json" => "json",
        "application/zip" => "zip",
        "application/x-zip-compressed" => "zip",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => "docx",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => "xlsx",
        "application/vnd.ms-excel" => "xls",
        "application/msword" => "doc",
        _ => return None,
    };
    Some(ext.to_string())
}

fn filter_name_for_ext(ext: &str) -> String {
    match ext {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" => "Imagem",
        "pdf" => "PDF",
        "txt" | "csv" | "md" => "Texto",
        "html" => "HTML",
        "json" => "JSON",
        "zip" => "ZIP",
        "docx" | "doc" => "Documento",
        "xlsx" | "xls" => "Planilha",
        _ => "Arquivo",
    }
    .to_string()
}
