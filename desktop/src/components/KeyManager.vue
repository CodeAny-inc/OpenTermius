<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useKeysStore } from "../stores/keys";
import { useVaultStore } from "../stores/vault";

const keys = useKeysStore();
const vault = useVaultStore();

const showImport = ref(false);
const importLabel = ref("");
const importKey = ref("");
const importPassphrase = ref("");
const genLabel = ref("");

onMounted(() => {
  keys.load();
});

async function generate() {
  if (!genLabel.value.trim()) {
    genLabel.value = `key-${Date.now()}`;
  }
  try {
    await keys.generateKey(genLabel.value);
    genLabel.value = "";
  } catch (e) {
    alert(`Failed to generate key: ${e}`);
  }
}

async function doImport() {
  if (!importKey.value.trim() || !importLabel.value.trim()) return;
  try {
    await keys.importKey(
      importLabel.value,
      importKey.value,
      importPassphrase.value || null,
    );
    showImport.value = false;
    importLabel.value = "";
    importKey.value = "";
    importPassphrase.value = "";
  } catch (e) {
    alert(`Failed to import key: ${e}`);
  }
}

async function deleteKey(keyId: string, label: string) {
  if (confirm(`Delete key "${label}"? This cannot be undone.`)) {
    try {
      await keys.deleteKey(keyId);
    } catch (e) {
      alert(`Failed to delete key: ${e}`);
    }
  }
}
</script>

<template>
  <div style="padding: 16px; overflow-y: auto; height: 100%;">
    <div style="display: flex; align-items: center; margin-bottom: 16px; gap: 12px;">
      <h2 style="font-size: 16px;">SSH Keys</h2>
      <div style="margin-left: auto; display: flex; gap: 8px;">
        <input v-model="genLabel" placeholder="key label" class="search-input" style="width: 120px;" />
        <button class="btn btn-sm" @click="generate">Generate Ed25519</button>
        <button class="btn btn-sm secondary" @click="showImport = !showImport">Import</button>
      </div>
    </div>

    <div v-if="!vault.unlocked" class="empty-state" style="padding: 40px;">
      <p>Unlock the vault to manage keys</p>
    </div>

    <div v-else-if="showImport" class="modal-overlay" @click.self="showImport = false">
      <div class="modal">
        <h2>Import Key</h2>
        <div class="form-group">
          <label>Label</label>
          <input v-model="importLabel" placeholder="my-key" />
        </div>
        <div class="form-group">
          <label>OpenSSH Private Key</label>
          <textarea v-model="importKey" rows="10" placeholder="-----BEGIN OPENSSH PRIVATE KEY-----&#10;..." style="font-family: monospace; font-size: 11px;"></textarea>
        </div>
        <div class="form-group">
          <label>Key Passphrase (if encrypted)</label>
          <input v-model="importPassphrase" type="password" placeholder="optional" />
        </div>
        <div class="modal-actions">
          <button class="btn secondary" @click="showImport = false">Cancel</button>
          <button class="btn" @click="doImport">Import</button>
        </div>
      </div>
    </div>

    <div v-else-if="keys.keys.length">
      <div v-for="key in keys.keys" :key="key.id" class="key-item">
        <span class="key-type-badge">{{ key.key_type }}</span>
        <div class="key-info">
          <div class="key-label">{{ key.label }}</div>
          <div class="key-meta">{{ key.fingerprint.slice(0, 48) }}...</div>
        </div>
        <button class="icon-btn" @click="deleteKey(key.id, key.label)" title="Delete">🗑</button>
      </div>
    </div>

    <div v-else class="empty-state" style="padding: 40px;">
      <p>No keys yet</p>
      <p style="font-size: 11px;">Generate or import a key to get started</p>
    </div>
  </div>
</template>
