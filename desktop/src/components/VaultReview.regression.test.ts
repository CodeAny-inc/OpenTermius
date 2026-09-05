import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount, type VueWrapper } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { nextTick } from "vue";
import * as api from "../api";
import { useUiStore } from "../stores/ui";
import { useVaultStore } from "../stores/vault";
import VaultUnlockModal from "./VaultUnlockModal.vue";
import VaultView from "./VaultView.vue";

vi.mock("../api", () => ({
  vaultIsInitialized: vi.fn(),
  isVaultUnlocked: vi.fn(),
  initializeVault: vi.fn(),
  unlockVault: vi.fn(),
  lockVault: vi.fn(),
  biometricAvailable: vi.fn(),
  biometricPassphraseStored: vi.fn(),
  unlockWithBiometric: vi.fn(),
  storeBiometricPassphrase: vi.fn(),
  clearBiometricPassphrase: vi.fn(),
}));

function buttonWithText(wrapper: VueWrapper, text: string) {
  const button = wrapper.findAll("button").find((item) => item.text().trim() === text);
  expect(button, `Missing button: ${text}`).toBeDefined();
  return button!;
}

// The modal remains mounted when hidden, so checking the DOM alone cannot
// establish that its reactive state has released a previously typed secret.
function modalState(wrapper: VueWrapper) {
  return wrapper.vm as unknown as { passphrase: string };
}

let pinia: ReturnType<typeof createPinia>;
let wrapper: VueWrapper | undefined;

beforeEach(() => {
  pinia = createPinia();
  setActivePinia(pinia);
  vi.resetAllMocks();
  vi.mocked(api.vaultIsInitialized).mockResolvedValue(true);
  vi.mocked(api.isVaultUnlocked).mockResolvedValue(false);
  vi.mocked(api.unlockVault).mockResolvedValue(undefined);
  vi.mocked(api.lockVault).mockResolvedValue(undefined);
  vi.mocked(api.unlockWithBiometric).mockResolvedValue(true);
  vi.mocked(api.biometricAvailable).mockResolvedValue(true);
  vi.mocked(api.biometricPassphraseStored).mockResolvedValue(true);
  useVaultStore().initialized = true;
});

afterEach(() => {
  wrapper?.unmount();
  wrapper = undefined;
  vi.restoreAllMocks();
});

function mountModal() {
  wrapper = mount(VaultUnlockModal, {
    global: { plugins: [pinia], stubs: { Teleport: true } },
  });
  return wrapper;
}

describe("SSH unlock modal secret cleanup", () => {
  it.each([false, true])(
    "clears the typed secret before biometric resolution (already unlocked: %s)",
    async (alreadyUnlocked) => {
      const vault = useVaultStore();
      const ui = useUiStore();
      vault.unlocked = alreadyUnlocked;
      const modal = mountModal();
      const request = ui.requestVaultUnlock();
      await flushPromises();
      await modal.get("#modal-pass").setValue("synthetic master passphrase");
      const state = modalState(modal);
      expect(state.passphrase).toBe("synthetic master passphrase");

      // Capture state before closing. The already-unlocked case also proves
      // success cleanup does not depend on a vault.unlocked transition.
      const atResolution: string[] = [];
      const resolveUnlock = ui.resolveVaultUnlock;
      vi.spyOn(ui, "resolveVaultUnlock").mockImplementation((success) => {
        atResolution.push(state.passphrase);
        resolveUnlock(success);
      });

      await buttonWithText(modal, "Unlock with Touch ID").trigger("click");
      await flushPromises();
      await expect(request).resolves.toBe(true);
      expect(atResolution).toEqual([""]);
      expect(state.passphrase).toBe("");
      expect(ui.showVaultUnlockModal).toBe(false);
      expect(api.unlockWithBiometric).toHaveBeenCalledTimes(1);
      expect(api.unlockVault).not.toHaveBeenCalled();

      await vault.lock();
      expect(state.passphrase).toBe("");
      const nextRequest = ui.requestVaultUnlock();
      await flushPromises();
      expect((modal.get("#modal-pass").element as HTMLInputElement).value).toBe("");
      expect((buttonWithText(modal, "Unlock").element as HTMLButtonElement).disabled).toBe(true);
      ui.resolveVaultUnlock(false);
      await expect(nextRequest).resolves.toBe(false);
    },
  );

  it("clears a programmatic close before the next render", async () => {
    const ui = useUiStore();
    const modal = mountModal();
    const request = ui.requestVaultUnlock();
    await flushPromises();
    await modal.get("#modal-pass").setValue("synthetic master passphrase");
    const state = modalState(modal);

    ui.resolveVaultUnlock(false);
    expect(state.passphrase).toBe("");
    await expect(request).resolves.toBe(false);
  });

  it("clears a same-tick close/reopen instead of retaining the previous secret", async () => {
    const ui = useUiStore();
    const modal = mountModal();
    const first = ui.requestVaultUnlock();
    await flushPromises();
    await modal.get("#modal-pass").setValue("synthetic master passphrase");
    const state = modalState(modal);

    ui.resolveVaultUnlock(false);
    const second = ui.requestVaultUnlock();
    expect(state.passphrase).toBe("");
    await nextTick();
    expect((modal.get("#modal-pass").element as HTMLInputElement).value).toBe("");
    await expect(first).resolves.toBe(false);
    ui.resolveVaultUnlock(false);
    await expect(second).resolves.toBe(false);
  });

  it.each(["unlock", "lock", "same-tick-lock"] as const)(
    "clears retained input on a store-driven %s transition",
    async (transition) => {
      const vault = useVaultStore();
      const ui = useUiStore();
      vault.unlocked = transition !== "unlock";
      const modal = mountModal();
      const request = ui.requestVaultUnlock();
      await flushPromises();
      await modal.get("#modal-pass").setValue("synthetic master passphrase");
      const state = modalState(modal);

      if (transition === "unlock") {
        vault.unlocked = true;
      } else if (transition === "lock") {
        // Automatic locks call the store, not the modal's submit/cancel paths.
        await vault.lock();
      } else {
        vault.unlocked = false;
        vault.unlocked = true;
      }
      expect(state.passphrase).toBe("");
      ui.resolveVaultUnlock(false);
      await expect(request).resolves.toBe(false);
    },
  );

  it("clears retained setup state when the modal component is unmounted", async () => {
    const ui = useUiStore();
    const modal = mountModal();
    const request = ui.requestVaultUnlock();
    await flushPromises();
    await modal.get("#modal-pass").setValue("synthetic master passphrase");
    const state = modalState(modal);

    modal.unmount();
    wrapper = undefined;
    expect(state.passphrase).toBe("");
    ui.resolveVaultUnlock(false);
    await expect(request).resolves.toBe(false);
  });
});

describe("VaultView failed biometric reconciliation", () => {
  it.each([false, true])(
    "shows a real store reconciliation failure with both biometric flags false (unlocked: %s)",
    async (unlocked) => {
      vi.mocked(api.isVaultUnlocked).mockResolvedValue(unlocked);
      vi.mocked(api.biometricPassphraseStored).mockRejectedValue(
        new Error("Keychain access requires valid entitlements"),
      );
      // Do not mock checkStatus: exercise the actual store's fail-closed path.
      wrapper = mount(VaultView, { global: { plugins: [pinia] } });
      await flushPromises();
      const vault = useVaultStore();
      expect(vault.unlocked).toBe(unlocked);
      expect(vault.biometricAvailable).toBe(false);
      expect(vault.biometricEnabled).toBe(false);
      expect(api.biometricPassphraseStored).toHaveBeenCalledTimes(1);
      expect(vault.error).toContain("Keychain access requires valid entitlements");
      expect(wrapper.text()).toContain("Keychain access requires valid entitlements");
      expect(wrapper.text()).not.toContain("Touch ID / Biometric Unlock");
      expect(api.unlockWithBiometric).not.toHaveBeenCalled();

      // Error visibility must not accidentally advertise biometric controls.
      if (unlocked) {
        expect(buttonWithText(wrapper, "Lock Vault Now").exists()).toBe(true);
      } else {
        expect(wrapper.find("#unlock-pass").exists()).toBe(true);
      }
    },
  );

  it("does not duplicate an unlocked runtime error when biometric settings are visible", async () => {
    vi.mocked(api.isVaultUnlocked).mockResolvedValue(true);
    wrapper = mount(VaultView, { global: { plugins: [pinia] } });
    await flushPromises();
    useVaultStore().error = "Synthetic biometric operation failure";
    await nextTick();

    expect(wrapper.text()).toContain("Touch ID / Biometric Unlock");
    const messages = wrapper.findAll("p").filter(
      (item) => item.text() === "Synthetic biometric operation failure",
    );
    expect(messages).toHaveLength(1);
  });
});
