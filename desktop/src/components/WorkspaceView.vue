<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useWorkspacesStore } from "../stores/workspaces";
import { useTabsStore, isPane, isSplit, type PaneTree } from "../stores/tabs";
import { useHostsStore } from "../stores/hosts";
import Button from "./ui/Button.vue";
import Input from "./ui/Input.vue";
import Dialog from "./ui/Dialog.vue";
import FormGroup from "./ui/FormGroup.vue";
import Label from "./ui/Label.vue";
import {
  FolderOpen,
  Plus,
  Trash2,
  Pencil,
  Folder,
  Save,
  Play,
  LayoutGrid,
} from "lucide-vue-next";
import type { Workspace, TabLayout, PaneLayout, Host } from "../types";

const workspaces = useWorkspacesStore();
const tabs = useTabsStore();
const hosts = useHostsStore();

const showForm = ref(false);
const editing = ref<Workspace | null>(null);
const name = ref("");

onMounted(() => {
  workspaces.load();
});

function addWorkspace() {
  editing.value = null;
  name.value = "";
  showForm.value = true;
}

function editWorkspace(ws: Workspace) {
  editing.value = ws;
  name.value = ws.name;
  showForm.value = true;
}

async function save() {
  if (!name.value.trim()) return;
  if (editing.value) {
    await workspaces.saveWorkspace({ ...editing.value, name: name.value });
  } else {
    await workspaces.createWorkspace(name.value);
  }
  showForm.value = false;
}

async function remove(id: string, name: string) {
  if (confirm(`Delete workspace "${name}"?`)) {
    await workspaces.deleteWorkspace(id);
  }
}

// --- Save current tab layout into a workspace ---
function paneTreeToLayout(node: PaneTree): PaneLayout {
  if (isPane(node)) {
    return {
      type: "pane",
      host_id: node.hostId ?? null,
      terminal_type: node.terminalType,
    };
  }
  return {
    type: "split",
    direction: node.direction,
    ratio: node.ratio,
    first: paneTreeToLayout(node.first),
    second: paneTreeToLayout(node.second),
  };
}

async function saveCurrentLayout(ws: Workspace) {
  // Convert current tabs to TabLayouts
  const tabLayouts: TabLayout[] = tabs.tabs.map((tab) => ({
    id: tab.id,
    title: tab.title,
    layout: paneTreeToLayout(tab.tree),
  }));
  await workspaces.saveWorkspace({ ...ws, tabs: tabLayouts });
}

// --- Restore a workspace layout into tabs ---
function layoutToPaneTree(layout: PaneLayout): PaneTree {
  if (layout.type === "pane") {
    const host: Host | undefined = layout.host_id
      ? hosts.hosts.find((h) => h.id === layout.host_id)
      : undefined;
    return {
      id: crypto.randomUUID(),
      sessionId: null,
      hostId: layout.host_id ?? null,
      terminalType: (layout.terminal_type as "ssh" | "local") ?? (host ? "ssh" : "local"),
      title: host?.label ?? "Local Terminal",
      connected: false,
      closing: false,
    };
  }
  return {
    id: crypto.randomUUID(),
    direction: layout.direction,
    ratio: layout.ratio,
    first: layoutToPaneTree(layout.first),
    second: layoutToPaneTree(layout.second),
  };
}

function restoreWorkspace(ws: Workspace) {
  if (!ws.tabs || ws.tabs.length === 0) {
    // Just create a fresh local terminal
    tabs.newTab();
    return;
  }

  // Close all existing tabs first
  const existingIds = tabs.tabs.map((t) => t.id);
  existingIds.forEach((id) => tabs.closeTab(id));

  // Recreate tabs from the workspace
  for (const tabLayout of ws.tabs) {
    const tree = layoutToPaneTree(tabLayout.layout);
    tabs.tabs.push({
      id: tabLayout.id,
      title: tabLayout.title,
      tree,
    });
  }
  if (tabs.tabs.length > 0) {
    tabs.setActiveTab(tabs.tabs[0].id);
  }
}
</script>

<template>
  <div class="flex flex-col h-full overflow-hidden">
    <!-- Header -->
    <div class="flex h-11 items-center gap-2 border-b border-border px-4">
      <h2 class="text-[14px] font-semibold">Workspaces</h2>
      <div class="ml-auto">
        <Button size="sm" @click="addWorkspace">
          <Plus class="size-3.5" :stroke-width="1.75" />
          New Workspace
        </Button>
      </div>
    </div>

    <!-- Info banner -->
    <div class="px-4 py-2 border-b border-border bg-muted/30">
      <p class="text-[11px] text-muted-foreground flex items-center gap-1.5">
        <LayoutGrid class="size-3" :stroke-width="1.75" />
        Save your current terminal tab layout into a workspace, or restore a saved layout.
      </p>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-3">
      <div v-if="workspaces.workspaces.length" class="flex flex-col gap-1.5">
        <div
          v-for="ws in workspaces.workspaces"
          :key="ws.id"
          class="group flex items-center gap-3 rounded-md border border-border bg-card p-3 transition-colors duration-100 hover:border-muted-foreground/30"
        >
          <div class="flex h-9 w-9 items-center justify-center rounded-md bg-muted">
            <FolderOpen class="size-4 text-muted-foreground" :stroke-width="1.75" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-[13px] font-medium truncate">{{ ws.name }}</div>
            <div class="text-[11px] text-muted-foreground truncate mt-0.5">
              {{ ws.tabs?.length || 0 }} saved tab layout{{ (ws.tabs?.length || 0) === 1 ? '' : 's' }}
            </div>
          </div>
          <div class="flex items-center gap-1">
            <!-- Save current layout -->
            <button
              class="flex h-7 items-center gap-1 rounded px-2 text-[11px] text-muted-foreground hover:bg-accent hover:text-foreground transition-colors duration-100"
              aria-label="Save current layout"
              title="Save current tab layout into this workspace"
              @click="saveCurrentLayout(ws)"
            >
              <Save class="size-3" :stroke-width="1.75" />
              Save
            </button>
            <!-- Restore layout -->
            <button
              class="flex h-7 items-center gap-1 rounded px-2 text-[11px] text-muted-foreground hover:bg-accent hover:text-foreground transition-colors duration-100"
              aria-label="Restore layout"
              title="Restore this workspace's tab layout"
              @click="restoreWorkspace(ws)"
            >
              <Play class="size-3" :stroke-width="1.75" />
              Restore
            </button>
            <!-- Edit -->
            <button
              class="flex h-7 w-7 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground transition-colors duration-100 opacity-0 group-hover:opacity-100 transition-opacity"
              aria-label="Edit workspace"
              @click="editWorkspace(ws)"
            >
              <Pencil class="size-3.5" :stroke-width="1.75" />
            </button>
            <!-- Delete -->
            <button
              class="flex h-7 w-7 items-center justify-center rounded text-muted-foreground hover:bg-destructive/20 hover:text-destructive transition-colors duration-100 opacity-0 group-hover:opacity-100 transition-opacity"
              aria-label="Delete workspace"
              @click="remove(ws.id, ws.name)"
            >
              <Trash2 class="size-3.5" :stroke-width="1.75" />
            </button>
          </div>
        </div>
      </div>

      <!-- Empty state -->
      <div v-else class="flex flex-col items-center justify-center py-16 px-6 gap-3 text-center">
        <Folder class="size-8 text-muted-foreground/50" :stroke-width="1.5" />
        <div>
          <p class="text-[14px] font-medium text-foreground">No workspaces yet</p>
          <p class="text-[12px] text-muted-foreground mt-1">
            Create a workspace and save your terminal layouts for different projects
          </p>
        </div>
        <Button size="sm" @click="addWorkspace">
          <Plus class="size-3.5" :stroke-width="1.75" />
          New Workspace
        </Button>
      </div>
    </div>

    <Dialog
      v-if="showForm"
      :open="true"
      :title="editing ? 'Edit Workspace' : 'New Workspace'"
      width="440px"
      @close="showForm = false"
    >
      <div class="flex flex-col gap-4">
        <FormGroup>
          <Label for="ws-name">Name</Label>
          <Input id="ws-name" v-model="name" placeholder="Production, Staging..." @keydown.enter="save" />
        </FormGroup>
      </div>
      <template #footer>
        <Button variant="ghost" @click="showForm = false">Cancel</Button>
        <Button :disabled="!name.trim()" @click="save">
          {{ editing ? "Save changes" : "Create workspace" }}
        </Button>
      </template>
    </Dialog>
  </div>
</template>
