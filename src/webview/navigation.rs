use crate::auth::domains::is_auth_url as is_auth_url_strict;

const MAIN_URL: &str = "https://chat.qwen.ai";

pub fn is_allowed(url: &str) -> bool {
<<<<<<< HEAD
    if url.starts_with(MAIN_URL) {
        // Ensure exact host chat.qwen.ai, not chat.qwen.ai.evil.com
        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                if host == "chat.qwen.ai" {
                    return true;
                }
            }
        }
        return false;
    }
    is_auth_url_strict(url)
=======
    url.starts_with(MAIN_URL) || is_auth_url(url)
}

fn is_auth_url(url: &str) -> bool {
    AUTH_DOMAINS.iter().any(|d| url.contains(d)) || AUTH_PATHS.iter().any(|p| url.contains(p))
>>>>>>> bec7255 (Fix: Upload medias e username)
}
