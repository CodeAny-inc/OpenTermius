<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useVaultStore } from "./stores/vault";
import { useHostsStore } from "./stores/hosts";
import { useKeysStore } from "./stores/keys";
import { useWorkspacesStore } from "./stores/workspaces";
import { useTabsStore } from "./stores/tabs";
import Sidebar from "./components/Sidebar.vue";
import TerminalArea from "./components/TerminalArea.vue";
import HostList from "./components/HostList.vue";
import KeyManager from "./components/KeyManager.vue";
import VaultView from "./components/VaultView.vue";
import KnownHostsView from "./components/KnownHostsView.vue";
import WorkspaceView from "./components/WorkspaceView.vue";

const vault = useVaultStore();
const hosts = useHostsStore();
const keys = useKeysStore();
const workspaces = useWorkspacesStore();
const tabs = useTabsStore();

const activeView = ref("hosts");

const showMainContent = computed(() => activeView.value === "terminal");

onMounted(async () => {
  await vault.checkStatus();
  await hosts.load();
  await keys.load();
  await workspaces.load();
});
</script>

<template>
  <Sidebar
    :active-view="activeView"
    @navigate="activeView = $event"
  />
  <div class="main-area">
    <TerminalArea v-if="showMainContent" />
    <HostList v-else-if="activeView === 'hosts'" />
    <KeyManager v-else-if="activeView === 'keys'" />
    <VaultView v-else-if="activeView === 'vault'" />
    <KnownHostsView v-else-if="activeView === 'known-hosts'" />
    <WorkspaceView v-else-if="activeView === 'workspaces'" />
  </div>
</template>
