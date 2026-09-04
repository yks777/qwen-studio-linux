use tauri::{
    image::Image,
    menu::{Menu, MenuItem, MenuItemBuilder, Submenu, SubmenuBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Listener,
};

use crate::profile::manager;

fn build_tray_menu(app: &tauri::AppHandle) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let new_window_item = MenuItem::with_id(app, "new_window", "New Window", true, None::<&str>)?;
    let open_panel_item =
        MenuItemBuilder::with_id("profiles_panel", "Abrir painel dos perfils").build(app)?;
    let mut owned: Vec<tauri::menu::MenuItem<tauri::Wry>> = Vec::new();
    for p in manager::load() {
        owned.push(MenuItemBuilder::with_id(format!("open-profile:{}", p.id), p.name).build(app)?);
    }
    let item_refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = owned
        .iter()
        .map(|i| i as &dyn tauri::menu::IsMenuItem<tauri::Wry>)
        .collect();
    let open_profile_sub: Submenu<tauri::Wry> = SubmenuBuilder::new(app, "Abrir perfil")
        .items(&item_refs)
        .build()?;
    let create_item = MenuItemBuilder::with_id("create_profile", "Criar perfil").build(app)?;
    let profiles_sub: Submenu<tauri::Wry> = SubmenuBuilder::new(app, "Perfils")
        .item(&open_panel_item)
        .separator()
        .item(&open_profile_sub)
        .separator()
        .item(&create_item)
        .build()?;
    let show_item = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    Menu::with_items(app, &[&new_window_item, &profiles_sub, &show_item, &quit_item])
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

pub fn setup(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let menu = build_tray_menu(app)?;
    let icon_bytes = include_bytes!("../../icons/icon.png");
    let img = image::load_from_memory(icon_bytes)
        .unwrap_or_else(|e| {
            log::error!("[Tray] failed to load icon: {}", e);
            image::DynamicImage::new_rgba8(32, 32)
        })
        .into_rgba8();
    let (width, height) = img.dimensions();
    let rgba = img.into_raw();
    let icon = Image::new_owned(rgba, width, height);

    let _tray = TrayIconBuilder::with_id("main")
        .icon(icon)
        .tooltip("Qwem Studio Linux")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "new_window" => {
                let app_clone = app.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = crate::ipc::window::create_new_window(app_clone).await;
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
                    if let Some(profile) = manager::load().into_iter().find(|p| p.id == profile_id)
                    {
                        let _ = crate::app::lifecycle::open_profile_window(&app, &profile);
                    }
                });
            }
            "show" => {
                if let Some(window) = crate::app::window_utils::active_webview_window(app) {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "quit" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    crate::app::lifecycle::capture_all_sessions(&app).await;
                    app.exit(0);
                });
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                if let Some(window) =
                    crate::app::window_utils::active_webview_window(tray.app_handle())
                {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    // Rebuild tray menu when profiles change — fixes stale menu (famous Tauri pattern)
    let rebuild_handle = app.clone();
    app.listen("profiles-updated", move |_| {
        if let Some(tray) = rebuild_handle.tray_by_id("main") {
            if let Ok(new_menu) = build_tray_menu(&rebuild_handle) {
                let _ = tray.set_menu(Some(new_menu));
            }
        }
    });

    Ok(())
}
