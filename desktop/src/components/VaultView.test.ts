import { flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { describe, expect, it, vi } from "vitest";
import VaultView from "./VaultView.vue";
import { useVaultStore } from "../stores/vault";

describe("VaultView biometric settings", () => {
  it("shows the Keychain error when disabling biometric unlock fails", async () => {
    const pinia = createPinia();
    setActivePinia(pinia);
    const vault = useVaultStore();

    vault.initialized = true;
    vault.unlocked = true;
    vault.biometricAvailable = true;
    vault.biometricEnabled = true;

    vi.spyOn(vault, "checkStatus").mockResolvedValue(undefined);
    const disableBiometric = vi
      .spyOn(vault, "disableBiometric")
      .mockRejectedValue(new Error("Keychain delete failed"));

    const wrapper = mount(VaultView, {
      global: {
        plugins: [pinia],
      },
    });
    await flushPromises();

    const disableButton = wrapper
      .findAll("button")
      .find((button) => button.text().includes("Disable biometric unlock"));
    expect(disableButton).toBeDefined();

    await disableButton!.trigger("click");
    await flushPromises();

    expect(disableBiometric).toHaveBeenCalledTimes(1);
    expect(wrapper.text()).toContain("Error: Keychain delete failed");

    wrapper.unmount();
  });
});
