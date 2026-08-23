use std::fs;
use std::path::Path;

fn copy_dir(src: &Path, dst: &Path) {
    let _ = fs::create_dir_all(dst);
    if let Ok(entries) = fs::read_dir(src) {
        for entry in entries.flatten() {
            let path = entry.path();
            let target = dst.join(entry.file_name());
            if path.is_dir() {
                copy_dir(&path, &target);
            } else if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                if ext == "html" || ext == "css" || ext == "js" || ext == "json" {
                    let _ = fs::copy(&path, &target);
                }
            }
        }
    }
}

fn main() {
    let src = Path::new("src/profile_picker");
    let dst = Path::new("dist/profile-picker");
    if src.exists() {
        copy_dir(src, dst);
    }
    tauri_build::build()
}
