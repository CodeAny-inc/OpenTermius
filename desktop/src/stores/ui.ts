import { defineStore } from "pinia";
import { ref } from "vue";

export const useUiStore = defineStore("ui", () => {
  // Vault unlock modal
  const showVaultUnlockModal = ref(false);
  let vaultUnlockResolve: ((success: boolean) => void) | null = null;

  // Fullscreen pane
  const fullscreenPaneId = ref<string | null>(null);

  function requestVaultUnlock(): Promise<boolean> {
    return new Promise((resolve) => {
      vaultUnlockResolve = resolve;
      showVaultUnlockModal.value = true;
    });
  }

  function resolveVaultUnlock(success: boolean) {
    if (vaultUnlockResolve) {
      vaultUnlockResolve(success);
      vaultUnlockResolve = null;
    }
    showVaultUnlockModal.value = false;
  }

  function toggleFullscreen(paneId: string) {
    if (fullscreenPaneId.value === paneId) {
      fullscreenPaneId.value = null;
    } else {
      fullscreenPaneId.value = paneId;
    }
  }

  function exitFullscreen() {
    fullscreenPaneId.value = null;
  }

  return {
    showVaultUnlockModal,
    fullscreenPaneId,
    requestVaultUnlock,
    resolveVaultUnlock,
    toggleFullscreen,
    exitFullscreen,
  };
});
