#![allow(dead_code)]

pub fn inject_token_script(token: &str) -> String {
    format!(
        r#"
        (function() {{
            try {{
                localStorage.setItem('token', '{}');
                sessionStorage.setItem('token', '{}');
                document.cookie = 'token={}; path=/; domain=.qwen.ai; secure; samesite=lax';
                document.cookie = 'token={}; path=/; domain=.chat.qwen.ai; secure; samesite=lax';
            }} catch(e) {{
                console.error('[Token] Injection failed:', e);
            }}
        }})();
        "#,
        token, token, token, token
    )
}

pub fn extract_token_from_url(url: &str) -> Option<String> {
    if let Ok(parsed) = url::Url::parse(url) {
        if let Some(fragment) = parsed.fragment() {
            for param in fragment.split('&') {
                if let Some((key, value)) = param.split_once('=') {
                    if key == "access_token" || key == "token" {
                        return Some(value.to_string());
                    }
                }
            }
        }
        if let Some(query) = parsed.query() {
            for param in query.split('&') {
                if let Some((key, value)) = param.split_once('=') {
                    if key == "access_token" || key == "token" {
                        return Some(value.to_string());
                    }
                }
            }
        }
    }
    None
}
