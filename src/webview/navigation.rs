use crate::auth::domains::is_auth_url as is_auth_url_strict;

const MAIN_URL: &str = "https://chat.qwen.ai";

pub fn is_allowed(url: &str) -> bool {
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
}

pub fn is_preview_url(url: &str) -> bool {
    let Ok(parsed) = url::Url::parse(url) else {
        return false;
    };
    let Some(host) = parsed.host_str() else {
        return false;
    };
    host == "qwenlm.io" || host.ends_with(".qwenlm.io")
}

pub fn should_allow_inline(url: &str) -> bool {
    if is_allowed(url) || is_preview_url(url) {
        return true;
    }
    url.starts_with("about:") || url.starts_with("data:") || url.starts_with("blob:")
}
