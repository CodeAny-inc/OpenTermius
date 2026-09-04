<script setup lang="ts">
import { ref } from "vue";
import { useVaultStore } from "../stores/vault";

const vault = useVaultStore();

const passphrase = ref("");
const confirmPassphrase = ref("");

async function initialize() {
  if (passphrase.value.length < 8) {
    alert("Passphrase must be at least 8 characters");
    return;
  }
  if (passphrase.value !== confirmPassphrase.value) {
    alert("Passphrases do not match");
    return;
  }
  await vault.initialize(passphrase.value);
  passphrase.value = "";
  confirmPassphrase.value = "";
}

async function unlock() {
  try {
    await vault.unlock(passphrase.value);
    passphrase.value = "";
  } catch {
    // error is set in store
  }
}

async function lock() {
  await vault.lock();
}
</script>

<template>
  <div class="vault-view">
    <h2 style="font-size: 16px; margin-bottom: 16px;">Vault</h2>

    <!-- Not initialized: show setup -->
    <div v-if="vault.needsSetup" class="vault-card">
      <p>
        The vault stores your private SSH keys encrypted at rest using
        Argon2id key derivation and AES-256-GCM encryption. Choose a strong
        passphrase — it cannot be recovered if lost.
      </p>
      <div class="form-group">
        <label>Passphrase (min 8 characters)</label>
        <input v-model="passphrase" type="password" />
      </div>
      <div class="form-group">
        <label>Confirm passphrase</label>
        <input v-model="confirmPassphrase" type="password" @keydown.enter="initialize" />
      </div>
      <div v-if="vault.error" style="color: var(--danger); font-size: 12px; margin-bottom: 8px;">
        {{ vault.error }}
      </div>
      <button class="btn" @click="initialize">Initialize Vault</button>
    </div>

    <!-- Initialized but locked: show unlock -->
    <div v-else-if="vault.needsUnlock" class="vault-card">
      <p>Enter your vault passphrase to unlock and access your SSH keys.</p>
      <div class="form-group">
        <label>Passphrase</label>
        <input v-model="passphrase" type="password" @keydown.enter="unlock" />
      </div>
      <div v-if="vault.error" style="color: var(--danger); font-size: 12px; margin-bottom: 8px;">
        {{ vault.error }}
      </div>
      <button class="btn" @click="unlock">Unlock</button>
    </div>

    <!-- Unlocked: show status -->
    <div v-else class="vault-card">
      <p style="color: var(--ok);">Vault is unlocked. Private keys are available in memory.</p>
      <p>
        The passphrase is held in memory only while unlocked. Locking the
        vault will zeroize the passphrase and prevent key usage until unlocked again.
      </p>
      <button class="btn secondary" @click="lock">Lock Vault</button>
    </div>
  </div>
</template>
