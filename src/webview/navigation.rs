use crate::auth::domains::is_auth_url as is_auth_url_strict;

const MAIN_URL: &str = "https://chat.qwen.ai";

pub fn is_allowed(url: &str) -> bool {
<<<<<<< HEAD
<<<<<<< HEAD
    // 1) chat.qwen.ai sempre permitido (host exato para evitar chat.qwen.ai.evil.com)
    if url.starts_with(MAIN_URL) {
=======
    if url.starts_with(MAIN_URL) {
        // Ensure exact host chat.qwen.ai, not chat.qwen.ai.evil.com
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                if host == "chat.qwen.ai" {
                    return true;
                }
            }
        }
        return false;
    }
<<<<<<< HEAD
    // 2) URLs de auth OAuth (Google/Aliyun) também sempre permitidas
    if is_auth_url_strict(url) {
        return true;
    }
    // 3) Navegação browser-like: permitir qualquer https/http com host válido.
    //    Isso devolve ao WebView a capacidade de navegar em docs, CDN, links markdown etc.
    //    Esquemas perigosos (javascript:, data:, blob:) continuam bloqueados pelo WebKit/CSP.
    if url.starts_with("https://") || url.starts_with("http://") {
        if let Ok(parsed) = url::Url::parse(url) {
            if parsed.host_str().is_some() {
                return true;
            }
        }
    }
    // 4) about:blank e similares usados pelo WebKit internamente
    if url.starts_with("about:") || url.starts_with("data:") {
        return url.starts_with("about:blank");
    }
    false
=======
    url.starts_with(MAIN_URL) || is_auth_url(url)
}

fn is_auth_url(url: &str) -> bool {
    AUTH_DOMAINS.iter().any(|d| url.contains(d)) || AUTH_PATHS.iter().any(|p| url.contains(p))
<<<<<<< HEAD
>>>>>>> c0c2f30 (Fix: Upload medias e username)
=======
    is_auth_url_strict(url)
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
=======
>>>>>>> c0c2f30 (Fix: Upload medias e username)
}
