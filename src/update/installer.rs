use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum BundleType {
    AppImage,
    Deb,
    Rpm,
    Unknown,
}

pub fn detect_bundle_type() -> BundleType {
    if let Ok(exe) = std::env::current_exe() {
        let path = exe.to_string_lossy().to_lowercase();
        if path.contains("appimage") || std::env::var("APPIMAGE").is_ok() {
            return BundleType::AppImage;
        }
    }

    if let Ok(output) = std::process::Command::new("dpkg")
        .arg("--status")
        .arg("qwen-studio-linux")
        .output()
    {
        if output.status.success() {
            return BundleType::Deb;
        }
    }

    if let Ok(output) = std::process::Command::new("rpm")
        .arg("-q")
        .arg("qwen-studio-linux")
        .output()
    {
        if output.status.success() {
            return BundleType::Rpm;
        }
    }

    BundleType::Unknown
}

pub async fn install_update(file_path: PathBuf) -> Result<String, String> {
    let bundle = detect_bundle_type();

    match bundle {
        BundleType::Deb => {
            let status = std::process::Command::new("pkexec")
                .args(["dpkg", "-i", &file_path.to_string_lossy()])
                .status()
                .map_err(|e| e.to_string())?;
            if status.success() {
                Ok("Installation complete".into())
            } else {
                Err("Installation failed".into())
            }
        }
        BundleType::Rpm => {
            let status = std::process::Command::new("pkexec")
                .args(["rpm", "-Uvh", &file_path.to_string_lossy()])
                .status()
                .map_err(|e| e.to_string())?;
            if status.success() {
                Ok("Installation complete".into())
            } else {
                Err("Installation failed".into())
            }
        }
        BundleType::AppImage => {
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let backup = exe.with_extension("old");
            std::fs::rename(&exe, &backup).map_err(|e| e.to_string())?;
            std::fs::copy(&file_path, &exe).map_err(|e| e.to_string())?;
            std::fs::set_permissions(&exe, std::fs::Permissions::from_mode(0o755))
                .map_err(|e| e.to_string())?;
            Ok("AppImage updated. Please restart.".into())
        }
        BundleType::Unknown => {
            Err("Unknown bundle type. Please install manually.".into())
        }
    }
}
