use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem, Submenu, SubmenuBuilder},
    Listener,
};

use crate::profile::manager;

type MenuResult = Result<Submenu<tauri::Wry>, Box<dyn std::error::Error>>;

/// Builds (and applies) the full application menu, including the dynamic
/// "Perfils" submenu. Safe to call repeatedly — used to refresh the profile
/// list after profiles are created/renamed/deleted.
pub fn build_app_menu(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let file_menu = SubmenuBuilder::new(app, "File")
        .item(&MenuItemBuilder::with_id("minimize", "Minimize").build(app)?)
        .item(&MenuItemBuilder::with_id("maximize", "Maximize").build(app)?)
        .separator()
        .item(&MenuItemBuilder::with_id("quit", "Quit").build(app)?)
        .build()?;

    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .item(&PredefinedMenuItem::undo(app, None)?)
        .item(&PredefinedMenuItem::redo(app, None)?)
        .separator()
        .item(&PredefinedMenuItem::cut(app, None)?)
        .item(&PredefinedMenuItem::copy(app, None)?)
        .item(&MenuItemBuilder::with_id("paste", "Paste").build(app)?)
        .item(
            &MenuItemBuilder::with_id("paste-image", "Paste Image")
                .accelerator("CmdOrCtrl+Shift+V")
                .build(app)?,
        )
        .item(&PredefinedMenuItem::select_all(app, None)?)
        .build()?;

    let view_menu = SubmenuBuilder::new(app, "View")
        .item(&MenuItemBuilder::with_id("reload", "Reload").build(app)?)
        .item(&MenuItemBuilder::with_id("devtools", "Toggle DevTools").build(app)?)
        .separator()
        .item(&MenuItemBuilder::with_id("zoom_in", "Zoom In").build(app)?)
        .item(&MenuItemBuilder::with_id("zoom_out", "Zoom Out").build(app)?)
        .item(&MenuItemBuilder::with_id("zoom_reset", "Reset Zoom").build(app)?)
        .build()?;

    let window_menu = SubmenuBuilder::new(app, "Window")
        .item(&MenuItemBuilder::with_id("new_window", "New Window").build(app)?)
        .item(&PredefinedMenuItem::fullscreen(app, None)?)
        .build()?;

    let profiles_menu = build_profiles_submenu(app)?;

    let help_menu = SubmenuBuilder::new(app, "Help")
        .item(&MenuItemBuilder::with_id("documentation", "Documentation").build(app)?)
        .item(&MenuItemBuilder::with_id("github", "GitHub").build(app)?)
        .separator()
        .item(&MenuItemBuilder::with_id("check_updates", "Check for Updates").build(app)?)
        .build()?;

    let menu = MenuBuilder::new(app)
        .item(&file_menu)
        .item(&edit_menu)
        .item(&view_menu)
        .item(&window_menu)
        .item(&profiles_menu)
        .item(&help_menu)
        .build()?;

    app.set_menu(menu)?;
    Ok(())
}

/// Builds the "Perfils" submenu, reflecting the current set of profiles.
fn build_profiles_submenu(app: &tauri::AppHandle) -> MenuResult {
    let profiles = manager::load();

    let open_panel =
        MenuItemBuilder::with_id("profiles_panel", "Abrir painel dos perfils").build(app)?;

    let mut owned: Vec<tauri::menu::MenuItem<tauri::Wry>> = Vec::new();
    for p in &profiles {
        owned.push(
            MenuItemBuilder::with_id(format!("open-profile:{}", p.id), p.name.clone())
                .build(app)?,
        );
    }
    let item_refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = owned
        .iter()
        .map(|i| i as &dyn tauri::menu::IsMenuItem<tauri::Wry>)
        .collect();

    let open_profile_sub = SubmenuBuilder::new(app, "Abrir perfil")
        .items(&item_refs)
        .build()?;

    let create_profile_sub = SubmenuBuilder::new(app, "Criar novo Perfil")
        .item(&MenuItemBuilder::with_id("create_profile", "Criar perfil").build(app)?)
        .build()?;

    let submenu = SubmenuBuilder::new(app, "Perfils")
        .item(&open_panel)
        .separator()
        .item(&open_profile_sub)
        .separator()
        .item(&create_profile_sub)
        .build()?;

    Ok(submenu)
}

pub fn setup(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    build_app_menu(app)?;

    let handle = app.clone();
    app.on_menu_event(move |app, event| {
        let id = event.id().as_ref();
        match id {
            "minimize" => {
                if let Some(w) = crate::app::window_utils::active_webview_window(app) {
                    let _ = w.minimize();
                }
            }
            "maximize" => {
                if let Some(w) = crate::app::window_utils::active_webview_window(app) {
                    if w.is_maximized().unwrap_or(false) {
                        let _ = w.unmaximize();
                    } else {
                        let _ = w.maximize();
                    }
                }
            }
            "quit" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    crate::app::lifecycle::capture_all_sessions(&app).await;
                    app.exit(0);
                });
            }
            "reload" => {
                if let Some(w) = crate::app::window_utils::active_webview_window(app) {
                    let _ = w.eval("location.reload();");
                }
            }
            "devtools" => {
                if let Some(w) = crate::app::window_utils::active_webview_window(app) {
                    if w.is_devtools_open() {
                        w.close_devtools();
                    } else {
                        w.open_devtools();
                    }
                }
            }
            "zoom_in" => {
                if let Some(w) = crate::app::window_utils::active_webview_window(app) {
                    let _ = w.eval(
                        "window.__qwenSetZoom && window.__qwenSetZoom(Math.min(2.0, (window.__qwenCurrentZoom||1)+0.1));",
                    );
                }
            }
            "zoom_out" => {
                if let Some(w) = crate::app::window_utils::active_webview_window(app) {
                    let _ = w.eval(
                        "window.__qwenSetZoom && window.__qwenSetZoom(Math.max(0.5, (window.__qwenCurrentZoom||1)-0.1));",
                    );
                }
            }
            "zoom_reset" => {
                if let Some(w) = crate::app::window_utils::active_webview_window(app) {
                    let _ = w.eval("window.__qwenSetZoom && window.__qwenSetZoom(1.0);");
                }
            }
            "new_window" => {
                let handle = handle.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = crate::ipc::window::create_new_window(handle).await;
                });
            }
            "profiles_panel" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = crate::app::lifecycle::open_profile_picker(&app);
                });
            }
            "create_profile" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    crate::app::lifecycle::open_picker_and_focus_create(&app);
                });
            }
            id if id.starts_with("open-profile:") => {
                let profile_id = id.trim_start_matches("open-profile:").to_string();
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Some(profile) = manager::load()
                        .into_iter()
                        .find(|p| p.id == profile_id)
                    {
                        let _ = crate::app::lifecycle::open_profile_window(&app, &profile);
                    }
                });
            }
            "github" => {
                let _ = open::that("https://github.com/yks777/qwen-studio-linux");
            }
            "check_updates" => {
                let handle = handle.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = crate::update::commands::check_for_updates(handle, false).await;
                });
            }
            "paste" => {
                if let Some(w) = crate::app::window_utils::active_webview_window(app) {
                    let _ = w.eval("window.__qwenScheduleFallbackPaste && window.__qwenScheduleFallbackPaste();");
                }
            }
            "paste-image" => {
                if let Some(w) = crate::app::window_utils::active_webview_window(app) {
                    let _ = w.eval("window.__qwenScheduleFallbackPaste && window.__qwenScheduleFallbackPaste();");
                }
            }
            _ => {}
        }
    });

    // Rebuild the menu (notably the Perfils submenu) whenever profiles change.
    let rebuild_handle = app.clone();
    app.listen("profiles-updated", move |_| {
        let handle = rebuild_handle.clone();
        let closure_handle = handle.clone();
        let _ = handle.run_on_main_thread(move || {
            let _ = build_app_menu(&closure_handle);
        });
    });

    Ok(())
}
