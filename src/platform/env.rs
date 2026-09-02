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

    // DMABUF: auto-detect WebKit >=2.42 usa GPU (economia CPU/bateria), <2.42 mantém CPU fallback para evitar tela branca.
    // Override via QWEN_DISABLE_DMABUF_RENDERER=1 força CPU, =0 força GPU.
    let disable_dmabuf = match std::env::var("QWEN_DISABLE_DMABUF_RENDERER") {
        Ok(v) => !(v == "0" || v.eq_ignore_ascii_case("false")),
        Err(_) => {
            // Sem override explícito: auto-detect baseado na versão WebKit em runtime
            // Wayland nativo pode ter issues com DMABUF, mantém fallback se wayland forçado
            if use_wayland && has_wayland {
                true
            } else {
                let (major, minor) = unsafe {
                    (
                        webkit2gtk::ffi::webkit_get_major_version(),
                        webkit2gtk::ffi::webkit_get_minor_version(),
                    )
                };
                // WebKit >=2.42 tem fix para DMABUF blank screen
                !(major > 2 || (major == 2 && minor >= 42))
            }
        }
    };
    if disable_dmabuf {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        log::info!("[Env] DMABUF renderer desabilitado (CPU fallback)");
    } else {
        std::env::remove_var("WEBKIT_DISABLE_DMABUF_RENDERER");
        log::info!("[Env] DMABUF renderer habilitado (GPU)");
    }
}
