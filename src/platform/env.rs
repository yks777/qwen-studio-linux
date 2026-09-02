pub fn configure_environment() {
    std::env::set_var("GDK_BACKEND", "x11");

    // Compositing: permite override via env var para evitar tela branca em drivers antigos.
    // QWEN_DISABLE_COMPOSITING=1 força CPU (seguro), =0 força GPU (mais rápido).
    // Se não definido, default = GPU off apenas quando necessário (mantém DMABUF off).
    let disable_compositing = std::env::var("QWEN_DISABLE_COMPOSITING")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if disable_compositing {
        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    } else {
        std::env::remove_var("WEBKIT_DISABLE_COMPOSITING_MODE");
    }

    // DMABUF renderer desabilitado por padrão (workaround tela branca WebKitGTK <2.42)
    // Permite override via QWEN_DISABLE_DMABUF_RENDERER=0 para testar GPU path.
    let disable_dmabuf = std::env::var("QWEN_DISABLE_DMABUF_RENDERER")
        .map(|v| !(v == "0" || v.eq_ignore_ascii_case("false")))
        .unwrap_or(true);
    if disable_dmabuf {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    } else {
        std::env::remove_var("WEBKIT_DISABLE_DMABUF_RENDERER");
    }
}
