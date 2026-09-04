<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import { useTabsStore, type Pane } from "../stores/tabs";
import { useHostsStore } from "../stores/hosts";
import * as api from "../api";
import {
  SplitSquareHorizontal,
  SplitSquareVertical,
  X,
  Circle,
  CircleDot,
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
let term: Terminal | null = null;
let fitAddon: FitAddon | null = null;
let unlistenData: UnlistenFn | null = null;
let unlistenClosed: UnlistenFn | null = null;
let resizeObserver: ResizeObserver | null = null;
// Local session ID — set before terminal creation, used in all callbacks
let currentSessionId: string | null = null;

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

  // Generate session ID BEFORE setting up listeners
  currentSessionId = crypto.randomUUID();
  const sessionId = currentSessionId;

  // Set up event listeners BEFORE creating the terminal on the backend.
  // The backend starts emitting data immediately after creation, so if
  // we set up listeners after, we lose the initial output (shell prompt).
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

  // Register onData callback using local sessionId
  term.onData((data) => {
    if (sessionId) {
      const bytes = Array.from(new TextEncoder().encode(data));
      api.sessionWrite(sessionId, bytes);
    }
  });

  // Now create the terminal on the backend
  await connectSession(sessionId);

  // Set up resize observer
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
  // Close the session on the backend
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
</script>

<template>
  <div class="flex flex-col w-full h-full">
    <!-- Pane header — compact, quiet -->
    <div class="flex h-6 items-center gap-1.5 px-2 bg-sidebar border-b border-sidebar-border flex-shrink-0">
      <CircleDot
        v-if="pane.connected"
        class="size-2.5 text-green-500"
        :stroke-width="0"
        fill="currentColor"
      />
      <Circle
        v-else
        class="size-2.5 text-muted-foreground"
        :stroke-width="1.75"
      />
      <span class="text-[11px] text-muted-foreground truncate flex-1">{{ pane.title }}</span>
      <div class="flex items-center gap-0.5">
        <button
          class="flex h-5 w-5 items-center justify-center rounded text-muted-foreground hover:bg-sidebar-accent hover:text-foreground transition-colors duration-100"
          aria-label="Split horizontally"
          title="Split horizontally"
          @click="emit('split-h')"
        >
          <SplitSquareHorizontal class="size-3" :stroke-width="1.75" />
        </button>
        <button
          class="flex h-5 w-5 items-center justify-center rounded text-muted-foreground hover:bg-sidebar-accent hover:text-foreground transition-colors duration-100"
          aria-label="Split vertically"
          title="Split vertically"
          @click="emit('split-v')"
        >
          <SplitSquareVertical class="size-3" :stroke-width="1.75" />
        </button>
        <button
          class="flex h-5 w-5 items-center justify-center rounded text-muted-foreground hover:bg-destructive/20 hover:text-destructive transition-colors duration-100"
          aria-label="Close pane"
          title="Close pane"
          @click="emit('close')"
        >
          <X class="size-3" :stroke-width="1.75" />
        </button>
      </div>
    </div>
    <!-- Terminal body -->
    <div ref="containerRef" class="flex-1 overflow-hidden bg-black"></div>
  </div>
</template>
