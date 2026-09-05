import { beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import * as api from "../api";
import { useVaultStore } from "./vault";

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

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: Error) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}

describe("biometric state ordering", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.resetAllMocks();
    vi.mocked(api.biometricAvailable).mockResolvedValue(true);
    vi.mocked(api.biometricPassphraseStored).mockResolvedValue(false);
    vi.mocked(api.storeBiometricPassphrase).mockResolvedValue(undefined);
    vi.mocked(api.clearBiometricPassphrase).mockResolvedValue(undefined);
    vi.mocked(api.initializeVault).mockResolvedValue(true);
    const vault = useVaultStore();
    vault.initialized = true;
    vault.biometricAvailable = true;
  });

  it.each(["enable", "disable"] as const)(
    "ignores a probe that completes after a successful %s",
    async (operation) => {
      const vault = useVaultStore();
      const previousEnrollment = operation === "disable";
      vault.biometricEnabled = previousEnrollment;
      const probe = deferred<boolean>();
      vi.mocked(api.biometricPassphraseStored).mockReturnValueOnce(probe.promise);
      const refresh = vault.refreshBiometricState();
      await flushPromises();
      expect(api.biometricPassphraseStored).toHaveBeenCalledTimes(1);

      if (operation === "enable") {
        await vault.enableBiometric("correct horse battery staple");
      } else {
        await vault.disableBiometric();
      }
      expect(vault.biometricEnabled).toBe(!previousEnrollment);

      probe.resolve(previousEnrollment);
      await refresh;
      expect(vault.biometricEnabled).toBe(!previousEnrollment);
    },
  );

  it("ignores a stale probe failure after enrollment succeeds", async () => {
    const vault = useVaultStore();
    const probe = deferred<boolean>();
    vi.mocked(api.biometricPassphraseStored).mockReturnValueOnce(probe.promise);
    const refresh = vault.refreshBiometricState();
    await flushPromises();

    await vault.enableBiometric("correct horse battery staple");
    probe.reject(new Error("obsolete Keychain error"));
    await expect(refresh).resolves.toBeUndefined();
    expect(vault.biometricEnabled).toBe(true);
    expect(vault.biometricAvailable).toBe(true);
    expect(vault.error).toBeNull();
  });

  it.each(["success", "failure"] as const)(
    "keeps the latest refresh when an older probe ends in %s",
    async (outcome) => {
      const vault = useVaultStore();
      const probe = deferred<boolean>();
      vi.mocked(api.biometricPassphraseStored)
        .mockReturnValueOnce(probe.promise)
        .mockResolvedValueOnce(true);
      const older = vault.refreshBiometricState();
      await flushPromises();
      await vault.refreshBiometricState();
      expect(vault.biometricEnabled).toBe(true);

      if (outcome === "success") probe.resolve(false);
      else probe.reject(new Error("obsolete Keychain error"));
      await expect(older).resolves.toBeUndefined();
      expect(vault.biometricEnabled).toBe(true);
      expect(vault.biometricAvailable).toBe(true);
    },
  );

  it("does not start a credential probe from a superseded capability response", async () => {
    const vault = useVaultStore();
    const availability = deferred<boolean>();
    vi.mocked(api.biometricAvailable).mockReturnValueOnce(availability.promise);
    const refresh = vault.refreshBiometricState();
    await flushPromises();
    expect(api.biometricAvailable).toHaveBeenCalledTimes(1);

    await vault.enableBiometric("correct horse battery staple");
    availability.resolve(false);
    await refresh;
    expect(api.biometricPassphraseStored).not.toHaveBeenCalled();
    expect(vault.biometricEnabled).toBe(true);
    expect(vault.biometricAvailable).toBe(true);
  });

  it.each(["enable", "disable"] as const)(
    "waits for a pending %s before taking a new snapshot",
    async (operation) => {
      const vault = useVaultStore();
      const mutation = deferred<void>();
      const enabledAfterMutation = operation === "enable";
      vi.mocked(api.biometricPassphraseStored).mockResolvedValue(enabledAfterMutation);
      let pendingMutation: Promise<void>;
      if (operation === "enable") {
        vi.mocked(api.storeBiometricPassphrase).mockReturnValueOnce(mutation.promise);
        pendingMutation = vault.enableBiometric("correct horse battery staple");
      } else {
        vault.biometricEnabled = true;
        vi.mocked(api.clearBiometricPassphrase).mockReturnValueOnce(mutation.promise);
        pendingMutation = vault.disableBiometric();
      }
      await flushPromises();
      const refresh = vault.refreshBiometricState();
      await flushPromises();
      expect(api.biometricAvailable).not.toHaveBeenCalled();
      expect(api.biometricPassphraseStored).not.toHaveBeenCalled();

      mutation.resolve(undefined);
      await pendingMutation;
      await refresh;
      expect(api.biometricPassphraseStored).toHaveBeenCalledTimes(1);
      expect(vault.biometricEnabled).toBe(enabledAfterMutation);
    },
  );

  it("serializes enrollment and deletion across callers", async () => {
    const vault = useVaultStore();
    const enrollment = deferred<void>();
    vi.mocked(api.storeBiometricPassphrase).mockReturnValueOnce(enrollment.promise);
    const enable = vault.enableBiometric("correct horse battery staple");
    const disable = vault.disableBiometric();
    await flushPromises();
    expect(api.storeBiometricPassphrase).toHaveBeenCalledTimes(1);
    expect(api.clearBiometricPassphrase).not.toHaveBeenCalled();

    enrollment.resolve(undefined);
    await Promise.all([enable, disable]);
    expect(api.clearBiometricPassphrase).toHaveBeenCalledTimes(1);
    expect(vault.biometricEnabled).toBe(false);
  });

  it("continues processing mutations after an enrollment failure", async () => {
    const vault = useVaultStore();
    const enrollment = deferred<void>();
    vi.mocked(api.storeBiometricPassphrase).mockReturnValueOnce(enrollment.promise);
    const enable = vault.enableBiometric("correct horse battery staple");
    const failedEnable = expect(enable).rejects.toThrow("enrollment failed");
    const disable = vault.disableBiometric();
    await flushPromises();
    expect(api.clearBiometricPassphrase).not.toHaveBeenCalled();

    enrollment.reject(new Error("enrollment failed"));
    await failedEnable;
    await disable;
    expect(api.clearBiometricPassphrase).toHaveBeenCalledTimes(1);
    expect(vault.biometricEnabled).toBe(false);
    expect(vault.error).toBeNull();
  });

  it("does not publish a previous vault's probe after successful initialization", async () => {
    const vault = useVaultStore();
    const probe = deferred<boolean>();
    vi.mocked(api.biometricPassphraseStored).mockReturnValueOnce(probe.promise);
    const refresh = vault.refreshBiometricState();
    await flushPromises();

    // Model stale frontend state; the backend remains responsible for deciding
    // whether initialization is permitted and for assigning the new binding ID.
    await vault.initialize("new vault passphrase");
    probe.resolve(true);
    await refresh;
    expect(vault.initialized).toBe(true);
    expect(vault.biometricEnabled).toBe(false);
  });

  it("dispatches initialization before a later lock can invalidate its auth generation", async () => {
    const vault = useVaultStore();
    vault.initialized = false;
    const creation = deferred<void>();
    let authGeneration = 0;
    vi.mocked(api.initializeVault).mockImplementationOnce(() => {
      const generationAtStart = authGeneration;
      return creation.promise.then(() => generationAtStart === authGeneration);
    });
    vi.mocked(api.lockVault).mockImplementationOnce(async () => {
      ++authGeneration;
    });

    const initialization = vault.initialize("new vault passphrase");
    await vault.lock();
    creation.resolve(undefined);
    await initialization;

    expect(api.initializeVault).toHaveBeenCalledTimes(1);
    expect(vault.initialized).toBe(true);
    expect(vault.unlocked).toBe(false);
  });
});
