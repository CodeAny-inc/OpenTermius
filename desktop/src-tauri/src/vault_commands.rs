use crate::state::AppState;
use std::sync::Arc;
use tauri::State;

type ApiResult<T> = std::result::Result<T, String>;

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

/// Unlock the vault only after proving that the supplied passphrase decrypts
/// the authenticated vault payload. Never mark the vault unlocked optimistically.
#[tauri::command]
pub async fn secure_unlock_vault(
    state: State<'_, Arc<AppState>>,
    passphrase: String,
) -> ApiResult<()> {
    let vault = state.vault.lock().await;
    vault.verify_passphrase(&passphrase).map_err(err)?;
    drop(vault);

    let mut pw = state.passphrase.lock().await;
    *pw = Some(zeroize::Zeroizing::new(passphrase));
    Ok(())
}
