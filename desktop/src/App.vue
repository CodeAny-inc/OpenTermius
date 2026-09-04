<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";
import { useVaultStore } from "./stores/vault";
import { useHostsStore } from "./stores/hosts";
import { useKeysStore } from "./stores/keys";
import { useIdentitiesStore } from "./stores/identities";
import { useWorkspacesStore } from "./stores/workspaces";
import { useTabsStore } from "./stores/tabs";
import AppSidebar from "./components/AppSidebar.vue";
import TerminalArea from "./components/TerminalArea.vue";
import HostList from "./components/HostList.vue";
import IdentityManager from "./components/IdentityManager.vue";
import KeyManager from "./components/KeyManager.vue";
import VaultView from "./components/VaultView.vue";
import KnownHostsView from "./components/KnownHostsView.vue";
import WorkspaceView from "./components/WorkspaceView.vue";
import CommandPalette from "./components/CommandPalette.vue";
import UpdateBanner from "./components/UpdateBanner.vue";
import * as api from "./api";
import type { UpdateInfo } from "./api";

const vault = useVaultStore();
const hosts = useHostsStore();
const keys = useKeysStore();
const identities = useIdentitiesStore();
const workspaces = useWorkspacesStore();
const tabs = useTabsStore();

const activeView = ref("hosts");
const commandPaletteOpen = ref(false);
const manualUpdateInfo = ref<api.UpdateInfo | null>(null);

const showMainContent = computed(() => activeView.value === "terminal");

function onSidebarUpdate(info: api.UpdateInfo) {
  manualUpdateInfo.value = info;
}

function handleKeydown(e: KeyboardEvent) {
  const cmd = e.metaKey || e.ctrlKey;
  if (cmd && e.key === "k") {
    e.preventDefault();
    commandPaletteOpen.value = !commandPaletteOpen.value;
  } else if (cmd && e.key === "n") {
    e.preventDefault();
    activeView.value = "terminal";
    tabs.newTab();
  } else if (cmd && e.key === ",") {
    e.preventDefault();
    activeView.value = "vault";
  } else if (e.key === "Escape" && commandPaletteOpen.value) {
    commandPaletteOpen.value = false;
  }
}

onMounted(async () => {
  window.addEventListener("keydown", handleKeydown);
  await vault.checkStatus();
  await hosts.load();
  await keys.load();
  await identities.load();
  await workspaces.load();
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);
});
</script>

<template>
  <div class="flex h-screen w-screen overflow-hidden bg-background text-foreground">
    <AppSidebar
      :active-view="activeView"
      :vault-unlocked="vault.unlocked"
      :tab-count="tabs.tabs.length"
      @navigate="activeView = $event"
      @open-command-palette="commandPaletteOpen = true"
      @update-available="onSidebarUpdate"
    />

    <main class="flex-1 flex flex-col overflow-hidden min-w-[480px]">
      <TerminalArea v-if="showMainContent" />
      <HostList v-else-if="activeView === 'hosts'" />
      <IdentityManager v-else-if="activeView === 'identities'" />
      <KeyManager v-else-if="activeView === 'keys'" />
      <VaultView v-else-if="activeView === 'vault'" />
      <KnownHostsView v-else-if="activeView === 'known-hosts'" />
      <WorkspaceView v-else-if="activeView === 'workspaces'" />
    </main>
    <CommandPalette
      :open="commandPaletteOpen"
      @close="commandPaletteOpen = false"
      @navigate="activeView = $event; commandPaletteOpen = false"
    />
    <UpdateBanner :manual-update-info="manualUpdateInfo" />
  </div>
</template>
