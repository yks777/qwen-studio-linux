pub fn compare_versions(current: &str, latest: &str) -> i32 {
    let current_parts: Vec<u32> = current.split('.').filter_map(|p| p.parse().ok()).collect();
    let latest_parts: Vec<u32> = latest.split('.').filter_map(|p| p.parse().ok()).collect();

    let max_len = current_parts.len().max(latest_parts.len());
    for i in 0..max_len {
        let c = current_parts.get(i).unwrap_or(&0);
        let l = latest_parts.get(i).unwrap_or(&0);
        if c < l {
            return -1;
        }
        if c > l {
            return 1;
        }
    }
    0
}

use std::sync::LazyLock;
use std::time::Duration;

<<<<<<< HEAD
<<<<<<< HEAD
pub static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
=======
static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
=======
static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
    reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .connect_timeout(Duration::from_secs(5))
        .user_agent("qwen-studio-linux")
<<<<<<< HEAD
<<<<<<< HEAD
        .pool_idle_timeout(Duration::from_secs(5))
=======
        .pool_idle_timeout(Duration::from_secs(30))
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
=======
        .pool_idle_timeout(Duration::from_secs(30))
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
        .build()
        .expect("http client")
});

pub async fn fetch_latest_version() -> Result<(String, String, Option<String>), String> {
    let url = "https://api.github.com/repos/yks777/qwen-studio-linux/releases/latest";
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
    let resp = HTTP_CLIENT
=======
    let client = reqwest::Client::new();
    let resp = client
>>>>>>> c0c2f30 (Fix: Upload medias e username)
=======
    let resp = HTTP_CLIENT
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
=======
    let client = reqwest::Client::new();
    let resp = client
>>>>>>> c0c2f30 (Fix: Upload medias e username)
=======
    let resp = HTTP_CLIENT
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
        .get(url)
        .header("User-Agent", "qwen-studio-linux")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("GitHub API status {}", resp.status()));
    }
    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    let tag = json
        .get("tag_name")
        .and_then(|v| v.as_str())
        .unwrap_or("0.0.0")
        .trim_start_matches('v')
        .to_string();

    let notes = json
        .get("body")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let download_url = json
        .get("assets")
        .and_then(|a| a.as_array())
        .and_then(|assets| {
            assets
                .iter()
                .filter_map(|a| {
                    a.get("name")
                        .and_then(|n| n.as_str())
                        .zip(a.get("browser_download_url").and_then(|u| u.as_str()))
                })
                .find(|(name, _)| {
                    let n = name.to_lowercase();
                    // Prefer the x86_64/amd64 AppImage when multiple arch assets exist.
                    n.ends_with(".appimage")
                        && (n.contains("x86_64") || n.contains("amd64") || !n.contains("arm"))
                })
                .or_else(|| {
                    assets
                        .iter()
                        .filter_map(|a| {
                            a.get("name")
                                .and_then(|n| n.as_str())
                                .zip(a.get("browser_download_url").and_then(|u| u.as_str()))
                        })
                        .find(|(name, _)| name.to_lowercase().ends_with(".appimage"))
                })
                .map(|(_, url)| url.to_string())
        });

    Ok((tag, notes, download_url))
}
