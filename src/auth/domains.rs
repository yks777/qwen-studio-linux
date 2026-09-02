pub const AUTH_DOMAINS: &[&str] = &[
    "accounts.google.com",
    "oauth2.googleapis.com",
    "auth0.aliyun.com",
    "passport.aliyun.com",
    "qwen.ai",
    "chat.qwen.ai",
    "www.qwen.ai",
    "tongyi.aliyun.com",
    "dashscope.aliyun.com",
];

pub const AUTH_PATHS: &[&str] = &[
    "/oauth",
    "/auth",
    "/login",
    "/callback",
    "/token",
    "/openid",
];

fn host_is_allowed(host: &str) -> bool {
    AUTH_DOMAINS.iter().any(|d| host == *d || host.ends_with(&format!(".{}", d)))
}

fn path_is_auth(path: &str) -> bool {
    // Match auth-related path segments exactly, not substring of random host query
    let lower = path.to_ascii_lowercase();
    AUTH_PATHS.iter().any(|p| {
        lower == *p
            || lower.starts_with(&format!("{}/", p))
            || lower.starts_with(&format!("{}?", p))
            || lower.contains(p)
                && (lower.contains("/oauth")
                    || lower.contains("/callback")
                    || lower.contains("/login"))
    })
}

pub fn is_auth_url(url: &str) -> bool {
    let Ok(parsed) = url::Url::parse(url) else {
        return false;
    };
    let Some(host) = parsed.host_str() else {
        return false;
    };
    let host_allowed = host_is_allowed(host);
    if !host_allowed {
        return false;
    }
    // For known auth hosts, allow any path; for qwen.ai family, require auth-like path or exact host
    if host == "accounts.google.com" || host == "oauth2.googleapis.com" {
        return true;
    }
    // For aliyun/qwen domains, also check path looks like auth flow to reduce overly broad allow
    // but still permit OAuth redirects which often use /login, /oauth, /callback
    let path = parsed.path().to_ascii_lowercase();
    // Allow if host is qwen/aliyun and path is auth-like, or host is exact qwen.ai
    if host.ends_with("qwen.ai") || host.ends_with("aliyun.com") {
        // Require auth path for broad domains to avoid https://evil.qwen.ai.evil.com bypass already handled by host check
        // Here host check already ensures suffix, so we allow but still validate path contains auth hint
        // To avoid breaking legitimate non-login qwen pages, allow chat.qwen.ai always (handled in navigation.rs)
        // For other qwen/aliyun hosts, require auth path
        if host == "qwen.ai" || host == "www.qwen.ai" {
            return true;
        }
        return path_is_auth(&path) || path == "/" || path.is_empty();
    }
    true
}
