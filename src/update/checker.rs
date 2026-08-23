pub fn compare_versions(current: &str, latest: &str) -> i32 {
    let current_parts: Vec<u32> = current.split('.')
        .filter_map(|p| p.parse().ok())
        .collect();
    let latest_parts: Vec<u32> = latest.split('.')
        .filter_map(|p| p.parse().ok())
        .collect();

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

pub async fn fetch_latest_version() -> Result<(String, String), String> {
    let url = "https://api.github.com/repos/NicolasToledoo/qwen-studio-linux/releases/latest";
    let client = reqwest::Client::new();
    let resp = client.get(url)
        .header("User-Agent", "qwen-studio-linux")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    let tag = json.get("tag_name")
        .and_then(|v| v.as_str())
        .unwrap_or("0.0.0")
        .trim_start_matches('v')
        .to_string();

    let notes = json.get("body")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok((tag, notes))
}
