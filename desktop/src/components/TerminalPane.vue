<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import { useTabsStore, type Pane, type DropPosition } from "../stores/tabs";
import { useHostsStore } from "../stores/hosts";
import * as api from "../api";
import {
  SplitSquareHorizontal,
  SplitSquareVertical,
  X,
  Circle,
  CircleDot,
  GripVertical,
  Maximize2,
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

const containerRef = ref<HTMLElement | null>(null);
const paneRef = ref<HTMLElement | null>(null);
let term: Terminal | null = null;
let fitAddon: FitAddon | null = null;
let unlistenData: UnlistenFn | null = null;
let unlistenClosed: UnlistenFn | null = null;
let resizeObserver: ResizeObserver | null = null;
let currentSessionId: string | null = null;

const isActive = computed(() => tabs.activePaneId === props.pane.id);
const isDragging = computed(() => tabs.draggedPaneId === props.pane.id);
const isDragOver = computed(() => tabs.dragOverPaneId === props.pane.id);
const dragPosition = computed(() => tabs.dragOverPosition);
const someoneDragging = computed(() => tabs.draggedPaneId !== null);

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
    }
  });

  term.onData((data) => {
    if (sessionId) {
      const bytes = Array.from(new TextEncoder().encode(data));
      api.sessionWrite(sessionId, bytes);
    }
  });

  await connectSession(sessionId);

  resizeObserver = new ResizeObserver(() => {
    if (fitAddon && term) {
      try {
        fitAddon.fit();
        if (sessionId) {
          api.sessionResize(sessionId, term.cols, term.rows);
        }
      } catch (e) {
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

  if (props.pane.terminalType === "local") {
    try {
      await api.createLocalTerminal(sessionId, term.cols, term.rows);
      tabs.setPaneConnected(props.pane.id, sessionId);
    } catch (e) {
      if (term) {
        term.write(`\x1b[31mFailed to create local terminal: ${e}\x1b[0m\r\n`);
      }
    }
  } else if (props.pane.hostId) {
    const host = hosts.hosts.find((h) => h.id === props.pane.hostId);
    if (!host) {
      term?.write("Host not found\r\n");
      return;
    }
    try {
      const password = null;
      await api.connectSsh(sessionId, host, password, term.cols, term.rows);
      tabs.setPaneConnected(props.pane.id, sessionId);
    } catch (e) {
      term?.write(`\x1b[31mConnection failed: ${e}\x1b[0m\r\n`);
    }
  }
}

// --- Focus handling ---
function focusPane() {
  tabs.setActivePane(props.pane.id);
  if (term) {
    term.focus();
  }
}

// --- Drag and drop ---
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
  if (e.dataTransfer) {
    e.dataTransfer.dropEffect = "move";
  }

  // Determine drop position based on mouse position within the pane
  if (!paneRef.value) return;
  const rect = paneRef.value.getBoundingClientRect();
  const x = (e.clientX - rect.left) / rect.width;
  const y = (e.clientY - rect.top) / rect.height;

  // Center zone (swap) is a small area in the middle
  const centerSize = 0.2;
  const isCenter = x > 0.5 - centerSize && x < 0.5 + centerSize &&
                   y > 0.5 - centerSize && y < 0.5 + centerSize;

  let position: DropPosition;
  if (isCenter) {
    position = "center";
  } else {
    // Determine which edge zone
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
  // Only clear if we're actually leaving the pane (not entering a child)
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
  const isCenter = x > 0.5 - centerSize && x < 0.5 + centerSize &&
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
      'ring-1 ring-inset ring-primary/40': isActive && !isDragOver,
      'ring-2 ring-inset ring-primary': isActive && isDragOver,
    }"
    @click="focusPane"
    @dragover="onDragOver"
    @dragleave="onDragLeave"
    @drop="onDrop"
  >
    <!-- Pane header — draggable, compact -->
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
      <GripVertical
        class="size-3 text-muted-foreground/50 shrink-0"
        :stroke-width="1.75"
      />
      <CircleDot
        v-if="pane.connected"
        class="size-2.5 text-green-500 shrink-0"
        :stroke-width="0"
        fill="currentColor"
      />
      <Circle
        v-else
        class="size-2.5 text-muted-foreground shrink-0"
        :stroke-width="1.75"
      />
      <span
        class="text-[11px] truncate flex-1"
        :class="isActive ? 'text-foreground font-medium' : 'text-muted-foreground'"
      >
        {{ pane.title }}
      </span>
      <div class="flex items-center gap-0.5">
        <button
          class="flex h-5 w-5 items-center justify-center rounded text-muted-foreground hover:bg-sidebar-accent hover:text-foreground transition-colors duration-100"
          aria-label="Split horizontally"
          title="Split horizontally"
          @click.stop="emit('split-h')"
        >
          <SplitSquareHorizontal class="size-3" :stroke-width="1.75" />
        </button>
        <button
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

    <!-- Terminal body -->
    <div ref="containerRef" class="flex-1 overflow-hidden bg-black"></div>

    <!-- Drop zone overlay (shown when dragging over this pane) -->
    <div
      v-if="isDragOver && someoneDragging && !isDragging"
      class="absolute inset-0 z-20 pointer-events-none"
    >
      <!-- Top zone -->
      <div
        v-if="dragPosition === 'top'"
        class="absolute top-0 left-0 right-0 h-1/2 bg-primary/20 border-2 border-primary border-bottom-0 rounded-t-md flex items-center justify-center"
      >
        <div class="bg-primary text-primary-foreground text-[11px] font-medium px-2 py-1 rounded-md flex items-center gap-1">
          <Maximize2 class="size-3" :stroke-width="1.75" />
          Drop to split top
        </div>
      </div>
      <!-- Bottom zone -->
      <div
        v-else-if="dragPosition === 'bottom'"
        class="absolute bottom-0 left-0 right-0 h-1/2 bg-primary/20 border-2 border-primary border-top-0 rounded-b-md flex items-center justify-center"
      >
        <div class="bg-primary text-primary-foreground text-[11px] font-medium px-2 py-1 rounded-md flex items-center gap-1">
          <Maximize2 class="size-3" :stroke-width="1.75" />
          Drop to split bottom
        </div>
      </div>
      <!-- Left zone -->
      <div
        v-else-if="dragPosition === 'left'"
        class="absolute top-0 bottom-0 left-0 w-1/2 bg-primary/20 border-2 border-primary border-right-0 rounded-l-md flex items-center justify-center"
      >
        <div class="bg-primary text-primary-foreground text-[11px] font-medium px-2 py-1 rounded-md flex items-center gap-1">
          <Maximize2 class="size-3" :stroke-width="1.75" />
          Drop to split left
        </div>
      </div>
      <!-- Right zone -->
      <div
        v-else-if="dragPosition === 'right'"
        class="absolute top-0 bottom-0 right-0 w-1/2 bg-primary/20 border-2 border-primary border-left-0 rounded-r-md flex items-center justify-center"
      >
        <div class="bg-primary text-primary-foreground text-[11px] font-medium px-2 py-1 rounded-md flex items-center gap-1">
          <Maximize2 class="size-3" :stroke-width="1.75" />
          Drop to split right
        </div>
      </div>
      <!-- Center zone (swap) -->
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

    <!-- Dragging indicator (semi-transparent when being dragged) -->
    <div
      v-if="isDragging"
      class="absolute inset-0 z-30 bg-muted/50 pointer-events-none flex items-center justify-center"
    >
      <div class="text-[12px] font-medium text-muted-foreground">Moving pane...</div>
    </div>
  </div>
</template>
