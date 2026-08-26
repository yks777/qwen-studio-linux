mod app;
mod auth;
mod config;
mod events;
mod ipc;
mod mcp;
mod platform;
mod profile;
mod update;
mod webview;
mod js;

pub fn run() {
    platform::env::configure_environment();

    app::panic::install_panic_hook();

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_mcp_bridge::init())
        .setup(|app| {
            app::lifecycle::initialize(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::window::create_new_window,
            ipc::window::minimize_window,
            ipc::window::maximize_window,
            ipc::window::close_window,
            ipc::window::open_devtool,
            ipc::window::toggle_hidden_devtools,
            ipc::window::open_external_link,
            ipc::window::switch_theme,
            ipc::window::switch_ln,
            ipc::window::update_title_bar_for_system_theme,
            ipc::window::get_language,
            ipc::clipboard::read_clipboard_image,
            ipc::dialog::show_native_dialog,
            ipc::dialog::request_file_access,
            ipc::settings::get_setting,
            ipc::settings::set_setting,
            ipc::export::export_chat,
            ipc::shortcuts::handle_shortcut,
            ipc::crash::list_crash_logs,
            ipc::crash::read_crash_log,
            ipc::version::get_app_version,
            ipc::version::get_platform_info,
            mcp::commands::mcp_client_connect,
            mcp::commands::mcp_client_close,
            mcp::commands::mcp_client_tool_list,
            mcp::commands::mcp_client_tool_call,
            mcp::commands::mcp_client_get_config,
            mcp::commands::mcp_client_update_config,
            update::commands::check_for_updates,
            update::commands::install_update_with_progress,
            update::commands::restart_app,
            update::commands::get_update_info,
            events::commands::webview_loaded,
            profile::commands::list_profiles,
            profile::commands::create_profile,
            profile::commands::rename_profile,
            profile::commands::delete_profile,
            profile::commands::launch_profile,
            profile::commands::list_categories,
            profile::commands::update_profile,
        ])
        .build(tauri::generate_context!())
        .unwrap_or_else(|e| {
            log::error!("[Fatal] Failed to build application: {}", e);
            panic!("Failed to build application: {}", e);
        })
        .run(app::lifecycle::on_run_event);
}
