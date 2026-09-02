use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use serde::Serialize;
use tauri::ipc::Response;

#[derive(Serialize)]
pub struct DropMeta {
    pub path: String,
    pub name: String,
    pub mime: String,
    pub size: u64,
}

pub(crate) fn mime_for_path(path: &Path) -> String {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("bmp") => "image/bmp",
        Some("svg") => "image/svg+xml",
        Some("pdf") => "application/pdf",
        Some("txt") => "text/plain",
        Some("md") => "text/markdown",
        Some("csv") => "text/csv",
        Some("json") => "application/json",
        Some("xml") => "application/xml",
        Some("html") => "text/html",
        Some("doc") => "application/msword",
        Some("docx") => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        Some("xls") => "application/vnd.ms-excel",
        Some("xlsx") => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        Some("ppt") => "application/vnd.ms-powerpoint",
        Some("pptx") => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        Some("zip") => "application/zip",
        Some("mp3") => "audio/mpeg",
        Some("mp4") => "video/mp4",
        Some("mov") => "video/quicktime",
        Some("mkv") => "video/x-matroska",
        Some("webm") => "video/webm",
        Some("avi") => "video/x-msvideo",
        Some("m4v") => "video/x-m4v",
        Some("flv") => "video/x-flv",
        Some("mpeg") | Some("mpg") => "video/mpeg",
        Some("wav") => "audio/wav",
        Some("ogg") => "audio/ogg",
        Some("aac") => "audio/aac",
        _ => "application/octet-stream",
    }
    .to_string()
}

#[tauri::command]
pub async fn get_file_metas(paths: Vec<String>) -> Result<Vec<DropMeta>, String> {
    let mut metas = Vec::new();
    for path_str in paths {
        let path = Path::new(&path_str);
        let mime = mime_for_path(path);
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file")
            .to_string();
        match tokio::fs::metadata(path).await {
            Ok(meta) => {
                let size = meta.len();
                metas.push(DropMeta {
                    path: path_str,
                    name,
                    mime,
                    size,
                });
            }
            Err(e) => {
                log::warn!("[drop] metadata failed for {}: {}", path_str, e);
            }
        }
    }
    Ok(metas)
}

<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
const MAX_CHUNK_SIZE: usize = 4 * 1024 * 1024;

#[tauri::command]
pub async fn read_file_chunk(path: String, offset: u64, length: usize) -> Result<Response, String> {
    if length == 0 || length > MAX_CHUNK_SIZE {
        return Err(format!(
            "Invalid length {}: must be 1..{}",
            length, MAX_CHUNK_SIZE
        ));
    }
    if path.contains('\0') {
        return Err("Invalid path".into());
    }
    // Path must be absolute or at least not contain traversal that escapes
    if path.contains("..") {
        // Allow .. only if canonicalization stays within allowed roots; for now reject explicit ..
        // callers are drag-drop paths which are canonical already
        let p = Path::new(&path);
        if p.components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            // We will validate via existence check below, but early reject suspicious patterns
            // is safer; still allow if file exists and is regular file under home/tmp
        }
    }
    let chunk = tauri::async_runtime::spawn_blocking(move || {
        let mut file =
            File::open(&path).map_err(|e| format!("Falha ao abrir '{}': {}", path, e))?;
        let meta = file.metadata().map_err(|e| format!("Falha ao ler metadata: {}", e))?;
        if !meta.is_file() {
            return Err("Not a regular file".into());
        }
        if offset > meta.len() {
            return Err(format!("Offset {} beyond file size {}", offset, meta.len()));
        }
        if offset + length as u64 > meta.len() + 1 {
            // clamp is handled by read, but avoid huge allocation already checked
        }
=======
=======
const MAX_CHUNK_SIZE: usize = 4 * 1024 * 1024;

>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
#[tauri::command]
pub async fn read_file_chunk(path: String, offset: u64, length: usize) -> Result<Response, String> {
    if length == 0 || length > MAX_CHUNK_SIZE {
        return Err(format!(
            "Invalid length {}: must be 1..{}",
            length, MAX_CHUNK_SIZE
        ));
    }
    if path.contains('\0') {
        return Err("Invalid path".into());
    }
    // Path must be absolute or at least not contain traversal that escapes
    if path.contains("..") {
        // Allow .. only if canonicalization stays within allowed roots; for now reject explicit ..
        // callers are drag-drop paths which are canonical already
        let p = Path::new(&path);
        if p.components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            // We will validate via existence check below, but early reject suspicious patterns
            // is safer; still allow if file exists and is regular file under home/tmp
        }
    }
    let chunk = tauri::async_runtime::spawn_blocking(move || {
        let mut file =
            File::open(&path).map_err(|e| format!("Falha ao abrir '{}': {}", path, e))?;
<<<<<<< HEAD
>>>>>>> c0c2f30 (Fix: Upload medias e username)
=======
        let meta = file.metadata().map_err(|e| format!("Falha ao ler metadata: {}", e))?;
        if !meta.is_file() {
            return Err("Not a regular file".into());
        }
        if offset > meta.len() {
            return Err(format!("Offset {} beyond file size {}", offset, meta.len()));
        }
        if offset + length as u64 > meta.len() + 1 {
            // clamp is handled by read, but avoid huge allocation already checked
        }
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
=======
=======
const MAX_CHUNK_SIZE: usize = 4 * 1024 * 1024;

>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
#[tauri::command]
pub async fn read_file_chunk(path: String, offset: u64, length: usize) -> Result<Response, String> {
    if length == 0 || length > MAX_CHUNK_SIZE {
        return Err(format!(
            "Invalid length {}: must be 1..{}",
            length, MAX_CHUNK_SIZE
        ));
    }
    if path.contains('\0') {
        return Err("Invalid path".into());
    }
    // Path must be absolute or at least not contain traversal that escapes
    if path.contains("..") {
        // Allow .. only if canonicalization stays within allowed roots; for now reject explicit ..
        // callers are drag-drop paths which are canonical already
        let p = Path::new(&path);
        if p.components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            // We will validate via existence check below, but early reject suspicious patterns
            // is safer; still allow if file exists and is regular file under home/tmp
        }
    }
    let chunk = tauri::async_runtime::spawn_blocking(move || {
        let mut file =
            File::open(&path).map_err(|e| format!("Falha ao abrir '{}': {}", path, e))?;
<<<<<<< HEAD
>>>>>>> c0c2f30 (Fix: Upload medias e username)
=======
        let meta = file.metadata().map_err(|e| format!("Falha ao ler metadata: {}", e))?;
        if !meta.is_file() {
            return Err("Not a regular file".into());
        }
        if offset > meta.len() {
            return Err(format!("Offset {} beyond file size {}", offset, meta.len()));
        }
        if offset + length as u64 > meta.len() + 1 {
            // clamp is handled by read, but avoid huge allocation already checked
        }
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
        file.seek(SeekFrom::Start(offset))
            .map_err(|e| format!("Falha ao seek offset {}: {}", offset, e))?;
        let mut buffer = vec![0u8; length];
        let bytes_read = file
            .read(&mut buffer)
            .map_err(|e| format!("Falha ao ler chunk: {}", e))?;
        buffer.truncate(bytes_read);
        Ok::<Vec<u8>, String>(buffer)
    })
    .await
    .map_err(|e| format!("Erro no spawn_blocking: {}", e))??;

    Ok(Response::new(chunk))
}
