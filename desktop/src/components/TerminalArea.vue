<script setup lang="ts">
import { computed, ref, nextTick, onMounted, onUnmounted } from "vue";
import { useTabsStore, collectPanes } from "../stores/tabs";
import { useHostsStore } from "../stores/hosts";
import { useUiStore } from "../stores/ui";
import type { Host } from "../types";
import { Plus, X, Server, TerminalSquare, SplitSquareHorizontal, SplitSquareVertical } from "lucide-vue-next";
import TerminalWorkspace from "./TerminalWorkspace.vue";

const props = withDefaults(defineProps<{ visible?: boolean }>(), { visible: true });
const emit = defineEmits<{ activate: [] }>();
const tabs = useTabsStore();
const hosts = useHostsStore();
const ui = useUiStore();
const draggedTabId = ref<string | null>(null);
const hostDialog = ref<HTMLDialogElement | null>(null);
const hostSearch = ref<HTMLInputElement | null>(null);
const query = ref("");
const placement = ref<"tab" | "horizontal" | "vertical">("horizontal");
const matchingHosts = computed(() => hosts.hosts.filter(h =>
  `${h.label} ${h.hostname} ${h.username}`.toLowerCase().includes(query.value.toLowerCase())));
const paneCount = computed(() => tabs.activeTab ? collectPanes(tabs.activeTab.tree).length : 0);
function activate(id: string) { tabs.setActiveTab(id); emit("activate"); }
function newTab() { tabs.newTab(); emit("activate"); }
function connected(id: string) {
  const tab = tabs.tabs.find(t => t.id === id);
  return tab ? collectPanes(tab.tree).filter(p => p.connected).length : 0;
}
async function chooseHost() {
  query.value = "";
  if (!tabs.activePane) placement.value = "tab";
  hostDialog.value?.showModal();
  await nextTick();
  hostSearch.value?.focus();
}
function openHost(host: Host) {
  if (placement.value === "tab" || !tabs.activePaneId) tabs.newTab(host);
  else tabs.splitPane(tabs.activePaneId, placement.value, host);
  hostDialog.value?.close();
  emit("activate");
}
function split(direction: "horizontal" | "vertical") {
  if (tabs.activePaneId) tabs.splitPane(tabs.activePaneId, direction);
}
function tabDrag(event: DragEvent, id: string) {
  draggedTabId.value = id;
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = "move";
    event.dataTransfer.setData("text/plain", id);
  }
}
function tabDrop(event: DragEvent, target: string) {
  event.preventDefault();
  if (tabs.draggedPaneId) {
    tabs.movePaneToTab(tabs.draggedPaneId, target);
    tabs.endDrag();
    emit("activate");
  } else if (draggedTabId.value) {
    tabs.reorderTab(tabs.tabs.findIndex(t => t.id === draggedTabId.value), tabs.tabs.findIndex(t => t.id === target));
  }
  draggedTabId.value = null;
}
function onKeyDown(event: KeyboardEvent) {
  if (!props.visible || hostDialog.value?.open || event.defaultPrevented) return;
  if ((event.metaKey || event.ctrlKey) && !event.shiftKey && !event.altKey) {
    const directions = { ArrowLeft: "left", ArrowRight: "right", ArrowUp: "up", ArrowDown: "down" } as const;
    const direction = directions[event.key as keyof typeof directions];
    if (direction) { event.preventDefault(); tabs.navigatePane(direction); }
  }
}
onMounted(() => window.addEventListener("keydown", onKeyDown));
onUnmounted(() => window.removeEventListener("keydown", onKeyDown));
</script>

<template>
  <section class="flex flex-col min-h-0 min-w-0" :class="visible ? 'flex-1' : 'shrink-0'" aria-label="Terminal sessions">
    <div v-show="!ui.fullscreenPaneId && (visible || tabs.tabs.length)"
      class="flex h-11 shrink-0 items-center gap-1 border-b border-border bg-sidebar px-2 pl-11 md:pl-2 overflow-x-auto"
      aria-label="Open terminal tabs">
      <div v-for="tab in tabs.tabs" :key="tab.id" class="group flex shrink-0 items-center rounded-md border"
        :class="visible && tab.id === tabs.activeTabId ? 'border-border bg-background shadow-sm' : 'border-transparent text-muted-foreground hover:bg-sidebar-accent'"
        draggable="true" @dragstart="tabDrag($event, tab.id)" @dragend="draggedTabId = null"
        @dragover.prevent @drop="tabDrop($event, tab.id)"
        @auxclick.middle.prevent="tabs.closeTab(tab.id)">
        <button class="flex h-8 items-center gap-2 px-3 text-xs rounded-md focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          :aria-pressed="visible && tab.id === tabs.activeTabId" :data-tab-id="tab.id"
          :title="`Show ${tab.title} terminal · ${connected(tab.id)} connected`" @click="activate(tab.id)">
          <span class="size-1.5 rounded-full" :class="connected(tab.id) ? 'bg-green-500' : 'bg-muted-foreground'" aria-hidden="true" />
          <span class="max-w-[160px] truncate">{{ tab.title }}</span>
          <span v-if="collectPanes(tab.tree).length > 1" class="rounded bg-muted px-1 text-[10px]">{{ collectPanes(tab.tree).length }}</span>
        </button>
        <button class="mr-1 flex size-6 items-center justify-center rounded text-muted-foreground hover:bg-destructive/15 hover:text-destructive focus-visible:ring-2 focus-visible:ring-ring"
          :aria-label="`Close tab ${tab.title}`" @click="tabs.closeTab(tab.id)"><X class="size-3" /></button>
      </div>
      <button class="flex size-8 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring"
        aria-label="New local tab" title="New local tab (Cmd/Ctrl+N)" @click="newTab"><Plus class="size-4" /></button>
      <button v-if="!visible && tabs.tabs.length" class="ml-auto shrink-0 px-2 text-xs text-muted-foreground hover:text-foreground"
        @click="emit('activate')">Back to terminal</button>
    </div>
    <div v-show="visible && !ui.fullscreenPaneId" class="flex shrink-0 flex-wrap items-center gap-2 border-b border-border bg-background px-3 py-2">
      <TerminalSquare class="size-4 text-muted-foreground" />
      <span class="mr-auto text-xs font-medium">Terminal workspace <span class="ml-1 text-muted-foreground">{{ paneCount }} {{ paneCount === 1 ? 'pane' : 'panes' }}</span></span>
      <button class="inline-flex h-8 items-center gap-1.5 rounded-md bg-primary px-3 text-xs font-medium text-primary-foreground hover:bg-primary/90 focus-visible:ring-2 focus-visible:ring-ring"
        @click="chooseHost"><Server class="size-3.5" /> Add host</button>
      <button class="toolbar-button" @click="newTab"><Plus class="size-3.5" /> Local tab</button>
      <button class="toolbar-button" :disabled="!tabs.activePane" title="Add a local shell to the right. Use Add host to split with a remote host."
        @click="split('horizontal')"><SplitSquareHorizontal class="size-3.5" /> Split right</button>
      <button class="toolbar-button" :disabled="!tabs.activePane" title="Add a local shell below. Use Add host to split with a remote host."
        @click="split('vertical')"><SplitSquareVertical class="size-3.5" /> Split below</button>
    </div>
    <TerminalWorkspace v-show="visible && tabs.tabs.length > 0" :visible="visible" />
    <div v-if="visible && tabs.tabs.length === 0" class="flex flex-1 flex-col items-center justify-center gap-4 p-6 text-center">
      <TerminalSquare class="size-12 text-muted-foreground/40" />
      <h1 class="text-lg font-medium">Your terminal workspace</h1>
      <p class="max-w-md text-sm text-muted-foreground">Connect a host or open a local shell. Add multiple hosts side by side without restarting your existing sessions.</p>
      <button class="toolbar-button" @click="chooseHost">Connect a host</button>
    </div>
    <dialog ref="hostDialog" class="w-[min(480px,calc(100vw-32px))] rounded-xl border border-border bg-background p-0 text-foreground shadow-xl backdrop:bg-black/60"
      aria-labelledby="host-picker-title" @keydown.stop>
      <div class="flex items-center justify-between border-b border-border p-4">
        <h2 id="host-picker-title" class="text-sm font-semibold">Add a host to your workspace</h2>
        <button aria-label="Close host picker" class="rounded p-1 hover:bg-muted" @click="hostDialog?.close()"><X class="size-4" /></button>
      </div>
      <div class="space-y-3 p-4">
        <label class="flex items-center gap-3 text-xs">Open host in
          <select v-model="placement" class="min-w-0 flex-1 rounded-md border border-border bg-background p-2" aria-label="Open host in">
            <option value="tab">New tab</option>
            <option value="horizontal" :disabled="!tabs.activePane">Split right of active pane</option>
            <option value="vertical" :disabled="!tabs.activePane">Split below active pane</option>
          </select>
        </label>
        <input ref="hostSearch" v-model="query" aria-label="Search hosts" placeholder="Search by name, address or username"
          class="w-full rounded-md border border-border bg-background p-2 text-sm outline-none focus:ring-2 focus:ring-ring" />
        <div class="max-h-72 space-y-1 overflow-y-auto">
          <button v-for="host in matchingHosts" :key="host.id" class="flex w-full items-center gap-3 rounded-md p-3 text-left hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring"
            :aria-label="`Connect ${host.label}`" @click="openHost(host)">
            <Server class="size-4 shrink-0 text-muted-foreground" />
            <span class="min-w-0"><span class="block truncate text-sm font-medium">{{ host.label }}</span><span class="block truncate text-xs text-muted-foreground">{{ host.username }}@{{ host.hostname }}:{{ host.port }}</span></span>
          </button>
          <p v-if="!matchingHosts.length" class="p-3 text-sm text-muted-foreground">{{ hosts.hosts.length ? 'No matching hosts.' : 'Add your first host in Hosts, then return here.' }}</p>
        </div>
        <p class="text-xs text-muted-foreground">Only the new pane connects. Existing terminals keep their sessions and history.</p>
      </div>
    </dialog>
  </section>
</template>

<style scoped>
.toolbar-button { @apply inline-flex h-8 items-center gap-1.5 rounded-md border border-border px-2.5 text-xs text-muted-foreground hover:bg-muted hover:text-foreground disabled:opacity-40 disabled:cursor-not-allowed focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring; }
</style>
