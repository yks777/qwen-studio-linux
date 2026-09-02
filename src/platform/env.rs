pub fn configure_environment() {
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
=======
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
=======
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
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
<<<<<<< HEAD
<<<<<<< HEAD

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
=======
=======
>>>>>>> ce2f600 (optimization)
    // Se o usuário forçou software, aplica o workaround independente do backend.
    if force_software_rendering() {
        eprintln!("[env] Renderização por software FORÇADA via settings.json");
        env::set_var("GDK_BACKEND", "x11");
        env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
        env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        return;
    }

    let backend = detect_display_backend();
    eprintln!("[env] Backend gráfico detectado: {:?}", backend);

    match backend {
        DisplayBackend::Wayland => {
            // Wayland nativo → libera aceleração GPU (compositing + DMABUF).
            env::remove_var("GDK_BACKEND");
            env::remove_var("WEBKIT_DISABLE_COMPOSITING_MODE");
            env::remove_var("WEBKIT_DISABLE_DMABUF_RENDERER");
        }
        // X11 / XWayland / desconhecido → mantém workaround de software
        // que evita a tela em branco em alguns drivers WebKitGTK.
        _ => {
            env::set_var("GDK_BACKEND", "x11");
            env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
            env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
<<<<<<< HEAD
>>>>>>> ce2f600 (optimization)
    }
=======
    std::env::set_var("GDK_BACKEND", "x11");
<<<<<<< HEAD
    std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
>>>>>>> 5877c22 (restore)
=======
=======
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
=======
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)

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
>>>>>>> 0f81055 (Melhorias)
=======
    }
>>>>>>> ce2f600 (optimization)
=======
    std::env::set_var("GDK_BACKEND", "x11");
<<<<<<< HEAD
    std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
>>>>>>> 5877c22 (restore)
=======

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
>>>>>>> 0f81055 (Melhorias)
}
