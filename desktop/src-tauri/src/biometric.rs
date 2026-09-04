use crate::state::AppState;
use std::sync::Arc;
use tauri::State;

type ApiResult<T> = std::result::Result<T, String>;

/// Keychain service and account identifiers for the stored vault passphrase.
const SERVICE: &str = "com.opentermius.vault";
const ACCOUNT: &str = "master-passphrase";

// ============================================================
// macOS implementation — Touch ID via Security framework
// ============================================================

#[cfg(target_os = "macos")]
mod macos {
    use super::{SERVICE, ACCOUNT};
    use security_framework::access_control::SecAccessControl;
    use security_framework::base::Error;
    use security_framework::passwords::{self, AccessControlOptions, PasswordOptions};

    /// Check whether Touch ID / biometry is available by attempting to create
    /// a `SecAccessControl` object that requires biometry. If the device has
    /// no biometric hardware or no fingerprints enrolled, the creation fails.
    pub fn biometry_available() -> bool {
        SecAccessControl::create_with_flags(AccessControlOptions::BIOMETRY_ANY.bits()).is_ok()
    }

    /// Check whether a keychain item exists **without triggering the biometric
    /// prompt**. Uses `SecItemCopyMatching` with `kSecReturnData` absent so only
    /// attributes are queried, not the encrypted data.
    pub fn passphrase_stored() -> bool {
        use core_foundation::base::TCFType;
        use core_foundation::dictionary::CFDictionary;
        use core_foundation_sys::base::{CFTypeRef, OSStatus};
        use security_framework_sys::keychain_item::SecItemCopyMatching;

        #[allow(deprecated)]
        let options = PasswordOptions::new_generic_password(SERVICE, ACCOUNT);
        #[allow(deprecated)]
        let params = CFDictionary::from_CFType_pairs(&options.query[..]);

        let mut ret: CFTypeRef = std::ptr::null();
        let status: OSStatus =
            unsafe { SecItemCopyMatching(params.as_concrete_TypeRef(), &mut ret) };

        // Any status other than "item not found" means the entry exists.
        status != security_framework_sys::base::errSecItemNotFound
    }

    /// Store the passphrase in the macOS keychain with biometric (Touch ID)
    /// access control. The passphrase can only be retrieved after the user
    /// successfully authenticates with Touch ID.
    pub fn store_passphrase(passphrase: &str) -> Result<(), String> {
        // Delete any existing item first — SecItemUpdate does not reliably
        // update access-control attributes, so we remove and re-add.
        let _ = passwords::delete_generic_password(SERVICE, ACCOUNT);

        let mut options = PasswordOptions::new_generic_password(SERVICE, ACCOUNT);
        options.set_access_control_options(AccessControlOptions::BIOMETRY_ANY);

        passwords::set_generic_password_options(passphrase.as_bytes(), options)
            .map_err(format_keychain_error)
    }

    /// Retrieve the passphrase from the keychain. This call blocks and
    /// triggers the Touch ID prompt. Returns a `Zeroizing<String>` so the
    /// passphrase is wiped from memory when dropped.
    pub fn retrieve_passphrase() -> Result<zeroize::Zeroizing<String>, String> {
        let data = passwords::generic_password(PasswordOptions::new_generic_password(
            SERVICE,
            ACCOUNT,
        ))
        .map_err(format_keychain_error)?;

        let s = String::from_utf8(data)
            .map_err(|e| format!("keychain data is not valid UTF-8: {e}"))?;
        Ok(zeroize::Zeroizing::new(s))
    }

    /// Delete the passphrase from the keychain.
    pub fn clear_passphrase() -> Result<(), String> {
        passwords::delete_generic_password(SERVICE, ACCOUNT)
            .map_err(format_keychain_error)
    }

    fn format_keychain_error(e: Error) -> String {
        let code = e.code();
        match code {
            security_framework_sys::base::errSecItemNotFound => {
                "no biometric passphrase stored".into()
            }
            // -25293: auth failed or user canceled the Touch ID prompt
            security_framework_sys::base::errSecAuthFailed => {
                "Touch ID authentication failed or was canceled".into()
            }
            // -128: user clicked "Cancel" on the Touch ID dialog
            -128 => "Touch ID was canceled by the user".into(),
            _ => format!("keychain error (code {code})"),
        }
    }
}

// ============================================================
// Non-macOS stub — biometric unlock unavailable
// ============================================================

#[cfg(not(target_os = "macos"))]
mod stub {
    pub fn biometry_available() -> bool {
        false
    }
    pub fn passphrase_stored() -> bool {
        false
    }
    pub fn store_passphrase(_passphrase: &str) -> Result<(), String> {
        Err("biometric unlock is not available on this platform".into())
    }
    pub fn retrieve_passphrase() -> Result<zeroize::Zeroizing<String>, String> {
        Err("biometric unlock is not available on this platform".into())
    }
    pub fn clear_passphrase() -> Result<(), String> {
        Err("biometric unlock is not available on this platform".into())
    }
}

#[cfg(target_os = "macos")]
use macos as platform;
#[cfg(not(target_os = "macos"))]
use stub as platform;

// ============================================================
// Tauri commands
// ============================================================

/// Returns `true` on macOS devices with Touch ID enrolled.
#[tauri::command]
pub async fn biometric_available() -> ApiResult<bool> {
    Ok(platform::biometry_available())
}

/// Checks whether a biometric passphrase has been stored in the keychain.
/// This does **not** trigger the Touch ID prompt.
#[tauri::command]
pub async fn biometric_passphrase_stored() -> ApiResult<bool> {
    Ok(platform::passphrase_stored())
}

/// Store the vault passphrase in the macOS keychain, protected by Touch ID.
/// The passphrase is verified against the vault before storing to prevent
/// saving an incorrect passphrase.
#[tauri::command]
pub async fn store_biometric_passphrase(
    state: State<'_, Arc<AppState>>,
    passphrase: String,
) -> ApiResult<()> {
    // Verify the passphrase can actually decrypt the vault before storing it.
    let vault = state.vault.lock().await;
    vault.verify_passphrase(&passphrase).map_err(|e| e.to_string())?;
    drop(vault);

    platform::store_passphrase(&passphrase)
}

/// Unlock the vault by retrieving the passphrase from the keychain.
/// On macOS this triggers the Touch ID prompt. The call blocks until the
/// user authenticates or cancels.
#[tauri::command]
pub async fn unlock_with_biometric(
    state: State<'_, Arc<AppState>>,
) -> ApiResult<bool> {
    // Keychain retrieval blocks the calling thread while the Touch ID dialog
    // is shown, so run it on a blocking thread to avoid stalling the runtime.
    let passphrase = tokio::task::spawn_blocking(platform::retrieve_passphrase)
        .await
        .map_err(|e| e.to_string())??;

    // Verify the retrieved passphrase against the vault.
    let vault = state.vault.lock().await;
    vault
        .verify_passphrase(passphrase.as_str())
        .map_err(|e| e.to_string())?;
    drop(vault);

    // Store the passphrase in AppState for the unlocked session.
    let mut pw = state.passphrase.lock().await;
    *pw = Some(passphrase);

    Ok(true)
}

/// Remove the biometric passphrase from the keychain.
#[tauri::command]
pub async fn clear_biometric_passphrase() -> ApiResult<()> {
    platform::clear_passphrase()
}
