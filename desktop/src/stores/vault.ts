import { defineStore } from "pinia";
import { ref, computed } from "vue";
import * as api from "../api";

export const useVaultStore = defineStore("vault", () => {
  const initialized = ref(false);
  const unlocked = ref(false);
  const error = ref<string | null>(null);

  const needsSetup = computed(() => !initialized.value);
  const needsUnlock = computed(
    () => initialized.value && !unlocked.value,
  );

  async function checkStatus() {
    initialized.value = await api.vaultIsInitialized();
    unlocked.value = await api.isVaultUnlocked();
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

  async function lock() {
    await api.lockVault();
    unlocked.value = false;
  }

  return {
    initialized,
    unlocked,
    error,
    needsSetup,
    needsUnlock,
    checkStatus,
    initialize,
    unlock,
    lock,
  };
});
