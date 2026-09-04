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

  async function checkStatus() {
    error.value = null;
    initialized.value = await api.vaultIsInitialized();
    unlocked.value = await api.isVaultUnlocked();
    biometricAvailable.value = await api.biometricAvailable();
    biometricEnabled.value = false;

    if (biometricAvailable.value) {
      try {
        // The protected Keychain item is the source of truth. This backend
        // query is explicitly non-interactive and does not show Touch ID UI.
        biometricEnabled.value = await api.biometricPassphraseStored();
      } catch (e) {
        // Fail closed: never advertise biometric unlock if the Keychain state
        // cannot be established reliably.
        error.value = String(e);
      }
    }
  }

  async function initialize(passphrase: string) {
    error.value = null;
    try {
      await api.initializeVault(passphrase);
      initialized.value = true;
      unlocked.value = true;
    } catch (e) {
      error.value = String(e);
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
      error.value = String(e);
      throw e;
    }
  }

  async function enableBiometric(passphrase: string) {
    error.value = null;
    try {
      await api.storeBiometricPassphrase(passphrase);
      biometricEnabled.value = true;
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function disableBiometric() {
    error.value = null;
    try {
      await api.clearBiometricPassphrase();
      biometricEnabled.value = false;
    } catch (e) {
      error.value = String(e);
      throw e;
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
    initialize,
    unlock,
    unlockWithBiometric,
    enableBiometric,
    disableBiometric,
    lock,
  };
});
