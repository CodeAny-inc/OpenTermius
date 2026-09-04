<script setup lang="ts">
import { ref, onMounted } from "vue";
import * as api from "../api";
import type { KnownHostEntry } from "../types";

const knownHosts = ref<KnownHostEntry[]>([]);

onMounted(async () => {
  await load();
});

async function load() {
  knownHosts.value = await api.listKnownHosts();
}

async function remove(host: string, port: number) {
  const display = `${host}:${port}`;
  if (confirm(`Remove known host entry for ${display}?`)) {
    await api.removeKnownHost(host, port);
    await load();
  }
}
</script>

<template>
  <div style="padding: 16px; overflow-y: auto; height: 100%;">
    <h2 style="font-size: 16px; margin-bottom: 16px;">Known Hosts</h2>
    <p style="font-size: 12px; color: var(--muted); margin-bottom: 16px;">
      Host keys are recorded on first connection (trust-on-first-use).
      If a host's key changes, the connection will be rejected. Remove an
      entry here to re-establish trust.
    </p>

    <div v-if="knownHosts.length">
      <div
        v-for="(entry, i) in knownHosts"
        :key="i"
        class="key-item"
      >
        <div class="key-info">
          <div class="key-label">{{ entry.host }}</div>
          <div class="key-meta">{{ entry.key_type }} · {{ entry.fingerprint.slice(0, 48) }}...</div>
        </div>
        <button
          class="icon-btn"
          @click="remove(entry.host, parseInt(entry.host.split(':')[1] || '22'))"
          title="Remove"
        >🗑</button>
      </div>
    </div>
    <div v-else class="empty-state" style="padding: 40px;">
      <p>No known hosts recorded yet</p>
    </div>
  </div>
</template>
