pub fn configure_environment() {
    // KDE Plasma: força x11 (XWayland) incondicional para wry/GTK exibir GtkMenuBar clássico.
    // Igual ao que funcionava em qwen-studio-linux-fix-error/src/platform/env.rs:2 (set_var x11).
    // Opt-in Wayland nativo só via QWEN_USE_WAYLAND=1 ou QWEN_FORCE_WAYLAND=1.
    let use_wayland = std::env::var("QWEN_USE_WAYLAND")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
        || std::env::var("QWEN_FORCE_WAYLAND")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
    let current = std::env::var("GDK_BACKEND").unwrap_or_default();
    let has_wayland = std::env::var("WAYLAND_DISPLAY").is_ok();
    if use_wayland && has_wayland {
        std::env::set_var("GDK_BACKEND", "wayland");
        log::info!("[Env] QWEN_USE_WAYLAND=1, forçando GDK_BACKEND=wayland (menu pode sumir no Plasma sem appmenu-gtk-module)");
    } else {
        if current == "wayland" {
            log::warn!(
                "[Env] GDK_BACKEND=wayland detectado, sobrescrevendo para x11 para exibir menu (use QWEN_USE_WAYLAND=1 para manter wayland)"
            );
        }
        std::env::set_var("GDK_BACKEND", "x11");
        if has_wayland {
            log::info!("[Env] Wayland detectado, GDK_BACKEND=x11 forçado (fix-error compat)");
        }
    }

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
