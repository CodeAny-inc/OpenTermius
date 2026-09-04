#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;

use state::AppState;
use tauri::Manager;
#[cfg(not(debug_assertions))]
use tauri::Emitter;

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
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let app_data = app
                .path()
                .app_data_dir()
                .expect("no app data dir");
            let state = AppState::init(app.handle(), app_data);
            app.manage(state);

            // Check for updates on startup (non-blocking, silent)
            // Only in release builds — skip in dev to avoid noise
            #[cfg(not(debug_assertions))]
            {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    // Small delay to let the frontend mount and register
                    // its event listeners before we emit update events
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    check_for_updates_silent(handle).await;
                });
            }

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
            // identities
            commands::list_identities,
            commands::add_identity,
            commands::update_identity,
            commands::delete_identity,
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
            // file I/O
            commands::read_key_file,
            // updater
            commands::check_for_updates,
            commands::install_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running OpenTermius");
}

/// Silent background check — emits an event to the frontend if an update
/// is available, so the UI can show a banner. Does not auto-install.
///
/// Uses the custom prerelease-aware check (queries GitHub API for all
/// releases including prereleases, picks the highest semver version).
#[cfg(not(debug_assertions))]
async fn check_for_updates_silent(app: tauri::AppHandle) {
    match commands::check_with_prerelease_endpoint(&app).await {
        Ok(Some(update)) => {
            let current = app.package_info().version.to_string();
            tracing::info!(
                "update available: v{} (current: {})",
                update.version,
                current
            );
            // Emit event so the frontend can show an update banner.
            // Include ALL fields the frontend UpdateInfo type expects.
            let _ = app.emit("update-available", serde_json::json!({
                "available": true,
                "version": update.version,
                "current_version": current,
                "date": update.date.map(|d| d.to_string()),
                "body": update.body,
            }));
        }
        Ok(None) => {
            tracing::debug!("no update available");
        }
        Err(e) => {
            tracing::warn!("update check failed: {e}");
        }
    }
}
