<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from "vue";
import { useVaultStore } from "./stores/vault";
import { useHostsStore } from "./stores/hosts";
import { useKeysStore } from "./stores/keys";
import { useIdentitiesStore } from "./stores/identities";
import { useWorkspacesStore } from "./stores/workspaces";
import { useTabsStore } from "./stores/tabs";
import { useUiStore } from "./stores/ui";
import { useUpdateStore } from "./stores/update";
import AppSidebar from "./components/AppSidebar.vue";
import TerminalArea from "./components/TerminalArea.vue";
import HostList from "./components/HostList.vue";
import IdentityManager from "./components/IdentityManager.vue";
import KeyManager from "./components/KeyManager.vue";
import VaultView from "./components/VaultView.vue";
import KnownHostsView from "./components/KnownHostsView.vue";
import WorkspaceView from "./components/WorkspaceView.vue";
import SettingsView from "./components/SettingsView.vue";
import CommandPalette from "./components/CommandPalette.vue";
import UpdateModal from "./components/UpdateModal.vue";
import VaultUnlockModal from "./components/VaultUnlockModal.vue";

const vault = useVaultStore();
const hosts = useHostsStore();
const keys = useKeysStore();
const identities = useIdentitiesStore();
const workspaces = useWorkspacesStore();
const tabs = useTabsStore();
const ui = useUiStore();
const update = useUpdateStore();

const activeView = ref("hosts");
const commandPaletteOpen = ref(false);

const showMainContent = computed(() => activeView.value === "terminal");

// Show update modal automatically when an update is detected (unless dismissed)
watch(
  () => update.shouldNotify,
  (notify) => {
    if (notify) {
      update.showModal = true;
    }
  },
);

function handleKeydown(e: KeyboardEvent) {
  // Exit fullscreen on Escape
  if (e.key === "Escape" && ui.fullscreenPaneId) {
    e.preventDefault();
    ui.exitFullscreen();
    return;
  }
  const cmd = e.metaKey || e.ctrlKey;
  if (cmd && e.key === "k") {
    e.preventDefault();
    commandPaletteOpen.value = !commandPaletteOpen.value;
  } else if (cmd && e.key === "n") {
    e.preventDefault();
    activeView.value = "terminal";
    tabs.newTab();
  } else if (cmd && e.key === "u") {
    e.preventDefault();
    if (update.available) {
      update.showUpdateDialog();
    }
  } else if (cmd && e.key === ",") {
    e.preventDefault();
    activeView.value = "settings";
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

  // Check for updates on startup (non-blocking)
  await update.registerListeners();
  await update.check();
  if (update.shouldNotify) {
    update.showModal = true;
  }
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);
  update.unregisterListeners();
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
    />

    <main class="flex-1 flex flex-col overflow-hidden min-w-[480px]">
      <!-- TerminalArea uses v-show to preserve state across view switches -->
      <TerminalArea v-show="showMainContent" />
      <HostList v-if="activeView === 'hosts'" @switch-view="activeView = $event" />
      <IdentityManager v-else-if="activeView === 'identities'" />
      <KeyManager v-else-if="activeView === 'keys'" />
      <VaultView v-else-if="activeView === 'vault'" />
      <KnownHostsView v-else-if="activeView === 'known-hosts'" />
      <WorkspaceView v-else-if="activeView === 'workspaces'" />
      <SettingsView v-else-if="activeView === 'settings'" />
    </main>
    <CommandPalette
      :open="commandPaletteOpen"
      @close="commandPaletteOpen = false"
      @navigate="activeView = $event; commandPaletteOpen = false"
    />
    <UpdateModal />
    <VaultUnlockModal />
  </div>
</template>
