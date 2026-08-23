pub const AUTH_DOMAINS: &[&str] = &[
    "accounts.google.com",
    "auth0.aliyun.com",
    "passport.aliyun.com",
    "qwen.ai",
    "chat.qwen.ai",
    "www.qwen.ai",
    "alibaba.com",
    "aliyun.com",
    "alibabacloud.com",
    "tongyi.aliyun.com",
    "dashscope.aliyun.com",
    "accounts.google.com",
    "oauth2.googleapis.com",
];

pub const AUTH_PATHS: &[&str] = &[
    "/oauth",
    "/auth",
    "/login",
    "/callback",
    "/token",
    "/openid",
];

pub fn is_auth_url(url: &str) -> bool {
    AUTH_DOMAINS.iter().any(|d| url.contains(d))
        || AUTH_PATHS.iter().any(|p| url.contains(p))
}
