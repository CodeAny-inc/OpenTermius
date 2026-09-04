use crate::state::AppState;
use std::sync::Arc;
use tauri::State;

type ApiResult<T> = std::result::Result<T, String>;

/// Keychain service and account identifiers for the stored vault passphrase.
#[cfg(all(target_os = "macos", feature = "macos-biometric"))]
const SERVICE: &str = "com.opentermius.vault";
#[cfg(all(target_os = "macos", feature = "macos-biometric"))]
const ACCOUNT: &str = "master-passphrase";

// ============================================================
// macOS implementation — Touch ID via Security framework
// ============================================================

#[cfg(all(target_os = "macos", feature = "macos-biometric"))]
mod macos {
    use super::{ACCOUNT, SERVICE};
    use core_foundation::base::TCFType;
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::string::CFString;
    use core_foundation_sys::base::{CFTypeRef, OSStatus};
    use core_foundation_sys::string::CFStringRef;
    use objc2_local_authentication::{LAContext, LAPolicy};
    use security_framework::access_control::{ProtectionMode, SecAccessControl};
    use security_framework::base::Error;
    use security_framework::passwords::{self, AccessControlOptions, PasswordOptions};
    use security_framework_sys::base::{errSecAuthFailed, errSecItemNotFound, errSecSuccess};
    use security_framework_sys::item::kSecUseAuthenticationUI;
    use security_framework_sys::keychain_item::SecItemCopyMatching;

    // Security.framework does not currently expose these through
    // security-framework-sys. Keep the numeric OSStatus values local and map
    // them to explicit, fail-closed errors below.
    const ERR_SEC_INTERACTION_NOT_ALLOWED: OSStatus = -25308;
    const ERR_SEC_MISSING_ENTITLEMENT: OSStatus = -34018;

    // `kSecUseAuthenticationUIFail` is deprecated by Apple in favor of an
    // LAContext with interactionNotAllowed, but it remains the most direct way
    // to guarantee this existence-only query never presents authentication UI
    // without bridging an Objective-C LAContext through CoreFoundation FFI.
    #[link(name = "Security", kind = "framework")]
    extern "C" {
        static kSecUseAuthenticationUIFail: CFStringRef;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum PassphraseState {
        Stored,
        Missing,
        Invalidated,
    }

    fn password_options() -> PasswordOptions {
        let mut options = PasswordOptions::new_generic_password(SERVICE, ACCOUNT);
        // On macOS, SecItem defaults to the legacy file-based keychain. Access
        // control backed by Touch ID must live in the Data Protection Keychain.
        options.use_protected_keychain();
        options
    }

    /// Return whether this Mac can currently evaluate a biometric-only policy.
    /// On macOS this corresponds to Touch ID availability/enrollment and also
    /// accounts for temporary states such as biometric lockout.
    pub fn biometry_available() -> bool {
        unsafe {
            let context = LAContext::new();
            context
                .canEvaluatePolicy_error(LAPolicy::DeviceOwnerAuthenticationWithBiometrics)
                .is_ok()
        }
    }

    fn classify_passphrase_status(status: OSStatus) -> Result<PassphraseState, String> {
        match status {
            errSecSuccess | ERR_SEC_INTERACTION_NOT_ALLOWED => Ok(PassphraseState::Stored),
            errSecItemNotFound => Ok(PassphraseState::Missing),
            // An item protected with BIOMETRY_CURRENT_SET can become invalid
            // after fingerprints are added or removed. Treat that expected
            // lifecycle state as replaceable rather than making re-enrollment
            // impossible; the command caller has already verified the master
            // passphrase before store_passphrase() is reached.
            errSecAuthFailed => Ok(PassphraseState::Invalidated),
            code => Err(format_keychain_error(Error::from_code(code))),
        }
    }

    /// Probe the protected credential without displaying authentication UI.
    /// A valid biometric item requires interaction and therefore reports as
    /// `Stored`; an item invalidated by enrollment changes is distinguished so
    /// callers can safely offer explicit re-enrollment with the master password.
    fn passphrase_state() -> Result<PassphraseState, String> {
        let mut options = password_options();

        #[allow(deprecated)]
        unsafe {
            options.query.push((
                CFString::wrap_under_get_rule(kSecUseAuthenticationUI),
                CFString::wrap_under_get_rule(kSecUseAuthenticationUIFail).into_CFType(),
            ));
        }

        #[allow(deprecated)]
        let params = CFDictionary::from_CFType_pairs(&options.query[..]);

        let mut ret: CFTypeRef = std::ptr::null();
        let status: OSStatus =
            unsafe { SecItemCopyMatching(params.as_concrete_TypeRef(), &mut ret) };

        classify_passphrase_status(status)
    }

    /// Check whether a currently usable protected passphrase exists without
    /// displaying an authentication prompt. An item invalidated by Touch ID
    /// enrollment changes is reported as not stored so the UI can re-enable it.
    pub fn passphrase_stored() -> Result<bool, String> {
        Ok(matches!(passphrase_state()?, PassphraseState::Stored))
    }

    /// Store the passphrase in the Data Protection Keychain with biometric
    /// (Touch ID) access control. `BIOMETRY_CURRENT_SET` binds the credential
    /// to the Touch ID enrollment that exists at enable time, so adding or
    /// removing fingerprints invalidates the stored passphrase and requires
    /// the user to re-enable biometric unlock with the master passphrase.
    pub fn store_passphrase(passphrase: &str) -> Result<(), String> {
        match passphrase_state()? {
            PassphraseState::Stored => {
                return Err(concat!(
                    "biometric unlock is already enabled; disable it before replacing ",
                    "the protected credential"
                )
                .into());
            }
            PassphraseState::Missing | PassphraseState::Invalidated => {
                // The caller already verified the master passphrase against the
                // vault. Missing and invalidated credentials are therefore safe
                // to clean up before creating a fresh current-set item.
                delete_if_present()?;
            }
        }

        // Build the access-control object explicitly instead of using
        // PasswordOptions::set_access_control_options(), which unwraps ACL
        // creation internally. The ThisDeviceOnly accessibility class prevents
        // the raw vault passphrase from migrating to another device via backup.
        let access_control = SecAccessControl::create_with_protection(
            Some(ProtectionMode::AccessibleWhenUnlockedThisDeviceOnly),
            AccessControlOptions::BIOMETRY_CURRENT_SET.bits(),
        )
        .map_err(|e| {
            format!(
                "failed to create Touch ID access control: {}",
                format_keychain_error(e)
            )
        })?;

        let mut options = password_options();
        options.set_access_control(access_control);

        passwords::set_generic_password_options(passphrase.as_bytes(), options)
            .map_err(format_keychain_error)
    }

    /// Retrieve the passphrase from the Data Protection Keychain. This blocks
    /// while macOS presents the Touch ID prompt.
    pub fn retrieve_passphrase() -> Result<zeroize::Zeroizing<String>, String> {
        let data = passwords::generic_password(password_options()).map_err(format_keychain_error)?;
        let s = String::from_utf8(data)
            .map_err(|e| format!("keychain data is not valid UTF-8: {e}"))?;
        Ok(zeroize::Zeroizing::new(s))
    }

    /// Delete the protected passphrase. Clearing an already-missing item is
    /// intentionally idempotent so disabling biometric unlock is reliable.
    pub fn clear_passphrase() -> Result<(), String> {
        delete_if_present()
    }

    fn delete_if_present() -> Result<(), String> {
        match passwords::delete_generic_password_options(password_options()) {
            Ok(()) => Ok(()),
            Err(e) if e.code() == errSecItemNotFound => Ok(()),
            Err(e) => Err(format_keychain_error(e)),
        }
    }

    fn format_keychain_error(e: Error) -> String {
        let code = e.code();
        match code {
            errSecItemNotFound => "no biometric passphrase stored".into(),
            errSecAuthFailed => "Touch ID authentication failed or was canceled".into(),
            // errSecUserCanceled
            -128 => "Touch ID was canceled by the user".into(),
            ERR_SEC_INTERACTION_NOT_ALLOWED => {
                "Touch ID interaction is not allowed in the current context".into()
            }
            ERR_SEC_MISSING_ENTITLEMENT => concat!(
                "biometric keychain access requires a properly signed macOS app ",
                "with valid Data Protection Keychain entitlements"
            )
            .into(),
            _ => match e.message() {
                Some(message) => format!("keychain error (code {code}): {message}"),
                None => format!("keychain error (code {code})"),
            },
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{
            classify_passphrase_status, PassphraseState, ERR_SEC_INTERACTION_NOT_ALLOWED,
        };
        use security_framework_sys::base::{errSecAuthFailed, errSecItemNotFound, errSecSuccess};

        #[test]
        fn classifies_invalidated_biometry_as_replaceable() {
            assert_eq!(
                classify_passphrase_status(errSecAuthFailed).unwrap(),
                PassphraseState::Invalidated
            );
            assert_eq!(
                classify_passphrase_status(errSecItemNotFound).unwrap(),
                PassphraseState::Missing
            );
            assert_eq!(
                classify_passphrase_status(errSecSuccess).unwrap(),
                PassphraseState::Stored
            );
            assert_eq!(
                classify_passphrase_status(ERR_SEC_INTERACTION_NOT_ALLOWED).unwrap(),
                PassphraseState::Stored
            );
        }
    }
}

// ============================================================
// Stub — biometric unlock unavailable
// ============================================================

// The stub is used on non-macOS platforms and on ordinary/ad-hoc-signed macOS
// builds. `macos-biometric` is intentionally opt-in until the release pipeline
// provides valid Apple signing and Data Protection Keychain entitlements.
#[cfg(not(all(target_os = "macos", feature = "macos-biometric")))]
mod stub {
    pub fn biometry_available() -> bool {
        false
    }
    pub fn passphrase_stored() -> Result<bool, String> {
        Ok(false)
    }
    pub fn store_passphrase(_passphrase: &str) -> Result<(), String> {
        Err("biometric unlock is not available in this build".into())
    }
    pub fn retrieve_passphrase() -> Result<zeroize::Zeroizing<String>, String> {
        Err("biometric unlock is not available in this build".into())
    }
    pub fn clear_passphrase() -> Result<(), String> {
        // Clearing is an idempotent lifecycle operation. Unsupported builds
        // have no biometric credential to remove, so treating this as success
        // lets new-vault initialization perform unconditional stale-item cleanup.
        Ok(())
    }
}

#[cfg(all(target_os = "macos", feature = "macos-biometric"))]
use macos as platform;
#[cfg(not(all(target_os = "macos", feature = "macos-biometric")))]
use stub as platform;

/// Internal lifecycle hook used before a brand-new vault is created. Keeping
/// this at the backend boundary prevents callers from bypassing stale-Keychain
/// cleanup by invoking the vault initialization command directly.
pub(crate) fn clear_for_vault_initialization() -> ApiResult<()> {
    platform::clear_passphrase()
}

// ============================================================
// Tauri commands
// ============================================================

/// Returns `true` when this build enables biometric support and this Mac can
/// currently evaluate Touch ID. Ordinary/ad-hoc-signed builds return `false`.
#[tauri::command]
pub async fn biometric_available() -> ApiResult<bool> {
    Ok(platform::biometry_available())
}

/// Checks whether a protected biometric passphrase is stored without showing
/// an authentication prompt.
#[tauri::command]
pub async fn biometric_passphrase_stored() -> ApiResult<bool> {
    platform::passphrase_stored()
}

/// Store the vault passphrase in the OS keychain, protected by Touch ID.
/// The passphrase is verified against the vault before storing to prevent
/// saving an incorrect passphrase.
#[tauri::command]
pub async fn store_biometric_passphrase(
    state: State<'_, Arc<AppState>>,
    passphrase: String,
) -> ApiResult<()> {
    let passphrase = zeroize::Zeroizing::new(passphrase);

    let vault = state.vault.lock().await;
    vault
        .verify_passphrase(passphrase.as_str())
        .map_err(|e| e.to_string())?;
    drop(vault);

    platform::store_passphrase(passphrase.as_str())
}

/// Unlock the vault by retrieving the passphrase from the keychain. On an
/// enabled macOS build this triggers the Touch ID prompt. The blocking OS call
/// runs off the async runtime so the application stays responsive while
/// authentication is open. A newer lock invalidates the attempt before commit.
#[tauri::command]
pub async fn unlock_with_biometric(state: State<'_, Arc<AppState>>) -> ApiResult<bool> {
    let generation = state.auth_generation.current();
    let passphrase = tokio::task::spawn_blocking(platform::retrieve_passphrase)
        .await
        .map_err(|e| e.to_string())??;

    let vault = state.vault.lock().await;
    vault
        .verify_passphrase(passphrase.as_str())
        .map_err(|e| e.to_string())?;
    drop(vault);

    let mut pw = state.passphrase.lock().await;
    if !state.auth_generation.is_current(generation) {
        return Err("biometric unlock was superseded by a newer vault lock".into());
    }
    *pw = Some(passphrase);

    Ok(true)
}

/// Remove the biometric passphrase from the OS keychain. This is idempotent;
/// unsupported builds have no biometric credential and therefore succeed.
#[tauri::command]
pub async fn clear_biometric_passphrase() -> ApiResult<()> {
    clear_for_vault_initialization()
}
