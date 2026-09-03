use crate::auth::domains::is_auth_url as is_auth_url_strict;

const MAIN_URL: &str = "https://chat.qwen.ai";

pub fn is_allowed(url: &str) -> bool {
    // 1) chat.qwen.ai sempre permitido (host exato para evitar chat.qwen.ai.evil.com)
    if url.starts_with(MAIN_URL) {
        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                if host == "chat.qwen.ai" {
                    return true;
                }
            }
        }
        return false;
    }
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
}
