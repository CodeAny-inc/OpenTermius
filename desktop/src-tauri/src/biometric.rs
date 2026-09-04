use crate::state::AppState;
use std::sync::Arc;
use tauri::State;

type ApiResult<T> = std::result::Result<T, String>;

/// Biometric unlock is deliberately disabled until each supported platform has
/// an implementation that binds secret retrieval to OS-enforced user presence.
/// Generic Keychain/Credential Manager storage is not equivalent to Touch ID or
/// Windows Hello and must not be advertised as biometric protection.
#[tauri::command]
pub async fn biometric_available() -> ApiResult<bool> {
    Ok(false)
}

/// Kept for frontend API compatibility while biometric unlock is unavailable.
/// This must not probe a credential store because a real protected credential
/// lookup may itself trigger an authentication prompt.
#[tauri::command]
pub async fn biometric_passphrase_stored() -> ApiResult<bool> {
    Ok(false)
}

#[tauri::command]
pub async fn store_biometric_passphrase(
    _state: State<'_, Arc<AppState>>,
    _passphrase: String,
) -> ApiResult<()> {
    Err("biometric unlock is not available in this build".into())
}

#[tauri::command]
pub async fn unlock_with_biometric(
    _state: State<'_, Arc<AppState>>,
) -> ApiResult<bool> {
    Err("biometric unlock is not available in this build".into())
}

#[tauri::command]
pub async fn clear_biometric_passphrase() -> ApiResult<()> {
    Err("biometric unlock is not available in this build".into())
}
