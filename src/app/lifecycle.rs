use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::app::state::AppState;
use crate::profile::{manager, Profile};
use crate::webview::user_agent::USER_AGENT;
use tauri::webview::PageLoadEvent;
use tauri::{AppHandle, Emitter, Listener, Manager, WebviewUrl};

/// Window label for a given profile's main window (e.g. `main-<id>`).
pub fn profile_window_label(profile_id: &str) -> String {
    format!("main-{}", profile_id)
}

pub fn initialize(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState::new();
    app.manage(state);

    let handle = app.handle().clone();
    crate::events::bus::setup_event_forwarding(&handle);

    check_system_prerequisites();

    crate::platform::tray::setup(app.handle())?;
    crate::platform::menu::setup(app.handle())?;
    setup_single_instance(app.handle());

    let check_updates_enabled = crate::config::store::load().general.check_updates;

    if check_updates_enabled {
        let handle = app.handle().clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            let _ = crate::update::commands::check_for_updates(handle, false).await;
        });

        let handle = app.handle().clone();
        tauri::async_runtime::spawn(async move {
            loop {
                // 12h + jitter ±15min para evitar thundering herd e reduzir wakes
                let jitter = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() % 1800)
                    .unwrap_or(0);
                let interval = 12 * 60 * 60 + jitter % 900;
                tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
                let _ = crate::update::commands::check_for_updates(handle.clone(), false).await;
            }
        });
    }

    let _ = open_profile_picker(app.handle());

    Ok(())
}

pub fn open_profile_picker(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(existing) = app.get_webview_window("profile-picker") {
        let _ = existing.show();
        let _ = existing.set_focus();
        return Ok(());
    }

    // Lightweight init script for picker (no heavy platform_bridge)
    let picker_script = crate::js::build_picker_init_script();
    let window = tauri::WebviewWindowBuilder::new(
        app,
        "profile-picker",
        WebviewUrl::App("profile-picker/index.html".into()),
    )
    .title("Qwen Studio — Perfis")
    .inner_size(720.0, 600.0)
    .min_inner_size(560.0, 480.0)
    .center()
    .resizable(true)
    .decorations(true)
    .visible(false)
<<<<<<< HEAD
    .initialization_script(&picker_script)
=======
>>>>>>> c0c2f30 (Fix: Upload medias e username)
    .build()?;

    // Fallback: force-show the picker if the page's JS show() never fires,
    // so it can never get stuck hidden after the WebKit cold start.
    let fb = window.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        let _ = fb.show();
    });

    Ok(())
}

/// Opens (or focuses, if already open) a profile window.
///
/// Each profile gets its own window label (`main-<id>`) and its own isolated
/// data directory, allowing multiple profiles to run simultaneously.
pub fn open_profile_window(
    app: &AppHandle,
    profile: &Profile,
) -> Result<(), Box<dyn std::error::Error>> {
    let label = profile_window_label(&profile.id);

    // Focus the existing window for this profile instead of opening a duplicate.
    if let Some(existing) = app.get_webview_window(&label) {
        let _ = existing.show();
        let _ = existing.set_focus();
        return Ok(());
    }

    // Cap de janelas para evitar OOM linear (cada WebView ~200-400MB)
    const MAX_PROFILE_WINDOWS: usize = 4;
    let current_count = app
        .webview_windows()
        .values()
        .filter(|w| w.label().starts_with("main-"))
        .count();
    if current_count >= MAX_PROFILE_WINDOWS {
        log::warn!(
            "[Window] Limite de {} janelas atingido (atual {}), focando janela existente",
            MAX_PROFILE_WINDOWS,
            current_count
        );
        if let Some(state) = app.try_state::<AppState>() {
            if let Ok(focused) = state.last_focused.try_read() {
                if let Some(lbl) = focused.as_ref() {
                    if let Some(win) = app.get_webview_window(lbl) {
                        let _ = win.show();
                        let _ = win.set_focus();
                        return Ok(());
                    }
                }
            }
        }
        // Fallback: foca primeira janela main-* disponível
        if let Some(win) = app
            .webview_windows()
            .values()
            .find(|w| w.label().starts_with("main-"))
        {
            let _ = win.show();
            let _ = win.set_focus();
        }
        return Ok(());
    }

    let data_dir = manager::data_dir_for(&profile.id);
    let init_script = crate::js::build_init_script();

    let pid = profile.id.clone();
    let restored = Arc::new(AtomicBool::new(false));

    let _window = tauri::WebviewWindowBuilder::new(
        app,
        &label,
        WebviewUrl::External(
            manager::PROFILE_MAIN_URL
                .parse()
                .unwrap_or_else(|_| "https://chat.qwen.ai".parse().expect("hardcoded url valid")),
        ),
    )
    .title("Qwem Studio Linux")
    .inner_size(1280.0, 840.0)
    .min_inner_size(400.0, 600.0)
    .center()
    .resizable(true)
    .decorations(true)
    .accept_first_mouse(false)
    .user_agent(USER_AGENT)
    .data_directory(data_dir)
    .initialization_script(&init_script)
    .enable_clipboard_access()
    .on_navigation(|url| crate::webview::navigation::is_allowed(url.as_ref()))
<<<<<<< HEAD
    .on_page_load({
        let pid_clone = pid.clone();
        move |w, payload| {
            if payload.event() == PageLoadEvent::Finished {
                // Restaura zoom persistido (paridade navegador por origem)
                let zoom = crate::config::store::load().general.zoom;
                if zoom != 0.0 && (zoom - 1.0).abs() > f64::EPSILON {
                    let js = format!(
                        "document.documentElement.style.zoom='{}'; document.body.style.zoom='{}';",
                        zoom, zoom
                    );
                    let _ = w.eval(js);
                }
                if !restored.swap(true, Ordering::SeqCst) {
                    if let Some(session) = manager::load_session(&pid_clone) {
                        if !session.local_storage.is_empty() {
                            let js = crate::profile::cookies::restore_local_storage_js(&session);
                            let _ = w.eval(js);
                        }
                    }
=======
    .on_page_load(move |w, payload| {
        if payload.event() == PageLoadEvent::Finished && !restored.swap(true, Ordering::SeqCst) {
            if let Some(session) = manager::load_session(&pid) {
                if !session.local_storage.is_empty() {
                    let js = crate::profile::cookies::restore_local_storage_js(&session);
                    let _ = w.eval(js);
>>>>>>> c0c2f30 (Fix: Upload medias e username)
                }
            }
        }
    })
    .build()?;

    // Em WebKitGTK ≥2.50.2 o paste nativo (texto+imagem) já funciona via
    // PredefinedMenuItem::paste/cut/copy. Esconder a menubar com hide_menu()
    // desregistra o GtkAccelGroup em algumas versões Wry/GTK e faz Ctrl+C/V
    // não chegar ao WebView — por isso mantemos a barra visível no Linux.
    // O fallback explícito para imagem continua em Shift+Ctrl+V.

    crate::app::window_utils::attach_file_drop_handler(&_window);

    if let Some(state) = app.try_state::<AppState>() {
        let map = state.window_profiles.clone();
        let profile_clone = profile.clone();
        let label_clone = label.clone();
        tauri::async_runtime::spawn(async move {
            map.write().await.insert(label_clone, profile_clone);
        });

        let focused = state.last_focused.clone();
        let label_clone = label.clone();
        tauri::async_runtime::spawn(async move {
            *focused.write().await = Some(label_clone);
        });
    }

    let rid = profile.id.clone();
    let rapp = app.clone();
    tauri::async_runtime::spawn(async move {
        crate::profile::cookies::restore_session(&rapp, &label, &rid).await;
    });

    ensure_session_capture(app);

    Ok(())
}

/// Opens (or focuses) a profile's window. Used by the profile picker.
pub fn open_profile(app: &AppHandle, profile: &Profile) -> Result<(), Box<dyn std::error::Error>> {
    open_profile_window(app, profile)
}

/// Opens the profile picker and asks it to reveal the creation form.
/// The event is emitted both immediately and after a short delay so it is not
/// missed on a cold WebKit start (before the page attaches its listener).
pub fn open_picker_and_focus_create(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = open_profile_picker(&app);
        if let Some(picker) = app.get_webview_window("profile-picker") {
            let _ = picker.emit("focus-create", ());
        }
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
        if let Some(picker) = app.get_webview_window("profile-picker") {
            let _ = picker.emit("focus-create", ());
        }
    });
}

pub async fn capture_all_sessions(app: &AppHandle) {
    if let Some(state) = app.try_state::<AppState>() {
        let entries: Vec<(String, String)> = {
            let guard = state.window_profiles.read().await;
            guard
                .iter()
                .map(|(label, p)| (label.clone(), p.id.clone()))
                .collect()
        };
        // Parallel capture with timeout per session
        let futures: Vec<_> = entries
            .into_iter()
            .map(|(label, pid)| {
                let app = app.clone();
                async move {
                    let _ = tokio::time::timeout(
                        std::time::Duration::from_secs(8),
                        crate::profile::cookies::capture_session(&app, &label, &pid),
                    )
                    .await;
                }
            })
            .collect();
        futures::future::join_all(futures).await;
    }
}

fn ensure_session_capture(app: &AppHandle) {
    let state = match app.try_state::<AppState>() {
        Some(s) => s,
        None => return,
    };
    if state.session_capture_running.swap(true, Ordering::SeqCst) {
        return;
    }

    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(600));
        loop {
            interval.tick().await;
            // Só captura janela focada para reduzir wakes em idle (economia de CPU/IO).
            // Janelas desfocadas são salvas em Focused(false) com debounce e em CloseRequested.
            let focused_label = match app.try_state::<AppState>() {
                Some(state) => state.last_focused.read().await.clone(),
                None => None,
            };
            let entries: Vec<(String, String)> = match app.try_state::<AppState>() {
                Some(state) => {
                    let guard = state.window_profiles.read().await;
                    if let Some(ref focused) = focused_label {
                        if let Some(p) = guard.get(focused) {
                            if app.get_webview_window(focused).is_some() {
                                vec![(focused.clone(), p.id.clone())]
                            } else {
                                Vec::new()
                            }
                        } else {
                            Vec::new()
                        }
                    } else {
                        // Sem foco (app em background), captura no máximo 1 para manter sessão fresca sem acordar N janelas
                        guard
                            .iter()
                            .find(|(label, _)| app.get_webview_window(label).is_some())
                            .map(|(label, p)| vec![(label.clone(), p.id.clone())])
                            .unwrap_or_default()
                    }
                }
                None => Vec::new(),
            };
            if entries.is_empty() {
                continue;
            }
            let futures: Vec<_> = entries
                .into_iter()
                .map(|(label, pid)| {
                    let app = app.clone();
                    async move {
                        let _ = tokio::time::timeout(
                            std::time::Duration::from_secs(8),
                            crate::profile::cookies::capture_session(&app, &label, &pid),
                        )
                        .await;
                    }
                })
                .collect();
            futures::future::join_all(futures).await;
        }
    });
}

fn check_system_prerequisites() {
    // Run blocking check off the main thread with timeout to avoid stalling setup
    std::thread::spawn(|| {
        let result = std::process::Command::new("gst-inspect-1.0")
            .arg("autoaudiosink")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        if let Ok(status) = result {
            if !status.success() {
                log::warn!("[System] GStreamer 'autoaudiosink' plugin not found");
            }
        }
    });
}

pub fn setup_single_instance(app: &AppHandle) {
    let handle = app.clone();
    app.listen_any("single-instance-check", move |_| {
        let target = crate::app::window_utils::active_webview_window(&handle)
            .or_else(|| handle.get_webview_window("profile-picker"));
        if let Some(window) = target {
            let _ = window.show();
            let _ = window.set_focus();
        }
    });
}

pub fn on_run_event(app_handle: &AppHandle, event: tauri::RunEvent) {
    if let tauri::RunEvent::WindowEvent {
        label,
        event: win_event,
        ..
    } = event
    {
        match win_event {
            tauri::WindowEvent::CloseRequested { .. } => {
                if label.starts_with("main-") {
                    let app_h = app_handle.clone();
                    let label_clone = label.to_string();
                    tauri::async_runtime::spawn(async move {
                        if let Some(state) = app_h.try_state::<AppState>() {
                            let pid = {
                                let guard = state.window_profiles.read().await;
                                guard.get(&label_clone).map(|p| p.id.clone())
                            };
                            if let Some(pid) = pid {
                                crate::profile::cookies::capture_session(
                                    &app_h,
                                    &label_clone,
                                    &pid,
                                )
                                .await;
                            }
                            let mut guard = state.window_profiles.write().await;
                            guard.remove(&label_clone);
                        }
                    });

                    // If this was the last profile window, surface the picker.
                    let remaining = app_handle
                        .webview_windows()
                        .into_values()
                        .filter(|w| w.label().starts_with("main-") && w.label() != label)
                        .count();
                    if remaining == 0 {
                        let app_h = app_handle.clone();
                        tauri::async_runtime::spawn(async move {
                            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                            let _ = open_profile_picker(&app_h);
                        });
                    }
                    return;
                }

                if label == "profile-picker" {
                    // Allow the picker to close freely; the app stays alive in the tray.
                }
            }
            tauri::WindowEvent::Focused(true) if label.starts_with("main-") => {
                if let Some(state) = app_handle.try_state::<AppState>() {
                    // Use try_write to never panic if the lock is held by an async task.
                    // Fallback to async spawn if contended.
                    if let Ok(mut focused) = state.last_focused.try_write() {
                        *focused = Some(label.to_string());
                    } else {
                        let focused = state.last_focused.clone();
                        let label = label.to_string();
                        tauri::async_runtime::spawn(async move {
                            *focused.write().await = Some(label);
                        });
                    }
                }
            }
            tauri::WindowEvent::Focused(false) if label.starts_with("main-") => {
                // Flush sessão com debounce 2s ao perder foco (economiza wakes vs polling, garante persistência)
                let app_h = app_handle.clone();
                let label_clone = label.to_string();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    if let Some(win) = app_h.get_webview_window(&label_clone) {
                        if win.is_focused().unwrap_or(false) {
                            return;
                        }
                    }
                    if let Some(state) = app_h.try_state::<AppState>() {
                        let pid = {
                            let guard = state.window_profiles.read().await;
                            guard.get(&label_clone).map(|p| p.id.clone())
                        };
                        if let Some(pid) = pid {
                            let _ = tokio::time::timeout(
                                std::time::Duration::from_secs(8),
                                crate::profile::cookies::capture_session(&app_h, &label_clone, &pid),
                            )
                            .await;
                        }
                    }
                });
            }
            _ => {}
        }
    }
}
