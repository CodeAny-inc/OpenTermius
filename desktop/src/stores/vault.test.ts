import { describe, it, expect, beforeEach, vi } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useVaultStore } from "./vault";

vi.mock("../api", () => ({
  vaultIsInitialized: vi.fn(() => Promise.resolve(false)),
  isVaultUnlocked: vi.fn(() => Promise.resolve(false)),
  initializeVault: vi.fn(() => Promise.resolve()),
  unlockVault: vi.fn(() => Promise.resolve()),
  lockVault: vi.fn(() => Promise.resolve()),
  biometricAvailable: vi.fn(() => Promise.resolve(false)),
  biometricPassphraseStored: vi.fn(() => Promise.resolve(false)),
  storeBiometricPassphrase: vi.fn(() => Promise.resolve()),
  unlockWithBiometric: vi.fn(() => Promise.resolve(true)),
  clearBiometricPassphrase: vi.fn(() => Promise.resolve()),
}));

import * as api from "../api";

describe("vault store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
    vi.mocked(api.biometricAvailable).mockResolvedValue(false);
    vi.mocked(api.biometricPassphraseStored).mockResolvedValue(false);
  });

  describe("initial state", () => {
    it("starts uninitialized and locked", () => {
      const store = useVaultStore();
      expect(store.initialized).toBe(false);
      expect(store.unlocked).toBe(false);
      expect(store.error).toBeNull();
      expect(store.needsSetup).toBe(true);
      expect(store.needsUnlock).toBe(false);
    });
  });

  describe("checkStatus", () => {
    it("checks vault initialization and unlock status", async () => {
      const store = useVaultStore();
      vi.mocked(api.vaultIsInitialized).mockResolvedValue(true);
      vi.mocked(api.isVaultUnlocked).mockResolvedValue(false);

      await store.checkStatus();

      expect(store.initialized).toBe(true);
      expect(store.unlocked).toBe(false);
      expect(store.needsSetup).toBe(false);
      expect(store.needsUnlock).toBe(true);
    });

    it("uses the protected Keychain item as biometric source of truth", async () => {
      const store = useVaultStore();
      vi.mocked(api.biometricAvailable).mockResolvedValue(true);
      vi.mocked(api.biometricPassphraseStored).mockResolvedValue(true);

      await store.checkStatus();

      expect(api.biometricPassphraseStored).toHaveBeenCalledTimes(1);
      expect(store.biometricAvailable).toBe(true);
      expect(store.biometricEnabled).toBe(true);
      expect(store.error).toBeNull();
    });

    it("fails closed when the protected Keychain state cannot be queried", async () => {
      const store = useVaultStore();
      vi.mocked(api.biometricAvailable).mockResolvedValue(true);
      vi.mocked(api.biometricPassphraseStored).mockRejectedValue(
        new Error("Keychain unavailable"),
      );

      await store.checkStatus();

      expect(store.biometricAvailable).toBe(true);
      expect(store.biometricEnabled).toBe(false);
      expect(store.error).toBe("Error: Keychain unavailable");
    });
  });

  describe("initialize", () => {
    it("initializes and unlocks the vault", async () => {
      const store = useVaultStore();

      await store.initialize("my-passphrase");

      expect(api.initializeVault).toHaveBeenCalledWith("my-passphrase");
      expect(store.initialized).toBe(true);
      expect(store.unlocked).toBe(true);
      expect(store.error).toBeNull();
    });

    it("sets error on failure", async () => {
      const store = useVaultStore();
      vi.mocked(api.initializeVault).mockRejectedValue(new Error("Weak passphrase"));

      await store.initialize("weak");

      expect(store.initialized).toBe(false);
      expect(store.error).toBe("Error: Weak passphrase");
    });
  });

  describe("unlock", () => {
    it("unlocks the vault with a passphrase", async () => {
      const store = useVaultStore();
      store.initialized = true;

      await store.unlock("correct-passphrase");

      expect(api.unlockVault).toHaveBeenCalledWith("correct-passphrase");
      expect(store.unlocked).toBe(true);
      expect(store.error).toBeNull();
    });

    it("sets error and rethrows on wrong passphrase", async () => {
      const store = useVaultStore();
      store.initialized = true;
      vi.mocked(api.unlockVault).mockRejectedValue(new Error("Wrong passphrase"));

      await expect(store.unlock("wrong")).rejects.toThrow("Wrong passphrase");
      expect(store.unlocked).toBe(false);
      expect(store.error).toBe("Error: Wrong passphrase");
    });
  });

  describe("lock", () => {
    it("locks the vault", async () => {
      const store = useVaultStore();
      store.unlocked = true;

      await store.lock();

      expect(api.lockVault).toHaveBeenCalled();
      expect(store.unlocked).toBe(false);
    });
  });

  describe("computed flags", () => {
    it("needsSetup is true when not initialized", () => {
      const store = useVaultStore();
      store.initialized = false;
      expect(store.needsSetup).toBe(true);
    });

    it("needsUnlock is true when initialized but not unlocked", () => {
      const store = useVaultStore();
      store.initialized = true;
      store.unlocked = false;
      expect(store.needsSetup).toBe(false);
      expect(store.needsUnlock).toBe(true);
    });

    it("needsSetup and needsUnlock are false when initialized and unlocked", () => {
      const store = useVaultStore();
      store.initialized = true;
      store.unlocked = true;
      expect(store.needsSetup).toBe(false);
      expect(store.needsUnlock).toBe(false);
    });
  });
});
