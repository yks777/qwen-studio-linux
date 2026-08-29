use crate::auth::domains::{AUTH_DOMAINS, AUTH_PATHS};

const MAIN_URL: &str = "https://chat.qwen.ai";

pub fn is_allowed(url: &str) -> bool {
    url.starts_with(MAIN_URL) || is_auth_url(url)
}

fn is_auth_url(url: &str) -> bool {
    AUTH_DOMAINS.iter().any(|d| url.contains(d)) || AUTH_PATHS.iter().any(|p| url.contains(p))
}
