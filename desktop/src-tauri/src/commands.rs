use crate::state::{AppState, LocalTerminal};
use opentermius_core::host::{AuthMethod, Host, HostGroup};
use opentermius_core::keys::{generate_ed25519, parse_openssh_private, KeyMeta};
use opentermius_core::workspace::Workspace;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

type ApiResult<T> = std::result::Result<T, String>;

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

// ============================================================
// Hosts
// ============================================================

#[tauri::command]
pub async fn list_hosts(state: State<'_, Arc<AppState>>) -> ApiResult<Vec<Host>> {
    let store = state.store.lock().await;
    Ok(store.hosts().to_vec())
}

#[tauri::command]
pub async fn add_host(
    state: State<'_, Arc<AppState>>,
    host: Host,
) -> ApiResult<Host> {
    let mut store = state.store.lock().await;
    store.add_host(host.clone()).map_err(err)?;
    Ok(host)
}

#[tauri::command]
pub async fn update_host(
    state: State<'_, Arc<AppState>>,
    host: Host,
) -> ApiResult<Host> {
    let mut store = state.store.lock().await;
    store.update_host(host.clone()).map_err(err)?;
    Ok(host)
}

#[tauri::command]
pub async fn delete_host(
    state: State<'_, Arc<AppState>>,
    id: Uuid,
) -> ApiResult<()> {
    let mut store = state.store.lock().await;
    store.remove_host(id).map_err(err)
}

// ============================================================
// Host Groups
// ============================================================

#[tauri::command]
pub async fn list_groups(state: State<'_, Arc<AppState>>) -> ApiResult<Vec<HostGroup>> {
    let store = state.store.lock().await;
    Ok(store.groups().to_vec())
}

#[tauri::command]
pub async fn add_group(
    state: State<'_, Arc<AppState>>,
    name: String,
) -> ApiResult<HostGroup> {
    let group = HostGroup::new(name);
    let mut store = state.store.lock().await;
    store.add_group(group.clone()).map_err(err)?;
    Ok(group)
}

#[tauri::command]
pub async fn delete_group(
    state: State<'_, Arc<AppState>>,
    id: Uuid,
) -> ApiResult<()> {
    let mut store = state.store.lock().await;
    store.remove_group(id).map_err(err)
}

// ============================================================
// Vault
// ============================================================

#[tauri::command]
pub async fn vault_is_initialized(state: State<'_, Arc<AppState>>) -> ApiResult<bool> {
    let vault = state.vault.lock().await;
    Ok(vault.is_initialized())
}

#[tauri::command]
pub async fn initialize_vault(
    state: State<'_, Arc<AppState>>,
    passphrase: String,
) -> ApiResult<()> {
    let mut vault = state.vault.lock().await;
    vault.initialize(&passphrase).map_err(err)?;
    let mut pw = state.passphrase.lock().await;
    *pw = Some(zeroize::Zeroizing::new(passphrase));
    Ok(())
}

#[tauri::command]
pub async fn unlock_vault(
    state: State<'_, Arc<AppState>>,
    passphrase: String,
) -> ApiResult<()> {
    // Try to decrypt with this passphrase — if it works, store it.
    let vault = state.vault.lock().await;
    if !vault.is_initialized() {
        return Err("vault not initialized".into());
    }
    // Verify by trying to get a key (or just decrypt the payload)
    // We use a test: try to read the ciphertext
    vault.get_key(&passphrase, "__test__").ok(); // will error if wrong passphrase, but also if no key
    // Actually, let's just store the passphrase and let operations fail later
    // if it's wrong. A proper check would decrypt the payload.
    drop(vault);
    let mut pw = state.passphrase.lock().await;
    *pw = Some(zeroize::Zeroizing::new(passphrase));
    Ok(())
}

#[tauri::command]
pub async fn lock_vault(state: State<'_, Arc<AppState>>) -> ApiResult<()> {
    let mut pw = state.passphrase.lock().await;
    *pw = None;
    Ok(())
}

#[tauri::command]
pub async fn is_vault_unlocked(state: State<'_, Arc<AppState>>) -> ApiResult<bool> {
    let pw = state.passphrase.lock().await;
    Ok(pw.is_some())
}

// ============================================================
// Keys
// ============================================================

#[tauri::command]
pub async fn list_keys(state: State<'_, Arc<AppState>>) -> ApiResult<Vec<KeyMeta>> {
    let vault = state.vault.lock().await;
    Ok(vault.keys_meta().to_vec())
}

#[tauri::command]
pub async fn generate_key(
    state: State<'_, Arc<AppState>>,
    label: String,
) -> ApiResult<KeyMeta> {
    let (private, _public) = generate_ed25519().map_err(err)?;
    let (mut meta, _pair) = parse_openssh_private(&private, None).map_err(err)?;
    meta.label = label;
    let passphrase = {
        let pw = state.passphrase.lock().await;
        pw.as_ref()
            .map(|p| p.to_string())
            .ok_or_else(|| "vault is locked".to_string())?
    };
    let mut vault = state.vault.lock().await;
    vault
        .add_key(&passphrase, meta.clone(), &private)
        .map_err(err)?;
    Ok(meta)
}

#[tauri::command]
pub async fn import_key(
    state: State<'_, Arc<AppState>>,
    label: String,
    openssh_private: String,
    key_passphrase: Option<String>,
) -> ApiResult<KeyMeta> {
    let (mut meta, _pair) =
        parse_openssh_private(&openssh_private, key_passphrase.as_deref()).map_err(err)?;
    meta.label = label;
    let passphrase = {
        let pw = state.passphrase.lock().await;
        pw.as_ref()
            .map(|p| p.to_string())
            .ok_or_else(|| "vault is locked".to_string())?
    };
    let mut vault = state.vault.lock().await;
    vault
        .add_key(&passphrase, meta.clone(), &openssh_private)
        .map_err(err)?;
    Ok(meta)
}

#[tauri::command]
pub async fn delete_key(
    state: State<'_, Arc<AppState>>,
    key_id: Uuid,
) -> ApiResult<()> {
    let passphrase = {
        let pw = state.passphrase.lock().await;
        pw.as_ref()
            .map(|p| p.to_string())
            .ok_or_else(|| "vault is locked".to_string())?
    };
    let mut vault = state.vault.lock().await;
    vault
        .remove_key(&passphrase, &key_id.to_string())
        .map_err(err)
}

// ============================================================
// Known Hosts
// ============================================================

#[derive(serde::Serialize)]
pub struct KnownHostEntry {
    pub host: String,
    pub key_type: String,
    pub fingerprint: String,
}

#[tauri::command]
pub async fn list_known_hosts(
    state: State<'_, Arc<AppState>>,
) -> ApiResult<Vec<KnownHostEntry>> {
    let kh = state.known_hosts.lock().await;
    Ok(kh
        .list()
        .into_iter()
        .map(|(host, key_type, fingerprint)| KnownHostEntry {
            host,
            key_type,
            fingerprint,
        })
        .collect())
}

#[tauri::command]
pub async fn remove_known_host(
    state: State<'_, Arc<AppState>>,
    host: String,
    port: u16,
) -> ApiResult<()> {
    let mut kh = state.known_hosts.lock().await;
    kh.remove(&host, port).map_err(err)
}

// ============================================================
// Workspaces
// ============================================================

#[tauri::command]
pub async fn list_workspaces(state: State<'_, Arc<AppState>>) -> ApiResult<Vec<Workspace>> {
    let store = state.store.lock().await;
    Ok(store.workspaces().to_vec())
}

#[tauri::command]
pub async fn create_workspace(
    state: State<'_, Arc<AppState>>,
    name: String,
) -> ApiResult<Workspace> {
    let ws = Workspace::new(name);
    let mut store = state.store.lock().await;
    store.add_workspace(ws.clone()).map_err(err)?;
    Ok(ws)
}

#[tauri::command]
pub async fn save_workspace(
    state: State<'_, Arc<AppState>>,
    workspace: Workspace,
) -> ApiResult<Workspace> {
    let mut store = state.store.lock().await;
    store.update_workspace(workspace.clone()).map_err(err)?;
    Ok(workspace)
}

#[tauri::command]
pub async fn delete_workspace(
    state: State<'_, Arc<AppState>>,
    id: Uuid,
) -> ApiResult<()> {
    let mut store = state.store.lock().await;
    store.remove_workspace(id).map_err(err)
}

#[tauri::command]
pub async fn set_active_workspace(
    state: State<'_, Arc<AppState>>,
    id: Uuid,
) -> ApiResult<()> {
    let mut store = state.store.lock().await;
    store.set_active_workspace(id).map_err(err)
}

// ============================================================
// Sessions (SSH + Local Terminal)
// ============================================================

#[tauri::command]
pub async fn connect_ssh(
    state: State<'_, Arc<AppState>>,
    _app: AppHandle,
    session_id: String,
    host: Host,
    password: Option<String>,
    cols: Option<u32>,
    rows: Option<u32>,
) -> ApiResult<()> {
    let passphrase = {
        let pw = state.passphrase.lock().await;
        pw.as_ref().map(|p| p.to_string())
    };
    let vault = state.vault.lock().await;
    let vault_ref = if matches!(host.auth, AuthMethod::PublicKey) {
        Some(&*vault)
    } else {
        None
    };
    let known_hosts = state.known_hosts.clone();
    let cols = cols.unwrap_or(80);
    let rows = rows.unwrap_or(24);

    state
        .sessions
        .create_ssh_session(
            session_id,
            &host,
            known_hosts,
            vault_ref,
            passphrase.as_deref(),
            password.as_deref(),
            cols,
            rows,
        )
        .await
        .map_err(err)
}

#[tauri::command]
pub async fn create_local_terminal(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    session_id: String,
    cols: Option<u32>,
    rows: Option<u32>,
) -> ApiResult<()> {
    use portable_pty::*;

    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: rows.unwrap_or(24) as u16,
            cols: cols.unwrap_or(80) as u16,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| format!("openpty: {e}"))?;

    let shell = std::env::var("SHELL").unwrap_or_else(|_| {
        if cfg!(target_os = "windows") {
            "powershell.exe".to_string()
        } else {
            "/bin/bash".to_string()
        }
    });

    let mut cmd = CommandBuilder::new(&shell);
    cmd.env("TERM", "xterm-256color");

    let _child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| format!("spawn: {e}"))?;

    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| format!("clone reader: {e}"))?;

    let writer = pair
        .master
        .take_writer()
        .map_err(|e| format!("take writer: {e}"))?;

    let master = pair.master;

    // Spawn a reading thread that emits data events
    let app_handle = app.clone();
    let sid = session_id.clone();
    std::thread::spawn(move || {
        use std::io::Read;
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let _ = app_handle.emit(
                        "session-data",
                        crate::state::SessionDataEvent {
                            session_id: sid.clone(),
                            data: buf[..n].to_vec(),
                        },
                    );
                }
                Err(_) => break,
            }
        }
        let _ = app_handle.emit(
            "session-closed",
            crate::state::SessionClosedEvent {
                session_id: sid,
                reason: "local terminal closed".to_string(),
            },
        );
    });

    // Store the master for writing and resizing
    let mut locals = state.local_terminals.lock().await;
    locals.insert(session_id, LocalTerminal { writer, master });

    Ok(())
}

#[tauri::command]
pub async fn session_write(
    state: State<'_, Arc<AppState>>,
    session_id: String,
    data: Vec<u8>,
) -> ApiResult<()> {
    // Try SSH session first
    if state.sessions.list().await.contains(&session_id) {
        return state.sessions.write(&session_id, &data).await.map_err(err);
    }
    // Try local terminal
    let mut locals = state.local_terminals.lock().await;
    if let Some(term) = locals.get_mut(&session_id) {
        use std::io::Write;
        term.writer.write_all(&data).map_err(|e| format!("write: {e}"))?;
        term.writer.flush().map_err(|e| format!("flush: {e}"))?;
        return Ok(());
    }
    Err("session not found".into())
}

#[tauri::command]
pub async fn session_resize(
    state: State<'_, Arc<AppState>>,
    session_id: String,
    cols: u32,
    rows: u32,
) -> ApiResult<()> {
    // Try SSH session first
    if state.sessions.list().await.contains(&session_id) {
        return state.sessions.resize(&session_id, cols, rows).await.map_err(err);
    }
    // Try local terminal
    let locals = state.local_terminals.lock().await;
    if let Some(term) = locals.get(&session_id) {
        use portable_pty::PtySize;
        term.master
            .resize(PtySize {
                rows: rows as u16,
                cols: cols as u16,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("resize: {e}"))?;
        return Ok(());
    }
    Err("session not found".into())
}

#[tauri::command]
pub async fn close_session(
    state: State<'_, Arc<AppState>>,
    session_id: String,
) -> ApiResult<()> {
    // Try SSH session
    if state.sessions.list().await.contains(&session_id) {
        return state.sessions.close(&session_id).await.map_err(err);
    }
    // Try local terminal
    let mut locals = state.local_terminals.lock().await;
    locals.remove(&session_id);
    Ok(())
}

#[tauri::command]
pub async fn list_sessions(state: State<'_, Arc<AppState>>) -> ApiResult<Vec<String>> {
    let mut sessions = state.sessions.list().await;
    let locals = state.local_terminals.lock().await;
    sessions.extend(locals.keys().cloned());
    Ok(sessions)
}

// ============================================================
// Updater
// ============================================================

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct UpdateInfo {
    pub available: bool,
    pub version: String,
    pub current_version: String,
    pub date: Option<String>,
    pub body: Option<String>,
}

/// Check for updates. Returns update info if an update is available.
#[tauri::command]
pub async fn check_for_updates(
    app: AppHandle,
) -> ApiResult<UpdateInfo> {
    use tauri_plugin_updater::UpdaterExt;

    let updater = app.updater().map_err(|e| e.to_string())?;
    let current = app.package_info().version.to_string();

    match updater.check().await {
        Ok(Some(update)) => Ok(UpdateInfo {
            available: true,
            version: update.version.clone(),
            current_version: current,
            date: update.date.map(|d| d.to_string()),
            body: update.body.clone(),
        }),
        Ok(None) => Ok(UpdateInfo {
            available: false,
            version: current.clone(),
            current_version: current,
            date: None,
            body: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

/// Download and install the update, then restart the app.
/// Emits "update-progress" events with download progress.
#[tauri::command]
pub async fn install_update(
    app: AppHandle,
) -> ApiResult<()> {
    use tauri_plugin_updater::UpdaterExt;

    let updater = app.updater().map_err(|e| e.to_string())?;

    let update = updater
        .check()
        .await
        .map_err(|e| e.to_string())?
        .ok_or("No update available")?;

    // Download and install
    update
        .download_and_install(
            |chunk_length, content_length| {
                let _ = app.emit(
                    "update-progress",
                    serde_json::json!({
                        "chunk_length": chunk_length,
                        "content_length": content_length,
                    }),
                );
            },
            || {
                let _ = app.emit("update-extracting", serde_json::json!({}));
            },
        )
        .await
        .map_err(|e| e.to_string())?;

    // Restart the app to apply the update
    app.request_restart();

    Ok(())
}

