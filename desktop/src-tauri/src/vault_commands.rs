use crate::state::AppState;
use opentermius_core::vault::Vault;
use std::sync::Arc;
use tauri::State;

type ApiResult<T> = std::result::Result<T, String>;

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

fn ensure_vault_uninitialized(vault: &Vault) -> ApiResult<()> {
    if vault.is_initialized() {
        return Err("vault already initialized".into());
    }
    Ok(())
}

/// Create a brand-new vault only after proving that no initialized vault is
/// already present and invalidating any biometric credential left behind by an
/// older/reset vault. Keeping the vault lock across the guard, cleanup, and
/// initialization prevents concurrent initialization from bypassing the check.
#[tauri::command]
pub async fn secure_initialize_vault(
    state: State<'_, Arc<AppState>>,
    passphrase: String,
) -> ApiResult<()> {
    let passphrase = zeroize::Zeroizing::new(passphrase);
    let mut vault = state.vault.lock().await;

    // This must happen before Keychain cleanup: a mistaken initialize call must
    // not delete the credential belonging to an existing vault.
    ensure_vault_uninitialized(&vault)?;
    crate::biometric::clear_for_vault_initialization()?;
    vault.initialize(passphrase.as_str()).map_err(err)?;
    drop(vault);

    // Treat initialization as a new authentication epoch so any biometric
    // request that started against stale state cannot later unlock this vault.
    // Invalidate while holding the passphrase mutex so a concurrent lock always
    // wins and leaves the final backend state locked.
    let mut pw = state.passphrase.lock().await;
    state.auth_generation.invalidate();
    *pw = Some(passphrase);
    Ok(())
}

/// Unlock the vault only after proving that the supplied passphrase decrypts
/// the authenticated vault payload. A lock that happens while verification is
/// in flight invalidates this attempt before it can commit the passphrase.
#[tauri::command]
pub async fn secure_unlock_vault(
    state: State<'_, Arc<AppState>>,
    passphrase: String,
) -> ApiResult<()> {
    let generation = state.auth_generation.current();
    let passphrase = zeroize::Zeroizing::new(passphrase);

    let vault = state.vault.lock().await;
    vault.verify_passphrase(passphrase.as_str()).map_err(err)?;
    drop(vault);

    let mut pw = state.passphrase.lock().await;
    if !state.auth_generation.is_current(generation) {
        return Err("vault unlock was superseded by a newer lock".into());
    }
    *pw = Some(passphrase);
    Ok(())
}

/// Lock the vault and invalidate every unlock attempt that started before this
/// operation. The generation is advanced before waiting for the passphrase
/// mutex, ensuring a pending Touch ID request cannot commit after this lock.
#[tauri::command]
pub async fn secure_lock_vault(state: State<'_, Arc<AppState>>) -> ApiResult<()> {
    state.auth_generation.invalidate();
    let mut pw = state.passphrase.lock().await;
    *pw = None;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::ensure_vault_uninitialized;
    use opentermius_core::vault::Vault;

    #[test]
    fn rejects_initialization_when_a_vault_already_exists() {
        let path = std::env::temp_dir().join(format!(
            "opentermius-vault-command-test-{}.json",
            uuid::Uuid::new_v4()
        ));
        let mut vault = Vault::open(path.clone()).expect("open vault");

        assert!(ensure_vault_uninitialized(&vault).is_ok());
        vault
            .initialize("correct horse battery staple")
            .expect("initialize vault");
        assert_eq!(
            ensure_vault_uninitialized(&vault).unwrap_err(),
            "vault already initialized"
        );

        let _ = std::fs::remove_file(path);
    }
}
