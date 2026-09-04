<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useWorkspacesStore } from "../stores/workspaces";
import { useHostsStore } from "../stores/hosts";
import { useTabsStore } from "../stores/tabs";
import type { Workspace, TabLayout, PaneLayout } from "../types";

const workspaces = useWorkspacesStore();
const hosts = useHostsStore();
const tabs = useTabsStore();

const showCreate = ref(false);
const newName = ref("");

onMounted(() => {
  workspaces.load();
});

async function createWorkspace() {
  if (!newName.value.trim()) return;
  await workspaces.createWorkspace(newName.value);
  newName.value = "";
  showCreate.value = false;
}

async function deleteWorkspace(id: string) {
  if (confirm("Delete this workspace?")) {
    await workspaces.deleteWorkspace(id);
  }
}

async function restoreWorkspace(ws: Workspace) {
  // Close all current tabs
  for (const tab of [...tabs.tabs]) {
    tabs.closeTab(tab.id);
  }
  // Restore tabs from workspace
  for (const tabLayout of ws.tabs) {
    restoreTab(tabLayout);
  }
}

function restoreTab(tabLayout: TabLayout) {
  // Create a new tab and restore its tree
  const tab = tabs.newTab();
  tab.title = tabLayout.title;
  // For now, just create panes for each leaf in the layout
  // A full implementation would reconstruct the split tree
  restorePane(tabLayout.layout, tab.id);
}

function restorePane(layout: PaneLayout, tabId: string) {
  if (layout.type === "pane") {
    const host = layout.host_id
      ? hosts.hosts.find((h) => h.id === layout.host_id)
      : undefined;
    if (host) {
      // The tab was already created with a local terminal pane
      // We need to replace it with an SSH pane
      // For simplicity, just open a new tab for each pane
      tabs.newTab(host);
    }
  } else {
    restorePane(layout.first, tabId);
    restorePane(layout.second, tabId);
  }
}

async function saveCurrentLayout() {
  // Save current tabs as a workspace
  if (!workspaces.workspaces.length) {
    showCreate.value = true;
    return;
  }
  // Save to the active workspace or create a new one
  const name = prompt("Workspace name:", `Workspace ${workspaces.workspaces.length + 1}`);
  if (!name) return;
  const ws = await workspaces.createWorkspace(name);

  // Convert current tabs to TabLayout
  ws.tabs = tabs.tabs.map((tab) => ({
    id: tab.id,
    title: tab.title,
    layout: convertTree(tab.tree),
  }));

  await workspaces.saveWorkspace(ws);
}

function convertTree(node: typeof tabs.tabs[0]["tree"]): PaneLayout {
  if ("direction" in node) {
    return {
      type: "split",
      direction: node.direction,
      ratio: node.ratio,
      first: convertTree(node.first),
      second: convertTree(node.second),
    };
  }
  return {
    type: "pane",
    host_id: node.hostId,
    terminal_type: node.terminalType,
  };
}
</script>

<template>
  <div style="padding: 16px; overflow-y: auto; height: 100%;">
    <div style="display: flex; align-items: center; margin-bottom: 16px; gap: 12px;">
      <h2 style="font-size: 16px;">Workspaces</h2>
      <div style="margin-left: auto; display: flex; gap: 8px;">
        <button class="btn btn-sm" @click="saveCurrentLayout">Save Current Layout</button>
        <button class="btn btn-sm secondary" @click="showCreate = !showCreate">+ New</button>
      </div>
    </div>

    <div v-if="showCreate" style="margin-bottom: 16px; display: flex; gap: 8px;">
      <input v-model="newName" class="search-input" placeholder="workspace name" @keydown.enter="createWorkspace" />
      <button class="btn btn-sm" @click="createWorkspace">Create</button>
    </div>

    <div v-if="workspaces.workspaces.length">
      <div
        v-for="ws in workspaces.workspaces"
        :key="ws.id"
        class="key-item"
      >
        <div class="key-info">
          <div class="key-label">{{ ws.name }}</div>
          <div class="key-meta">{{ ws.tabs.length }} tab(s)</div>
        </div>
        <button class="btn btn-sm" @click="restoreWorkspace(ws)">Restore</button>
        <button class="icon-btn" @click="deleteWorkspace(ws.id)">🗑</button>
      </div>
    </div>
    <div v-else class="empty-state" style="padding: 40px;">
      <p>No workspaces yet</p>
      <p style="font-size: 11px;">Save your current terminal layout as a workspace</p>
    </div>
  </div>
</template>
