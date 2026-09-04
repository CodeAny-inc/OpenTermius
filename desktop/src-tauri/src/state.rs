use opentermius_core::known_hosts::KnownHosts;
use opentermius_core::session::SessionManager;
use opentermius_core::store::Store;
use opentermius_core::vault::Vault;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

/// Shared app state. The vault passphrase is held in memory only for the
/// duration of an unlocked session; it is never persisted. On lock, it is
/// zeroized.
pub struct AppState {
    pub store: Mutex<Store>,
    pub vault: Mutex<Vault>,
    pub known_hosts: Arc<Mutex<KnownHosts>>,
    pub passphrase: Mutex<Option<zeroize::Zeroizing<String>>>,
    pub sessions: Arc<SessionManager>,
    pub local_terminals: Mutex<std::collections::HashMap<String, LocalTerminal>>,
    pub app_data_dir: PathBuf,
}

/// A local terminal session backed by portable-pty.
pub struct LocalTerminal {
    pub writer: Box<dyn std::io::Write + Send>,
    pub master: Box<dyn portable_pty::MasterPty + Send>,
    pub _child: Box<dyn portable_pty::Child + Send + Sync>,
}

impl AppState {
    pub fn init(app: &AppHandle, app_data: PathBuf) -> Arc<Self> {
        std::fs::create_dir_all(&app_data).ok();
        let store = Store::load(app_data.join("store.json")).expect("load store");
        let vault = Vault::open(app_data.join("vault.json")).expect("open vault");
        let known_hosts =
            KnownHosts::load(app_data.join("known_hosts.json")).expect("open known_hosts");

        let app_handle = app.clone();
        let data_callback = Arc::new(move |sid: &str, data: &[u8]| {
            let _ = app_handle.emit(
                "session-data",
                SessionDataEvent {
                    session_id: sid.to_string(),
                    data: data.to_vec(),
                },
            );
        });

        let app_handle2 = app.clone();
        let close_callback = Arc::new(move |sid: &str, reason: &str| {
            let _ = app_handle2.emit(
                "session-closed",
                SessionClosedEvent {
                    session_id: sid.to_string(),
                    reason: reason.to_string(),
                },
            );
        });

        let sessions = Arc::new(SessionManager::new(data_callback, close_callback));

        Arc::new(Self {
            store: Mutex::new(store),
            vault: Mutex::new(vault),
            known_hosts: Arc::new(Mutex::new(known_hosts)),
            passphrase: Mutex::new(None),
            sessions,
            local_terminals: Mutex::new(std::collections::HashMap::new()),
            app_data_dir: app_data,
        })
    }
}

#[derive(Clone, serde::Serialize)]
pub struct SessionDataEvent {
    pub session_id: String,
    pub data: Vec<u8>,
}

#[derive(Clone, serde::Serialize)]
pub struct SessionClosedEvent {
    pub session_id: String,
    pub reason: String,
}
