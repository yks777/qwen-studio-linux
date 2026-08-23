use std::path::PathBuf;

pub fn config_dir() -> PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("qwen-studio-linux");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

pub fn settings_file() -> PathBuf {
    config_dir().join("settings.json")
}

#[allow(dead_code)]
pub fn data_dir() -> PathBuf {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("qwen-studio-linux");
    let _ = std::fs::create_dir_all(&dir);
    dir
}
