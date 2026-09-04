import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount, type VueWrapper } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { nextTick } from "vue";
import VaultUnlockModal from "./VaultUnlockModal.vue";
import { useUiStore } from "../stores/ui";
import { useVaultStore } from "../stores/vault";

describe("VaultUnlockModal", () => {
  let wrapper: VueWrapper;

  beforeEach(() => {
    const pinia = createPinia();
    setActivePinia(pinia);
    wrapper = mount(VaultUnlockModal, {
      global: {
        plugins: [pinia],
        stubs: {
          Teleport: true,
        },
      },
    });
  });

  afterEach(() => {
    wrapper.unmount();
  });

  it("offers Touch ID for the SSH unlock flow and resolves on success", async () => {
    const vault = useVaultStore();
    const ui = useUiStore();
    vault.biometricAvailable = true;
    vault.biometricEnabled = true;

    const unlockWithBiometric = vi
      .spyOn(vault, "unlockWithBiometric")
      .mockResolvedValue(undefined);

    const unlockRequest = ui.requestVaultUnlock();
    await nextTick();

    const touchIdButton = wrapper
      .findAll("button")
      .find((button) => button.text().includes("Unlock with Touch ID"));

    expect(touchIdButton).toBeDefined();
    await touchIdButton!.trigger("click");

    expect(unlockWithBiometric).toHaveBeenCalledTimes(1);
    await expect(unlockRequest).resolves.toBe(true);
    expect(ui.showVaultUnlockModal).toBe(false);
  });

  it("blocks duplicate actions and cancellation while Touch ID is pending", async () => {
    const vault = useVaultStore();
    const ui = useUiStore();
    vault.biometricAvailable = true;
    vault.biometricEnabled = true;

    let finishBiometric!: () => void;
    const unlockWithBiometric = vi
      .spyOn(vault, "unlockWithBiometric")
      .mockImplementation(
        () =>
          new Promise<void>((resolve) => {
            finishBiometric = resolve;
          }),
      );

    const unlockRequest = ui.requestVaultUnlock();
    await nextTick();

    const touchIdButton = wrapper
      .findAll("button")
      .find((button) => button.text().includes("Unlock with Touch ID"));
    expect(touchIdButton).toBeDefined();

    await touchIdButton!.trigger("click");
    await nextTick();

    expect(unlockWithBiometric).toHaveBeenCalledTimes(1);
    expect((touchIdButton!.element as HTMLButtonElement).disabled).toBe(true);
    expect((wrapper.get("#modal-pass").element as HTMLInputElement).disabled).toBe(true);

    const cancelButton = wrapper
      .findAll("button")
      .find((button) => button.text().trim() === "Cancel");
    const passwordUnlockButton = wrapper
      .findAll("button")
      .find((button) => button.text().trim() === "Unlock");

    expect((cancelButton!.element as HTMLButtonElement).disabled).toBe(true);
    expect((passwordUnlockButton!.element as HTMLButtonElement).disabled).toBe(true);

    await touchIdButton!.trigger("click");
    expect(unlockWithBiometric).toHaveBeenCalledTimes(1);
    expect(ui.showVaultUnlockModal).toBe(true);

    finishBiometric();
    await flushPromises();

    await expect(unlockRequest).resolves.toBe(true);
    expect(ui.showVaultUnlockModal).toBe(false);
  });
});
