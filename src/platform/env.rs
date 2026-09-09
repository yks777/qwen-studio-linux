fn is_appimage() -> bool {
    std::env::var("APPIMAGE").is_ok()
        || std::env::current_exe()
            .map(|p| p.to_string_lossy().contains(".AppImage"))
            .unwrap_or(false)
        || std::env::var("APPDIR").is_ok()
}

fn configure_gstreamer_environment() {
    // AppImage com bundleMediaFramework:true já embute ~15-35MB de plugins,
    // mas precisa que o WebKitWebProcess filho herde GST_* corretos.
    // Se o AppImage setou GST_PLUGIN_PATH para dentro do mount e o host tem
    // plugins em /usr/lib/gstreamer-1.0, fazemos forwarding híbrido.

    // 1) Sempre garantir que o scanner aponte para binário válido
    if std::env::var("GST_PLUGIN_SCANNER").is_err() {
        for candidate in [
            "/usr/lib/gstreamer-1.0/gst-plugin-scanner",
            "/usr/libexec/gstreamer-1.0/gst-plugin-scanner",
            "/usr/lib64/gstreamer-1.0/gst-plugin-scanner",
        ] {
            if std::path::Path::new(candidate).exists() {
                std::env::set_var("GST_PLUGIN_SCANNER", candidate);
                break;
            }
        }
    }

    // 2) Se rodando como AppImage, montar GST_PLUGIN_SYSTEM_PATH híbrido
    // que inclui tanto o mount quanto o host, para que appsink/appsrc/autoaudiosink
    // sejam encontrados mesmo quando o bundle não tem todos os plugins.
    if is_appimage() {
        let host_paths = [
            "/usr/lib/gstreamer-1.0",
            "/usr/lib/x86_64-linux-gnu/gstreamer-1.0",
            "/usr/lib64/gstreamer-1.0",
        ];
        let existing = std::env::var("GST_PLUGIN_SYSTEM_PATH").unwrap_or_default();
        let existing_scanner = std::env::var("GST_PLUGIN_PATH").unwrap_or_default();

        // Detecta mount isolado: se GST_PLUGIN_PATH aponta para /tmp/.mount_*
        let is_isolated = existing_scanner.contains("/tmp/.mount_")
            || existing.contains("/tmp/.mount_");

        if is_isolated || existing.is_empty() {
            let mut merged = Vec::new();
            // Preserva o que já existe (mount)
            if !existing_scanner.is_empty() {
                merged.push(existing_scanner.clone());
            }
            if !existing.is_empty() {
                for p in existing.split(':') {
                    if !merged.iter().any(|m| m.contains(p)) {
                        merged.push(p.to_string());
                    }
                }
            }
            // Adiciona host paths que existem
            for hp in host_paths {
                if std::path::Path::new(hp).exists() && !merged.iter().any(|m| m == hp) {
                    merged.push(hp.to_string());
                }
            }
            if !merged.is_empty() {
                let joined = merged.join(":");
                std::env::set_var("GST_PLUGIN_SYSTEM_PATH", &joined);
                // Também espelha em GST_PLUGIN_PATH para WebKit antigo
                if std::env::var("GST_PLUGIN_PATH").is_err()
                    || is_isolated
                {
                    std::env::set_var("GST_PLUGIN_PATH", &joined);
                }
                log::info!("[Env] AppImage GST fix: GST_PLUGIN_SYSTEM_PATH={}", joined);
            }
        }

        // 3) Registro não-fork evita deadlock em alguns drivers Mesa
        if std::env::var("GST_REGISTRY_FORK").is_err() {
            std::env::set_var("GST_REGISTRY_FORK", "no");
        }
    }

    // 4) Log diagnóstico se plugins críticos faltam (não aborta, só avisa)
    let has_appsink = std::path::Path::new("/usr/lib/gstreamer-1.0/libgstapp.so").exists()
        || std::path::Path::new("/usr/lib/x86_64-linux-gnu/gstreamer-1.0/libgstapp.so").exists();
    if !has_appsink {
        log::warn!("[Env] libgstapp.so não encontrado no host — AppImage com bundleMediaFramework deve suprir");
    }
}

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

    // Compositing: por padrão DESABILITADO (fix tela branca / crash DMABUF em Mesa/Arch).
    // QWEN_DISABLE_COMPOSITING=0 força GPU (mais rápido), =1 força CPU (seguro).
    // Default seguro = 1, igual ao que era nos scripts npm originais.
    let disable_compositing = std::env::var("QWEN_DISABLE_COMPOSITING")
        .map(|v| !(v == "0" || v.eq_ignore_ascii_case("false")))
        .unwrap_or(true);
    if disable_compositing {
        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
        log::info!("[Env] WEBKIT_DISABLE_COMPOSITING_MODE=1 (default seguro; use QWEN_DISABLE_COMPOSITING=0 para GPU)");
    } else {
        std::env::remove_var("WEBKIT_DISABLE_COMPOSITING_MODE");
        log::info!("[Env] WEBKIT_DISABLE_COMPOSITING_MODE desabilitado (GPU ativo via QWEN_DISABLE_COMPOSITING=0)");
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

    // GStreamer / AppImage: garante que WebKitWebProcess enxergue plugins do host
    // quando o AppImage monta em /tmp/.mount_* com GST_PLUGIN_PATH isolado.
    configure_gstreamer_environment();

}
