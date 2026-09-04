import { defineStore } from "pinia";
import { ref, computed } from "vue";
import * as api from "../api";

const BIOMETRIC_ENABLED_KEY = "opentermius-biometric-enabled";

function loadBiometricEnabled(): boolean {
  try {
    return localStorage.getItem(BIOMETRIC_ENABLED_KEY) === "true";
  } catch {
    return false;
  }
}

function saveBiometricEnabled(enabled: boolean) {
  try {
    if (enabled) {
      localStorage.setItem(BIOMETRIC_ENABLED_KEY, "true");
    } else {
      localStorage.removeItem(BIOMETRIC_ENABLED_KEY);
    }
  } catch {
    // The backend credential remains authoritative; this marker is only used
    // to decide whether to offer/attempt biometric unlock without probing the
    // protected credential and triggering an authentication prompt.
  }
}

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
    initialized.value = await api.vaultIsInitialized();
    unlocked.value = await api.isVaultUnlocked();
    biometricAvailable.value = await api.biometricAvailable();
    biometricEnabled.value = biometricAvailable.value && loadBiometricEnabled();
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
      saveBiometricEnabled(true);
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
      saveBiometricEnabled(false);
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
