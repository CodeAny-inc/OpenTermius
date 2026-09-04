<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useHostsStore } from "../stores/hosts";
import { useTabsStore } from "../stores/tabs";
import { useVaultStore } from "../stores/vault";
import HostForm from "./HostForm.vue";
import type { Host } from "../types";

const hosts = useHostsStore();
const tabs = useTabsStore();
const vault = useVaultStore();

const showForm = ref(false);
const editingHost = ref<Host | null>(null);

onMounted(() => {
  hosts.load();
});

function connectToHost(host: Host) {
  tabs.newTab(host);
}

function editHost(host: Host) {
  editingHost.value = host;
  showForm.value = true;
}

function addHost() {
  editingHost.value = null;
  showForm.value = true;
}

async function deleteHost(host: Host) {
  if (confirm(`Delete host "${host.label}"?`)) {
    await hosts.deleteHost(host.id);
  }
}

function selectGroup(groupId: string | null) {
  hosts.selectedGroupId = groupId;
}
</script>

<template>
  <div style="padding: 16px; overflow-y: auto; height: 100%;">
    <div style="display: flex; align-items: center; margin-bottom: 16px; gap: 12px;">
      <h2 style="font-size: 16px;">Hosts</h2>
      <button class="btn btn-sm" style="margin-left: auto;" @click="addHost">+ Add Host</button>
    </div>

    <div class="search-box" style="padding: 0 0 12px 0;">
      <input
        v-model="hosts.searchQuery"
        class="search-input"
        placeholder="Search hosts..."
      />
    </div>

    <!-- Groups -->
    <div v-if="hosts.groups.length" style="margin-bottom: 12px;">
      <div
        class="group-item"
        :class="{ active: hosts.selectedGroupId === null }"
        @click="selectGroup(null)"
      >
        <span>All hosts</span>
      </div>
      <div
        v-for="group in hosts.groups"
        :key="group.id"
        class="group-item"
        :class="{ active: hosts.selectedGroupId === group.id }"
        @click="selectGroup(group.id)"
      >
        <div class="group-color" :style="{ background: group.color || 'var(--accent)' }"></div>
        <span>{{ group.name }}</span>
      </div>
    </div>

    <!-- Host list -->
    <ul class="host-list" v-if="hosts.filteredHosts.length">
      <li
        v-for="host in hosts.filteredHosts"
        :key="host.id"
        class="host-item"
        @click="connectToHost(host)"
      >
        <div class="host-info">
          <div class="host-label">{{ host.label }}</div>
          <div class="host-detail">{{ host.username }}@{{ host.hostname }}:{{ host.port }}</div>
        </div>
        <div class="host-actions">
          <button class="icon-btn" @click.stop="editHost(host)" title="Edit">✎</button>
          <button class="icon-btn" @click.stop="deleteHost(host)" title="Delete">🗑</button>
        </div>
      </li>
    </ul>
    <div v-else class="empty-state" style="padding: 40px;">
      <p>No hosts yet</p>
      <button class="btn" @click="addHost">Add your first host</button>
    </div>

    <HostForm
      v-if="showForm"
      :host="editingHost"
      @close="showForm = false"
    />
  </div>
</template>
