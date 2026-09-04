<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useHostsStore } from "../stores/hosts";
import { useTabsStore } from "../stores/tabs";
import HostForm from "./HostForm.vue";
import Button from "./ui/Button.vue";
import Input from "./ui/Input.vue";
import {
  Plus,
  Search,
  Server,
  Pencil,
  Trash2,
  Folder,
  ChevronRight,
} from "lucide-vue-next";
import { cn } from "../lib/cn";
import type { Host } from "../types";

const hosts = useHostsStore();
const tabs = useTabsStore();

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
  <div class="flex flex-col h-full overflow-hidden">
    <!-- Header -->
    <div class="flex h-11 items-center gap-2 border-b border-border px-4">
      <h2 class="text-[14px] font-semibold">Hosts</h2>
      <div class="ml-auto flex items-center gap-2">
        <Button size="sm" @click="addHost">
          <Plus class="size-3.5" :stroke-width="1.75" />
          Add Host
        </Button>
      </div>
    </div>

    <!-- Search -->
    <div class="px-3 py-2 border-b border-border">
      <div class="relative">
        <Search class="absolute left-2.5 top-1/2 -translate-y-1/2 size-3.5 text-muted-foreground" :stroke-width="1.75" />
        <Input
          v-model="hosts.searchQuery"
          placeholder="Search hosts..."
          class="pl-8"
        />
      </div>
    </div>

    <!-- Content area -->
    <div class="flex-1 overflow-y-auto">
      <!-- Groups -->
      <div v-if="hosts.groups.length" class="px-2 pt-2 pb-1">
        <button
          class="flex h-8 w-full items-center gap-2 rounded-md px-2 text-[13px] transition-colors duration-100"
          :class="hosts.selectedGroupId === null ? 'bg-sidebar-accent text-sidebar-accent-foreground' : 'text-muted-foreground hover:bg-muted'"
          @click="selectGroup(null)"
        >
          <Server class="size-3.5" :stroke-width="1.75" />
          <span>All hosts</span>
        </button>
        <button
          v-for="group in hosts.groups"
          :key="group.id"
          class="flex h-8 w-full items-center gap-2 rounded-md px-2 text-[13px] transition-colors duration-100"
          :class="hosts.selectedGroupId === group.id ? 'bg-sidebar-accent text-sidebar-accent-foreground' : 'text-muted-foreground hover:bg-muted'"
          @click="selectGroup(group.id)"
        >
          <Folder class="size-3.5" :stroke-width="1.75" />
          <span class="truncate">{{ group.name }}</span>
        </button>
      </div>

      <!-- Host list -->
      <div v-if="hosts.filteredHosts.length" class="px-2 pt-1">
        <div
          v-for="host in hosts.filteredHosts"
          :key="host.id"
          class="group flex h-10 items-center gap-2.5 rounded-md px-2 cursor-pointer transition-colors duration-100 hover:bg-muted"
          @click="connectToHost(host)"
        >
          <Server class="size-4 text-muted-foreground shrink-0" :stroke-width="1.75" />
          <div class="flex-1 min-w-0">
            <div class="text-[13px] font-medium truncate">{{ host.label }}</div>
            <div class="text-[11px] text-muted-foreground truncate font-mono">
              {{ host.username }}@{{ host.hostname }}:{{ host.port }}
            </div>
          </div>
          <div class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
            <button
              class="flex h-6 w-6 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground transition-colors duration-100"
              aria-label="Edit host"
              @click.stop="editHost(host)"
            >
              <Pencil class="size-3" :stroke-width="1.75" />
            </button>
            <button
              class="flex h-6 w-6 items-center justify-center rounded text-muted-foreground hover:bg-destructive/20 hover:text-destructive transition-colors duration-100"
              aria-label="Delete host"
              @click.stop="deleteHost(host)"
            >
              <Trash2 class="size-3" :stroke-width="1.75" />
            </button>
          </div>
          <ChevronRight class="size-3.5 text-muted-foreground opacity-0 group-hover:opacity-100 transition-opacity" :stroke-width="1.75" />
        </div>
      </div>

      <!-- Empty state -->
      <div v-else class="flex flex-col items-center justify-center py-16 px-6 gap-3 text-center">
        <Server class="size-8 text-muted-foreground/50" :stroke-width="1.5" />
        <div>
          <p class="text-[14px] font-medium text-foreground">No hosts yet</p>
          <p class="text-[12px] text-muted-foreground mt-1">Add your first SSH host to get started</p>
        </div>
        <Button size="sm" @click="addHost">
          <Plus class="size-3.5" :stroke-width="1.75" />
          Add Host
        </Button>
      </div>
    </div>

    <HostForm v-if="showForm" :host="editingHost" @close="showForm = false" />
  </div>
</template>
