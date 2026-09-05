<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onBeforeUnmount } from "vue";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { SearchAddon } from "@xterm/addon-search";
import "@xterm/xterm/css/xterm.css";
import { useTabsStore, type Pane, type DropPosition } from "../stores/tabs";
import { useHostsStore } from "../stores/hosts";
import { useIdentitiesStore } from "../stores/identities";
import { useVaultStore } from "../stores/vault";
import { useUiStore } from "../stores/ui";
import * as api from "../api";
import { SplitSquareHorizontal, SplitSquareVertical, X, GripVertical, Maximize2, Minimize2,
  RotateCw, Search, ChevronUp, ChevronDown, CaseSensitive, Regex, WholeWord, Loader2 } from "lucide-vue-next";
import type { UnlistenFn } from "@tauri-apps/api/event";

const props = withDefaults(defineProps<{ pane: Pane; tabId: string; visible?: boolean }>(), { visible: true });
const emit = defineEmits<{ "split-h": []; "split-v": []; close: [] }>();
const tabs = useTabsStore();
const hosts = useHostsStore();
const identities = useIdentitiesStore();
const vault = useVaultStore();
const ui = useUiStore();
const containerRef = ref<HTMLElement | null>(null);
const paneRef = ref<HTMLElement | null>(null);
const connecting = ref(false);
const error = ref("");
const showSearch = ref(false);
const searchQuery = ref("");
const searchCaseSensitive = ref(false);
const searchRegex = ref(false);
const searchWholeWord = ref(false);
const searchInputRef = ref<HTMLInputElement | null>(null);
let term: Terminal | null = null;
let fitAddon: FitAddon | null = null;
let searchAddon: SearchAddon | null = null;
let observer: ResizeObserver | null = null;
const listeners: UnlistenFn[] = [];
let currentSessionId: string | null = null;
let disposed = false;
let sessionEnded = false;
let listenersReady = false;
const isActive = computed(() => props.visible && tabs.activeTabId === props.tabId && tabs.activePaneId === props.pane.id);
const isFullscreen = computed(() => ui.fullscreenPaneId === props.pane.id);
const isDragging = computed(() => tabs.draggedPaneId === props.pane.id);
const isDragOver = computed(() => tabs.dragOverPaneId === props.pane.id);
const status = computed(() => connecting.value ? "Connecting" : props.pane.connected ? "Connected" : "Disconnected");
const otherTabs = computed(() => tabs.tabs.filter(t => t.id !== props.tabId));

function fit(focus = false) {
  if (disposed || !props.visible || !term || !fitAddon || !containerRef.value?.clientWidth || !containerRef.value.clientHeight) return;
  try {
    fitAddon.fit();
    if (currentSessionId && props.pane.connected) void api.sessionResize(currentSessionId, term.cols, term.rows).catch(() => {});
    if (focus && isActive.value && !showSearch.value) term.focus();
  } catch { /* ResizeObserver can run during teardown. */ }
}
watch([isActive, isFullscreen, () => props.visible], () => nextTick(() => fit(true)), { flush: "post" });
function focusPane() { tabs.setActivePane(props.pane.id); if (!showSearch.value) term?.focus(); }
function writeError(message: string) {
  error.value = message;
  term?.write(`\r\n\x1b[31m${message}\x1b[0m\r\n`);
}
async function connectSession() {
  if (disposed || connecting.value || !term || !listenersReady) return;
  connecting.value = true;
  error.value = "";
  tabs.setPaneDisconnected(props.pane.id);
  const previous = currentSessionId;
  const sessionId = crypto.randomUUID();
  currentSessionId = sessionId;
  sessionEnded = false;
  try {
    if (previous) await api.closeSession(previous).catch(() => {});
    if (disposed) return;
    fit();
    const terminal = term;
    if (!terminal) return;
    if (props.pane.terminalType === "local") {
      await api.createLocalTerminal(sessionId, terminal.cols, terminal.rows);
    } else {
      const host = hosts.hosts.find(h => h.id === props.pane.hostId);
      if (!host) throw new Error("Host not found. Check the saved host configuration.");
      const identity = identities.identities.find(i => i.id === host.identity_id);
      const auth = identity?.auth ?? host.auth;
      if (auth === "publickey" && !vault.unlocked) {
        if (!await ui.requestVaultUnlock()) throw new Error("Connection cancelled: vault remains locked.");
        if (disposed) return;
      }
      try {
        await api.connectSsh(sessionId, host, null, terminal.cols, terminal.rows);
      } catch (cause) {
        const message = String(cause);
        if (!message.includes("vault passphrase required") && !message.includes("vault required")) throw cause;
        if (!await ui.requestVaultUnlock()) throw new Error("Connection cancelled: vault remains locked.");
        if (disposed) return;
        await api.connectSsh(sessionId, host, null, terminal.cols, terminal.rows);
      }
    }
    // Closing while connect is pending must not resurrect a session or leak it.
    if (disposed || props.pane.closing) {
      await api.closeSession(sessionId).catch(() => {});
      return;
    }
    if (!sessionEnded) tabs.setPaneConnected(props.pane.id, sessionId);
    fit(true);
  } catch (cause) {
    if (!disposed) writeError(`Connection failed: ${String(cause)}`);
  } finally {
    connecting.value = false;
  }
}

onMounted(async () => {
  if (!containerRef.value) return;
  term = new Terminal({ fontSize: 13,
    fontFamily: "'SFMono-Regular', 'SF Mono', 'Cascadia Code', 'Roboto Mono', ui-monospace, monospace",
    theme: { background: "#000000", foreground: "#e6e9ef", cursor: "#4f9cf9", selectionBackground: "#264f78" },
    cursorBlink: true, scrollback: 10000 });
  fitAddon = new FitAddon();
  searchAddon = new SearchAddon();
  term.loadAddon(fitAddon);
  term.loadAddon(searchAddon);
  term.attachCustomKeyEventHandler(event => {
    const command = event.metaKey || event.ctrlKey;
    if (command && event.key.toLowerCase() === "f") {
      if (event.type === "keydown") { event.preventDefault(); openSearch(); }
      return false;
    }
    const directions = { ArrowLeft: "left", ArrowRight: "right", ArrowUp: "up", ArrowDown: "down" } as const;
    const direction = directions[event.key as keyof typeof directions];
    if (command && !event.shiftKey && !event.altKey && direction) {
      if (event.type === "keydown") { event.preventDefault(); tabs.navigatePane(direction); }
      return false;
    }
    return true;
  });
  term.open(containerRef.value);
  term.onData(data => {
    if (currentSessionId && props.pane.connected && !connecting.value)
      void api.sessionWrite(currentSessionId, Array.from(new TextEncoder().encode(data))).catch(cause => {
        if (!disposed) writeError(`Write failed: ${String(cause)}`);
      });
  });
  observer = new ResizeObserver(() => fit());
  observer.observe(containerRef.value);
  fit();
  try {
    const dataListener = await api.onSessionData(event => {
      if (!disposed && event.session_id === currentSessionId) term?.write(new Uint8Array(event.data));
    });
    if (disposed) { dataListener(); return; }
    listeners.push(dataListener);
    const closeListener = await api.onSessionClosed(event => {
      if (!disposed && event.session_id === currentSessionId) {
        sessionEnded = true;
        tabs.setPaneDisconnected(props.pane.id);
        writeError(`Session closed: ${event.reason}`);
      }
    });
    if (disposed) { closeListener(); return; }
    listeners.push(closeListener);
    listenersReady = true;
    await connectSession();
  } catch (cause) {
    if (!disposed) writeError(`Could not initialize terminal: ${String(cause)}`);
  }
});
onBeforeUnmount(() => {
  disposed = true;
  listeners.forEach(unlisten => unlisten());
  observer?.disconnect();
  term?.dispose();
  term = null;
  // Explicit close in the store already owns an established session's teardown.
  if (currentSessionId && (!props.pane.closing || props.pane.sessionId !== currentSessionId))
    void api.closeSession(currentSessionId).catch(() => {});
});
function openSearch() { showSearch.value = true; nextTick(() => { searchInputRef.value?.focus(); searchInputRef.value?.select(); }); }
function closeSearch() { showSearch.value = false; searchAddon?.clearDecorations(); term?.focus(); }
function doSearch(previous = false) {
  if (!searchQuery.value) { searchAddon?.clearDecorations(); return; }
  const options = { caseSensitive: searchCaseSensitive.value, regex: searchRegex.value, wholeWord: searchWholeWord.value,
    decorations: { matchOverviewRuler: "#4f9cf9", activeMatchColorOverviewRuler: "#f59e0b", matchBackground: "#264f78", activeMatchBackground: "#f59e0b80" } };
  try {
    if (previous) searchAddon?.findPrevious(searchQuery.value, options);
    else searchAddon?.findNext(searchQuery.value, options);
  } catch { /* An incomplete regular expression should not break the terminal. */ }
}
function searchKey(event: KeyboardEvent) {
  if (event.key === "Escape") { event.preventDefault(); closeSearch(); }
  else if (event.key === "Enter" || ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "g")) {
    event.preventDefault(); doSearch(event.shiftKey);
  }
}
function startDrag(event: DragEvent) {
  tabs.startDrag(props.pane.id);
  if (event.dataTransfer) { event.dataTransfer.effectAllowed = "move"; event.dataTransfer.setData("text/plain", props.pane.id); }
}
function dropPosition(event: DragEvent): DropPosition {
  const bounds = paneRef.value!.getBoundingClientRect();
  const x = (event.clientX - bounds.left) / bounds.width;
  const y = (event.clientY - bounds.top) / bounds.height;
  if (x > 0.3 && x < 0.7 && y > 0.3 && y < 0.7) return "center";
  const edges: [DropPosition, number][] = [["left", x], ["right", 1 - x], ["top", y], ["bottom", 1 - y]];
  return edges.sort((a, b) => a[1] - b[1])[0][0];
}
function dragOver(event: DragEvent) {
  if (!tabs.draggedPaneId || isDragging.value || !paneRef.value) return;
  event.preventDefault();
  tabs.setDragOver(props.pane.id, dropPosition(event));
}
function drop(event: DragEvent) {
  if (!tabs.draggedPaneId || !paneRef.value) return;
  event.preventDefault();
  tabs.dropPane(props.pane.id, dropPosition(event));
}
function moveToTab(event: Event) {
  const select = event.target as HTMLSelectElement;
  if (select.value) tabs.movePaneToTab(props.pane.id, select.value);
  select.value = "";
}
</script>

<template>
  <div ref="paneRef" class="flex h-full w-full flex-col bg-black"
    :class="[isFullscreen ? 'fixed inset-0 z-[90]' : 'relative', isActive ? 'ring-1 ring-inset ring-primary/60' : '']"
    :data-session-id="pane.sessionId" :data-host-id="pane.hostId" :data-active="isActive"
    @click="focusPane" @dragover="dragOver" @drop="drop"
    @dragleave="!paneRef?.contains($event.relatedTarget as Node) && tabs.clearDragOver()">
    <header class="flex h-9 shrink-0 items-center gap-2 border-b border-border bg-sidebar px-2"
      :class="isActive ? 'bg-primary/10' : ''">
      <div class="flex min-w-0 flex-1 cursor-grab items-center gap-1.5" draggable="true"
        :aria-label="`Drag pane ${pane.title}`" @dragstart="startDrag" @dragend="tabs.endDrag()">
        <GripVertical class="size-3 shrink-0 text-muted-foreground" />
        <span class="size-1.5 shrink-0 rounded-full" :class="pane.connected ? 'bg-green-500' : 'bg-muted-foreground'" />
        <span class="truncate text-xs font-medium" :title="pane.title">{{ pane.title }}</span>
      </div>
      <span class="hidden shrink-0 text-[10px] text-muted-foreground lg:inline" role="status">{{ status }}</span>
      <Loader2 v-if="connecting" class="size-3 animate-spin text-muted-foreground" />
      <div class="flex shrink-0 items-center gap-0.5" @click.stop>
        <select v-if="otherTabs.length" class="w-7 rounded bg-transparent text-xs text-muted-foreground" aria-label="Move pane to tab" title="Move this session to another tab" @change="moveToTab">
          <option value="">↗</option><option v-for="tab in otherTabs" :key="tab.id" :value="tab.id">{{ tab.title }}</option>
        </select>
        <button v-if="!pane.connected" class="pane-button" :disabled="connecting || !listenersReady" aria-label="Reconnect" title="Reconnect" @click="connectSession"><RotateCw class="size-3.5" /></button>
        <button class="pane-button" aria-label="Search in terminal" title="Search (Cmd/Ctrl+F)" @click="openSearch"><Search class="size-3.5" /></button>
        <button class="pane-button" :aria-label="isFullscreen ? 'Exit fullscreen' : 'Fullscreen'" @click="ui.toggleFullscreen(pane.id)"><Minimize2 v-if="isFullscreen" class="size-3.5" /><Maximize2 v-else class="size-3.5" /></button>
        <button v-if="!isFullscreen" class="pane-button" aria-label="Split horizontally" title="Split right with a local shell" @click="emit('split-h')"><SplitSquareHorizontal class="size-3.5" /></button>
        <button v-if="!isFullscreen" class="pane-button" aria-label="Split vertically" title="Split below with a local shell" @click="emit('split-v')"><SplitSquareVertical class="size-3.5" /></button>
        <button class="pane-button hover:!text-destructive" aria-label="Close pane" title="Close this session" @click="emit('close')"><X class="size-3.5" /></button>
      </div>
    </header>
    <div v-if="error" class="flex shrink-0 items-center gap-2 border-b border-border bg-background px-3 py-2 text-xs" role="alert">
      <span class="min-w-0 flex-1 text-muted-foreground">{{ error }}</span>
      <button v-if="!pane.connected" class="shrink-0 text-primary disabled:opacity-50" :disabled="connecting" @click.stop="connectSession">Reconnect</button>
    </div>
    <div ref="containerRef" class="min-h-0 flex-1 overflow-hidden bg-black" />
    <div v-if="showSearch" class="absolute right-2 top-10 z-40 flex max-w-[calc(100%-16px)] flex-wrap items-center gap-1 rounded-md border border-border bg-background p-1 shadow-lg" @click.stop @keydown.stop="searchKey">
      <input ref="searchInputRef" v-model="searchQuery" aria-label="Search terminal output" placeholder="Search..." class="h-7 w-36 min-w-0 bg-transparent px-2 text-xs outline-none" @input="doSearch()" />
      <button class="pane-button" :aria-pressed="searchCaseSensitive" aria-label="Case sensitive" @click="searchCaseSensitive = !searchCaseSensitive; doSearch()"><CaseSensitive class="size-3.5" /></button>
      <button class="pane-button" :aria-pressed="searchWholeWord" aria-label="Whole word" @click="searchWholeWord = !searchWholeWord; doSearch()"><WholeWord class="size-3.5" /></button>
      <button class="pane-button" :aria-pressed="searchRegex" aria-label="Regular expression" @click="searchRegex = !searchRegex; doSearch()"><Regex class="size-3.5" /></button>
      <button class="pane-button" aria-label="Previous match" @click="doSearch(true)"><ChevronUp class="size-3.5" /></button>
      <button class="pane-button" aria-label="Next match" @click="doSearch()"><ChevronDown class="size-3.5" /></button>
      <button class="pane-button" aria-label="Close search" @click="closeSearch"><X class="size-3.5" /></button>
    </div>
    <div v-if="isDragOver && tabs.draggedPaneId && !isDragging" class="pointer-events-none absolute inset-0 z-30 flex items-center justify-center border-2 border-dashed border-primary bg-primary/20">
      <span class="rounded bg-primary px-3 py-2 text-xs text-primary-foreground">{{ tabs.dragOverPosition === 'center' ? 'Swap pane positions' : `Move pane to ${tabs.dragOverPosition}` }}</span>
    </div>
  </div>
</template>

<style scoped>
.pane-button { @apply flex h-7 w-7 items-center justify-center rounded text-muted-foreground hover:bg-sidebar-accent hover:text-foreground disabled:opacity-40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring; }
.pane-button[aria-pressed="true"] { @apply bg-primary/20 text-primary; }
</style>
