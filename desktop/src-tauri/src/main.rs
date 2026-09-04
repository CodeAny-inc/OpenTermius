#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;

use state::AppState;
use tauri::Manager;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "opentermius_desktop=info,opentermius_core=info".into()),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            let app_data = app
                .path()
                .app_data_dir()
                .expect("no app data dir");
            let state = AppState::init(app.handle(), app_data);
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // hosts
            commands::list_hosts,
            commands::add_host,
            commands::update_host,
            commands::delete_host,
            // groups
            commands::list_groups,
            commands::add_group,
            commands::delete_group,
            // vault
            commands::vault_is_initialized,
            commands::initialize_vault,
            commands::unlock_vault,
            commands::lock_vault,
            commands::is_vault_unlocked,
            // keys
            commands::list_keys,
            commands::generate_key,
            commands::import_key,
            commands::delete_key,
            // known hosts
            commands::list_known_hosts,
            commands::remove_known_host,
            // workspaces
            commands::list_workspaces,
            commands::create_workspace,
            commands::save_workspace,
            commands::delete_workspace,
            commands::set_active_workspace,
            // sessions
            commands::connect_ssh,
            commands::create_local_terminal,
            commands::session_write,
            commands::session_resize,
            commands::close_session,
            commands::list_sessions,
        ])
        .run(tauri::generate_context!())
        .expect("error while running OpenTermius");
}
