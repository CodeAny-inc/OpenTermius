<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch, nextTick } from "vue";
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
import {
  SplitSquareHorizontal,
  SplitSquareVertical,
  X,
  Circle,
  CircleDot,
  GripVertical,
  Maximize2,
  Minimize2,
  RotateCw,
  Search,
  ChevronUp,
  ChevronDown,
  CaseSensitive,
  Regex,
  WholeWord,
} from "lucide-vue-next";
import type { UnlistenFn } from "@tauri-apps/api/event";

const props = defineProps<{
  pane: Pane;
  tabId: string;
}>();

const emit = defineEmits<{
  "split-h": [];
  "split-v": [];
  close: [];
}>();

const tabs = useTabsStore();
const hosts = useHostsStore();
const identities = useIdentitiesStore();
const vault = useVaultStore();
const ui = useUiStore();

const containerRef = ref<HTMLElement | null>(null);
const paneRef = ref<HTMLElement | null>(null);
let term: Terminal | null = null;
let fitAddon: FitAddon | null = null;
let searchAddon: SearchAddon | null = null;
let unlistenData: UnlistenFn | null = null;
let unlistenClosed: UnlistenFn | null = null;
let resizeObserver: ResizeObserver | null = null;
let currentSessionId: string | null = null;
let connectionAttempted = false;

const showSearch = ref(false);
const searchQuery = ref("");
const searchCaseSensitive = ref(false);
const searchRegex = ref(false);
const searchWholeWord = ref(false);
const searchInputRef = ref<HTMLInputElement | null>(null);

const isActive = computed(() => tabs.activePaneId === props.pane.id);
const isDragging = computed(() => tabs.draggedPaneId === props.pane.id);
const isDragOver = computed(() => tabs.dragOverPaneId === props.pane.id);
const dragPosition = computed(() => tabs.dragOverPosition);
const someoneDragging = computed(() => tabs.draggedPaneId !== null);
const isFullscreen = computed(() => ui.fullscreenPaneId === props.pane.id);

watch(
  () => vault.unlocked,
  async (unlocked) => {
    if (unlocked && currentSessionId && !props.pane.connected && connectionAttempted) {
      await connectSession(currentSessionId);
    }
  },
);

watch(isFullscreen, () => {
  nextTick(() => {
    if (fitAddon && term) {
      try {
        fitAddon.fit();
        if (currentSessionId) {
          api.sessionResize(currentSessionId, term.cols, term.rows);
        }
      } catch {
        // ignore
      }
    }
  });
});

watch(
  () => tabs.activeTabId,
  (newActiveTabId) => {
    if (newActiveTabId === props.tabId) {
      nextTick(() => {
        if (fitAddon && term) {
          try {
            fitAddon.fit();
            if (currentSessionId) {
              api.sessionResize(currentSessionId, term.cols, term.rows);
            }
            term.focus();
          } catch {
            // ignore
          }
        }
      });
    }
  },
);

onMounted(async () => {
  if (!containerRef.value) return;

  term = new Terminal({
    fontSize: 13,
    fontFamily: "'SFMono-Regular', 'SF Mono', 'Cascadia Code', 'Roboto Mono', ui-monospace, monospace",
    theme: {
      background: "#000000",
      foreground: "#e6e9ef",
      cursor: "#4f9cf9",
      selectionBackground: "#264f78",
    },
    cursorBlink: true,
    scrollback: 10000,
  });

  fitAddon = new FitAddon();
  term.loadAddon(fitAddon);
  searchAddon = new SearchAddon();
  term.loadAddon(searchAddon);

  // Intercept Ctrl/Cmd+F before xterm can convert it to ^F and send it to the PTY.
  term.attachCustomKeyEventHandler((event) => {
    const isSearchShortcut =
      (event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "f";
    if (!isSearchShortcut) return true;

    if (event.type === "keydown") {
      event.preventDefault();
      openSearch();
    }
    return false;
  });

  term.open(containerRef.value);
  fitAddon.fit();

  currentSessionId = crypto.randomUUID();
  const sessionId = currentSessionId;

  unlistenData = await api.onSessionData((event) => {
    if (event.session_id === sessionId && term) {
      const data = new Uint8Array(event.data);
      term.write(data);
    }
  });

  unlistenClosed = await api.onSessionClosed((event) => {
    if (event.session_id === sessionId && term) {
      term.write(`\r\n\x1b[31m[session closed: ${event.reason}]\x1b[0m\r\n`);
      tabs.setPaneDisconnected(props.pane.id);
    }
  });

  term.onData((data) => {
    if (currentSessionId) {
      const bytes = Array.from(new TextEncoder().encode(data));
      api.sessionWrite(currentSessionId, bytes);
    }
  });

  await connectSession(sessionId);

  resizeObserver = new ResizeObserver(() => {
    if (fitAddon && term) {
      try {
        fitAddon.fit();
        if (currentSessionId) {
          api.sessionResize(currentSessionId, term.cols, term.rows);
        }
      } catch {
        // Ignore resize errors during teardown
      }
    }
  });
  resizeObserver.observe(containerRef.value);
});

onUnmounted(() => {
  if (unlistenData) unlistenData();
  if (unlistenClosed) unlistenClosed();
  if (resizeObserver) resizeObserver.disconnect();
  if (term) term.dispose();
  if (currentSessionId) {
    api.closeSession(currentSessionId).catch(() => {});
  }
});

async function connectSession(sessionId: string) {
  if (!term || !fitAddon) return;
  connectionAttempted = true;

  if (props.pane.terminalType === "local") {
    try {
      await api.createLocalTerminal(sessionId, term.cols, term.rows);
      tabs.setPaneConnected(props.pane.id, sessionId);
    } catch (e) {
      term?.write(`\x1b[31mFailed to create local terminal: ${e}\x1b[0m\r\n`);
    }
    return;
  }

  if (!props.pane.hostId) return;
  const host = hosts.hosts.find((h) => h.id === props.pane.hostId);
  if (!host) {
    term?.write("Host not found\r\n");
    return;
  }

  const needsVault =
    host.auth === "publickey" ||
    (host.identity_id != null &&
      identities.identities.some(
        (i) => i.id === host.identity_id && i.auth === "publickey",
      ));

  if (needsVault && !vault.unlocked) {
    term?.write("\x1b[33mVault is locked — required for SSH key authentication.\x1b[0m\r\n");
    const success = await ui.requestVaultUnlock();
    if (!success) {
      term?.write("\x1b[31mConnection cancelled: vault remains locked.\x1b[0m\r\n");
      return;
    }
  }

  const password = null;
  try {
    await api.connectSsh(sessionId, host, password, term.cols, term.rows);
    tabs.setPaneConnected(props.pane.id, sessionId);
  } catch (e) {
    const msg = String(e);
    if (msg.includes("vault passphrase required") || msg.includes("vault required")) {
      term?.write("\x1b[33mVault passphrase required. Please unlock the vault.\x1b[0m\r\n");
      const success = await ui.requestVaultUnlock();
      if (!success) {
        term?.write("\x1b[31mConnection cancelled.\x1b[0m\r\n");
        return;
      }
      try {
        await api.connectSsh(sessionId, host, password, term.cols, term.rows);
        tabs.setPaneConnected(props.pane.id, sessionId);
      } catch (e2) {
        term?.write(`\x1b[31mConnection failed: ${e2}\x1b[0m\r\n`);
      }
    } else {
      term?.write(`\x1b[31mConnection failed: ${e}\x1b[0m\r\n`);
    }
  }
}

async function reconnect() {
  if (!term) return;

  if (currentSessionId) {
    api.closeSession(currentSessionId).catch(() => {});
  }

  term.reset();

  const newSessionId = crypto.randomUUID();
  currentSessionId = newSessionId;

  if (unlistenData) unlistenData();
  if (unlistenClosed) unlistenClosed();

  unlistenData = await api.onSessionData((event) => {
    if (event.session_id === newSessionId && term) {
      const data = new Uint8Array(event.data);
      term.write(data);
    }
  });

  unlistenClosed = await api.onSessionClosed((event) => {
    if (event.session_id === newSessionId && term) {
      term.write(`\r\n\x1b[31m[session closed: ${event.reason}]\x1b[0m\r\n`);
      tabs.setPaneDisconnected(props.pane.id);
    }
  });

  await connectSession(newSessionId);
}

function focusPane() {
  tabs.setActivePane(props.pane.id);
  term?.focus();
}

function searchOptions() {
  return {
    caseSensitive: searchCaseSensitive.value,
    regex: searchRegex.value,
    wholeWord: searchWholeWord.value,
    decorations: {
      matchOverviewRuler: "#4f9cf9",
      activeMatchColorOverviewRuler: "#f59e0b",
      matchBackground: "#264f78",
      activeMatchBackground: "#f59e0b80",
    },
  };
}

function openSearch() {
  showSearch.value = true;
  nextTick(() => {
    searchInputRef.value?.focus();
    searchInputRef.value?.select();
  });
}

function closeSearch() {
  showSearch.value = false;
  searchQuery.value = "";
  searchAddon?.clearDecorations();
  term?.focus();
}

function doSearch(direction: "next" | "prev") {
  if (!searchAddon || !searchQuery.value) return;
  if (direction === "next") {
    searchAddon.findNext(searchQuery.value, searchOptions());
  } else {
    searchAddon.findPrevious(searchQuery.value, searchOptions());
  }
}

function onSearchInput() {
  if (!searchAddon || !searchQuery.value) {
    searchAddon?.clearDecorations();
    return;
  }
  searchAddon.findNext(searchQuery.value, searchOptions());
}

function onSearchKeydown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    e.preventDefault();
    closeSearch();
  } else if (e.key === "Enter") {
    e.preventDefault();
    doSearch(e.shiftKey ? "prev" : "next");
  } else if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "g") {
    e.preventDefault();
    doSearch(e.shiftKey ? "prev" : "next");
  }
}

function onDragStart(e: DragEvent) {
  tabs.startDrag(props.pane.id);
  if (e.dataTransfer) {
    e.dataTransfer.effectAllowed = "move";
    e.dataTransfer.setData("text/plain", props.pane.id);
  }
}

function onDragEnd() {
  tabs.endDrag();
}

function onDragOver(e: DragEvent) {
  if (!tabs.draggedPaneId || tabs.draggedPaneId === props.pane.id) return;
  e.preventDefault();
  if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
  if (!paneRef.value) return;

  const rect = paneRef.value.getBoundingClientRect();
  const x = (e.clientX - rect.left) / rect.width;
  const y = (e.clientY - rect.top) / rect.height;
  const centerSize = 0.2;
  const isCenter =
    x > 0.5 - centerSize && x < 0.5 + centerSize &&
    y > 0.5 - centerSize && y < 0.5 + centerSize;

  let position: DropPosition;
  if (isCenter) {
    position = "center";
  } else {
    const distLeft = x;
    const distRight = 1 - x;
    const distTop = y;
    const distBottom = 1 - y;
    const minDist = Math.min(distLeft, distRight, distTop, distBottom);
    if (minDist === distLeft) position = "left";
    else if (minDist === distRight) position = "right";
    else if (minDist === distTop) position = "top";
    else position = "bottom";
  }
  tabs.setDragOver(props.pane.id, position);
}

function onDragLeave(e: DragEvent) {
  if (!paneRef.value) return;
  const rect = paneRef.value.getBoundingClientRect();
  const x = e.clientX;
  const y = e.clientY;
  if (x < rect.left || x > rect.right || y < rect.top || y > rect.bottom) {
    tabs.clearDragOver();
  }
}

function onDrop(e: DragEvent) {
  e.preventDefault();
  if (!tabs.draggedPaneId) return;
  if (!paneRef.value) {
    tabs.endDrag();
    return;
  }

  const rect = paneRef.value.getBoundingClientRect();
  const x = (e.clientX - rect.left) / rect.width;
  const y = (e.clientY - rect.top) / rect.height;
  const centerSize = 0.2;
  const isCenter =
    x > 0.5 - centerSize && x < 0.5 + centerSize &&
    y > 0.5 - centerSize && y < 0.5 + centerSize;

  let position: DropPosition;
  if (isCenter) {
    position = "center";
  } else {
    const distLeft = x;
    const distRight = 1 - x;
    const distTop = y;
    const distBottom = 1 - y;
    const minDist = Math.min(distLeft, distRight, distTop, distBottom);
    if (minDist === distLeft) position = "left";
    else if (minDist === distRight) position = "right";
    else if (minDist === distTop) position = "top";
    else position = "bottom";
  }
  tabs.dropPane(props.pane.id, position);
}
</script>

<template>
  <div
    ref="paneRef"
    class="flex flex-col w-full h-full relative transition-all duration-100"
    :class="{
      'ring-1 ring-inset ring-primary/40': isActive && !isDragOver && !isFullscreen,
      'ring-2 ring-inset ring-primary': isActive && isDragOver && !isFullscreen,
      'fixed inset-0 z-[90]': isFullscreen,
    }"
    @click="focusPane"
    @dragover="onDragOver"
    @dragleave="onDragLeave"
    @drop="onDrop"
  >
    <div
      class="flex h-6 items-center gap-1.5 px-2 bg-sidebar border-b border-sidebar-border flex-shrink-0 select-none"
      :class="{
        'cursor-grabbing': isDragging,
        'cursor-grab': !isDragging && someoneDragging === false,
        'bg-primary/10 border-primary/20': isActive,
      }"
      draggable="true"
      @dragstart="onDragStart"
      @dragend="onDragEnd"
      @click="focusPane"
    >
      <GripVertical class="size-3 text-muted-foreground/50 shrink-0" :stroke-width="1.75" />
      <CircleDot
        v-if="pane.connected"
        class="size-2.5 text-green-500 shrink-0"
        :stroke-width="0"
        fill="currentColor"
      />
      <Circle v-else class="size-2.5 text-muted-foreground shrink-0" :stroke-width="1.75" />
      <span
        class="text-[11px] truncate flex-1"
        :class="isActive ? 'text-foreground font-medium' : 'text-muted-foreground'"
      >{{ pane.title }}</span>
      <div class="flex items-center gap-0.5">
        <button
          v-if="!pane.connected && !isFullscreen"
          class="flex h-5 w-5 items-center justify-center rounded text-muted-foreground hover:text-green-500 hover:bg-sidebar-accent transition-colors duration-100"
          aria-label="Reconnect"
          title="Reconnect"
          @click.stop="reconnect"
        >
          <RotateCw class="size-3" :stroke-width="1.75" />
        </button>
        <button
          class="flex h-5 w-5 items-center justify-center rounded text-muted-foreground hover:bg-sidebar-accent hover:text-foreground transition-colors duration-100"
          aria-label="Search in terminal"
          title="Search (Cmd/Ctrl+F)"
          @click.stop="openSearch"
        >
          <Search class="size-3" :stroke-width="1.75" />
        </button>
        <button
          class="flex h-5 w-5 items-center justify-center rounded text-muted-foreground hover:bg-sidebar-accent hover:text-foreground transition-colors duration-100"
          :aria-label="isFullscreen ? 'Exit fullscreen' : 'Fullscreen'"
          :title="isFullscreen ? 'Exit fullscreen (Esc)' : 'Fullscreen'"
          @click.stop="ui.toggleFullscreen(props.pane.id)"
        >
          <Minimize2 v-if="isFullscreen" class="size-3" :stroke-width="1.75" />
          <Maximize2 v-else class="size-3" :stroke-width="1.75" />
        </button>
        <button
          v-if="!isFullscreen"
          class="flex h-5 w-5 items-center justify-center rounded text-muted-foreground hover:bg-sidebar-accent hover:text-foreground transition-colors duration-100"
          aria-label="Split horizontally"
          title="Split horizontally"
          @click.stop="emit('split-h')"
        >
          <SplitSquareHorizontal class="size-3" :stroke-width="1.75" />
        </button>
        <button
          v-if="!isFullscreen"
          class="flex h-5 w-5 items-center justify-center rounded text-muted-foreground hover:bg-sidebar-accent hover:text-foreground transition-colors duration-100"
          aria-label="Split vertically"
          title="Split vertically"
          @click.stop="emit('split-v')"
        >
          <SplitSquareVertical class="size-3" :stroke-width="1.75" />
        </button>
        <button
          class="flex h-5 w-5 items-center justify-center rounded text-muted-foreground hover:bg-destructive/20 hover:text-destructive transition-colors duration-100"
          aria-label="Close pane"
          title="Close pane"
          @click.stop="emit('close')"
        >
          <X class="size-3" :stroke-width="1.75" />
        </button>
      </div>
    </div>

    <div ref="containerRef" class="flex-1 overflow-hidden bg-black"></div>

    <div
      v-if="showSearch"
      class="absolute top-7 right-2 z-40 flex items-center gap-1 rounded-md border border-border bg-background shadow-lg p-1"
      @click.stop
    >
      <input
        ref="searchInputRef"
        v-model="searchQuery"
        type="text"
        placeholder="Search..."
        class="h-6 w-[160px] rounded bg-transparent px-2 text-[12px] text-foreground placeholder:text-muted-foreground outline-none border-none"
        @input="onSearchInput"
        @keydown="onSearchKeydown"
      />
      <button
        class="flex h-6 w-6 items-center justify-center rounded transition-colors duration-100"
        :class="searchCaseSensitive ? 'bg-primary/20 text-primary' : 'text-muted-foreground hover:bg-muted'"
        :title="'Case sensitive' + (searchCaseSensitive ? ' (on)' : '')"
        @click="searchCaseSensitive = !searchCaseSensitive; onSearchInput()"
      >
        <CaseSensitive class="size-3.5" :stroke-width="1.75" />
      </button>
      <button
        class="flex h-6 w-6 items-center justify-center rounded transition-colors duration-100"
        :class="searchWholeWord ? 'bg-primary/20 text-primary' : 'text-muted-foreground hover:bg-muted'"
        :title="'Whole word' + (searchWholeWord ? ' (on)' : '')"
        @click="searchWholeWord = !searchWholeWord; onSearchInput()"
      >
        <WholeWord class="size-3.5" :stroke-width="1.75" />
      </button>
      <button
        class="flex h-6 w-6 items-center justify-center rounded transition-colors duration-100"
        :class="searchRegex ? 'bg-primary/20 text-primary' : 'text-muted-foreground hover:bg-muted'"
        :title="'Regex' + (searchRegex ? ' (on)' : '')"
        @click="searchRegex = !searchRegex; onSearchInput()"
      >
        <Regex class="size-3.5" :stroke-width="1.75" />
      </button>
      <div class="w-px h-4 bg-border mx-0.5"></div>
      <button
        class="flex h-6 w-6 items-center justify-center rounded text-muted-foreground hover:bg-muted hover:text-foreground"
        title="Previous match (Shift+Enter)"
        @click="doSearch('prev')"
      >
        <ChevronUp class="size-3.5" :stroke-width="1.75" />
      </button>
      <button
        class="flex h-6 w-6 items-center justify-center rounded text-muted-foreground hover:bg-muted hover:text-foreground"
        title="Next match (Enter)"
        @click="doSearch('next')"
      >
        <ChevronDown class="size-3.5" :stroke-width="1.75" />
      </button>
      <button
        class="flex h-6 w-6 items-center justify-center rounded text-muted-foreground hover:bg-destructive/20 hover:text-destructive"
        title="Close search (Esc)"
        @click="closeSearch"
      >
        <X class="size-3.5" :stroke-width="1.75" />
      </button>
    </div>

    <div
      v-if="!pane.connected && connectionAttempted"
      class="absolute inset-0 top-6 z-10 flex items-center justify-center bg-black/60 pointer-events-auto"
    >
      <div class="flex flex-col items-center gap-2">
        <Circle class="size-6 text-muted-foreground" :stroke-width="1.5" />
        <span class="text-sm text-muted-foreground">Disconnected</span>
        <button
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-primary text-primary-foreground text-xs font-medium hover:bg-primary/90 transition-colors"
          @click.stop="reconnect"
        >
          <RotateCw class="size-3.5" :stroke-width="2" />
          Reconnect
        </button>
      </div>
    </div>

    <div
      v-if="isDragOver && someoneDragging && !isDragging"
      class="absolute inset-0 z-20 pointer-events-none"
    >
      <div
        v-if="dragPosition === 'top'"
        class="absolute top-0 left-0 right-0 h-1/2 bg-primary/20 border-2 border-primary border-bottom-0 rounded-t-md flex items-center justify-center"
      >
        <div class="bg-primary text-primary-foreground text-[11px] font-medium px-2 py-1 rounded-md flex items-center gap-1">
          <Maximize2 class="size-3" :stroke-width="1.75" />
          Drop to split top
        </div>
      </div>
      <div
        v-else-if="dragPosition === 'bottom'"
        class="absolute bottom-0 left-0 right-0 h-1/2 bg-primary/20 border-2 border-primary border-top-0 rounded-b-md flex items-center justify-center"
      >
        <div class="bg-primary text-primary-foreground text-[11px] font-medium px-2 py-1 rounded-md flex items-center gap-1">
          <Maximize2 class="size-3" :stroke-width="1.75" />
          Drop to split bottom
        </div>
      </div>
      <div
        v-else-if="dragPosition === 'left'"
        class="absolute top-0 bottom-0 left-0 w-1/2 bg-primary/20 border-2 border-primary border-right-0 rounded-l-md flex items-center justify-center"
      >
        <div class="bg-primary text-primary-foreground text-[11px] font-medium px-2 py-1 rounded-md flex items-center gap-1">
          <Maximize2 class="size-3" :stroke-width="1.75" />
          Drop to split left
        </div>
      </div>
      <div
        v-else-if="dragPosition === 'right'"
        class="absolute top-0 bottom-0 right-0 w-1/2 bg-primary/20 border-2 border-primary border-left-0 rounded-r-md flex items-center justify-center"
      >
        <div class="bg-primary text-primary-foreground text-[11px] font-medium px-2 py-1 rounded-md flex items-center gap-1">
          <Maximize2 class="size-3" :stroke-width="1.75" />
          Drop to split right
        </div>
      </div>
      <div
        v-else-if="dragPosition === 'center'"
        class="absolute inset-0 bg-primary/15 border-2 border-dashed border-primary rounded-md flex items-center justify-center"
      >
        <div class="bg-primary text-primary-foreground text-[11px] font-medium px-2 py-1 rounded-md flex items-center gap-1">
          <Maximize2 class="size-3" :stroke-width="1.75" />
          Drop to swap
        </div>
      </div>
    </div>

    <div
      v-if="isDragging"
      class="absolute inset-0 z-30 bg-muted/50 pointer-events-none flex items-center justify-center"
    >
      <div class="text-[12px] font-medium text-muted-foreground">Moving pane...</div>
    </div>
  </div>
</template>
