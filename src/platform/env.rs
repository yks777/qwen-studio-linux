use std::env;

/// Backend gráfico detectado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayBackend {
    /// Wayland nativo (melhor performance, GPU disponível).
    Wayland,
    /// Sessão X11 pura.
    X11,
    /// Sessão Wayland, mas o app foi forçado a X11 → roda via XWayland.
    XWayland,
    /// Não foi possível determinar.
    Unknown,
}

/// Detecta o backend gráfico real, espelhando a lógica do próprio GDK.
pub fn detect_display_backend() -> DisplayBackend {
    // 1. Override explícito do usuário via GDK_BACKEND.
    if let Ok(gdk) = env::var("GDK_BACKEND") {
        let g = gdk.to_ascii_lowercase();
        if g.contains("wayland") {
            return DisplayBackend::Wayland;
        }
        if g.contains("x11") || g.contains("broadway") {
            return if is_wayland_session() {
                DisplayBackend::XWayland
            } else {
                DisplayBackend::X11
            };
        }
    }

    // 2. Detecção por variáveis de sessão.
    if is_wayland_session() {
        return DisplayBackend::Wayland;
    }
    if env::var("DISPLAY").is_ok() {
        return DisplayBackend::X11;
    }
    DisplayBackend::Unknown
}

fn is_wayland_session() -> bool {
    env::var("WAYLAND_DISPLAY").is_ok()
        || env::var("XDG_SESSION_TYPE")
            .map(|s| s.eq_ignore_ascii_case("wayland"))
            .unwrap_or(false)
}

/// Lê a preferência do usuário em settings.json (opcional). Retorna true se
/// o usuário quer forçar renderização por software (workaround p/ tela branca).
fn force_software_rendering() -> bool {
    let path = crate::config::paths::settings_file();
    if let Ok(text) = std::fs::read_to_string(&path) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
            return v
                .get("performance")
                .and_then(|p| p.get("force_software_rendering"))
                .and_then(|b| b.as_bool())
                .unwrap_or(false);
        }
    }
    false
}

pub fn configure_environment() {
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
    }
}
