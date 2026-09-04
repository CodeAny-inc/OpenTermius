<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick } from "vue";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import { useTabsStore, type Pane } from "../stores/tabs";
import { useHostsStore } from "../stores/hosts";
import * as api from "../api";
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

onMounted(async () => {
  if (!containerRef.value) return;

  term = new Terminal({
    fontSize: 13,
    fontFamily: "Menlo, Monaco, 'Courier New', monospace",
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

  // Connect to session
  await connectSession();

  // Handle terminal input
  term.onData((data) => {
    if (props.pane.sessionId) {
      const bytes = Array.from(new TextEncoder().encode(data));
      api.sessionWrite(props.pane.sessionId, bytes);
    }
  });

  // Listen for session data events
  unlistenData = await api.onSessionData((event) => {
    if (event.session_id === props.pane.sessionId && term) {
      const data = new Uint8Array(event.data);
      term.write(data);
    }
  });

  unlistenClosed = await api.onSessionClosed((event) => {
    if (event.session_id === props.pane.sessionId && term) {
      term.write(`\r\n\x1b[31m[session closed: ${event.reason}]\x1b[0m\r\n`);
    }
  });

  // Handle resize
  resizeObserver = new ResizeObserver(() => {
    if (fitAddon && term) {
      fitAddon.fit();
      if (props.pane.sessionId) {
        api.sessionResize(
          props.pane.sessionId,
          term.cols,
          term.rows,
        );
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
});

async function connectSession() {
  if (!term || !fitAddon) return;
  const sessionId = crypto.randomUUID();

  if (props.pane.terminalType === "local") {
    await api.createLocalTerminal(sessionId, term.cols, term.rows);
    tabs.setPaneConnected(props.pane.id, sessionId);
  } else if (props.pane.hostId) {
    const host = hosts.hosts.find((h) => h.id === props.pane.hostId);
    if (!host) {
      term.write("Host not found\r\n");
      return;
    }
    try {
      const password = null; // TODO: get from keychain
      await api.connectSsh(sessionId, host, password, term.cols, term.rows);
      tabs.setPaneConnected(props.pane.id, sessionId);
    } catch (e) {
      term.write(`\x1b[31mConnection failed: ${e}\x1b[0m\r\n`);
    }
  }
}
</script>

<template>
  <div class="terminal-pane">
    <div class="terminal-header">
      <span>{{ pane.title }}</span>
      <span v-if="pane.connected" style="color: var(--ok)">●</span>
      <span v-else style="color: var(--muted)">○</span>
      <div class="pane-actions">
        <button title="Split horizontally" @click="emit('split-h')">⊞</button>
        <button title="Split vertically" @click="emit('split-v')">⊟</button>
        <button title="Close pane" @click="emit('close')">×</button>
      </div>
    </div>
    <div ref="containerRef" class="terminal-body"></div>
  </div>
</template>
