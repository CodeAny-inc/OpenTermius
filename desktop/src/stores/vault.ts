import { defineStore } from "pinia";
import { ref, computed } from "vue";
import * as api from "../api";

export const useVaultStore = defineStore("vault", () => {
  const initialized = ref(false);
  const unlocked = ref(false);
  const error = ref<string | null>(null);
  const biometricAvailable = ref(false);
  const biometricEnabled = ref(false);

  const needsSetup = computed(() => !initialized.value);
  const needsUnlock = computed(
    () => initialized.value && !unlocked.value,
  );

  async function reconcileBiometricState() {
    biometricEnabled.value = false;
    if (!initialized.value) return;

    // Credential existence and current biometric availability are deliberately
    // independent. canEvaluatePolicy can be false during a temporary Touch ID
    // lockout, but the protected Keychain item remains enabled and should become
    // usable again once LocalAuthentication can evaluate the policy.
    biometricEnabled.value = await api.biometricPassphraseStored();
  }

  async function refreshBiometricState() {
    try {
      biometricAvailable.value = await api.biometricAvailable();
      await reconcileBiometricState();
    } catch (e) {
      // A failed capability or Keychain probe must never leave stale UI state
      // advertising an unlock path whose current status could not be verified.
      biometricAvailable.value = false;
      biometricEnabled.value = false;
      throw e;
    }
  }

  async function checkStatus() {
    error.value = null;
    initialized.value = await api.vaultIsInitialized();
    unlocked.value = await api.isVaultUnlocked();

    try {
      // The protected Keychain item is the source of truth only for an existing
      // vault. Current LA availability is refreshed separately because it can
      // change at runtime without changing whether the credential is enrolled.
      await refreshBiometricState();
    } catch (e) {
      error.value = String(e);
    }
  }

  async function initialize(passphrase: string) {
    error.value = null;
    try {
      // The backend owns the entire initialization boundary: it first rejects
      // an already-initialized vault, then handles legacy Keychain cleanup and
      // creates the new vault. Do not clear biometric state before that guard,
      // or a stale frontend call could delete credentials for an existing vault.
      const unlockedAfterInitialization = await api.initializeVault(passphrase);
      initialized.value = true;
      unlocked.value = unlockedAfterInitialization;
      biometricEnabled.value = false;
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function unlock(passphrase: string) {
    error.value = null;
    try {
      await api.unlockVault(passphrase);
      unlocked.value = true;
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function unlockWithBiometric() {
    error.value = null;
    try {
      const success = await api.unlockWithBiometric();
      if (success) {
        unlocked.value = true;
      } else {
        throw new Error("Biometric unlock failed");
      }
    } catch (e) {
      const unlockError = e;
      try {
        // A failed attempt can itself change LocalAuthentication availability
        // (for example by entering biometric lockout), so refresh both pieces of
        // state while preserving the credential-enrollment bit independently.
        await refreshBiometricState();
      } catch {
        // refreshBiometricState already fails closed. Preserve the original
        // unlock error for the user instead of replacing it with a probe error.
      }
      error.value = String(unlockError);
      throw unlockError;
    }
  }

  async function enableBiometric(passphrase: string) {
    error.value = null;
    try {
      await api.storeBiometricPassphrase(passphrase);
      biometricEnabled.value = true;
    } catch (e) {
      const enableError = e;
      try {
        await reconcileBiometricState();
      } catch {
        // Fail closed and keep the original operation error.
      }
      error.value = String(enableError);
      throw enableError;
    }
  }

  async function disableBiometric() {
    error.value = null;
    try {
      await api.clearBiometricPassphrase();
      biometricEnabled.value = false;
    } catch (e) {
      const disableError = e;
      try {
        await reconcileBiometricState();
      } catch {
        // Fail closed and keep the original operation error.
      }
      error.value = String(disableError);
      throw disableError;
    }
  }

  async function lock() {
    await api.lockVault();
    unlocked.value = false;
  }

  return {
    initialized,
    unlocked,
    error,
    biometricAvailable,
    biometricEnabled,
    needsSetup,
    needsUnlock,
    checkStatus,
    refreshBiometricState,
    initialize,
    unlock,
    unlockWithBiometric,
    enableBiometric,
    disableBiometric,
    lock,
  };
});
