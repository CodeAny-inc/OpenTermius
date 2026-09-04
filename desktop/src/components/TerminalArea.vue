<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { useTabsStore } from "../stores/tabs";
import { useUiStore } from "../stores/ui";
import { Plus, X, GripHorizontal } from "lucide-vue-next";
import { cn } from "../lib/cn";
import SplitView from "./SplitView.vue";

const tabs = useTabsStore();
const ui = useUiStore();

// Tab drag state
const draggedTabId = ref<string | null>(null);
const dragOverTabId = ref<string | null>(null);

function newTab() {
  tabs.newTab();
}

function switchTab(id: string) {
  tabs.setActiveTab(id);
}

function closeTab(id: string) {
  tabs.closeTab(id);
}

// --- Tab drag and drop (reordering) ---
function onTabDragStart(e: DragEvent, tabId: string) {
  draggedTabId.value = tabId;
  if (e.dataTransfer) {
    e.dataTransfer.effectAllowed = "move";
    e.dataTransfer.setData("text/plain", tabId);
  }
}

function onTabDragEnd() {
  draggedTabId.value = null;
  dragOverTabId.value = null;
}

function onTabDragOver(e: DragEvent, tabId: string) {
  if (!draggedTabId.value || draggedTabId.value === tabId) return;
  e.preventDefault();
  if (e.dataTransfer) {
    e.dataTransfer.dropEffect = "move";
  }
  dragOverTabId.value = tabId;
}

function onTabDrop(e: DragEvent, targetTabId: string) {
  e.preventDefault();
  if (!draggedTabId.value || draggedTabId.value === targetTabId) {
    onTabDragEnd();
    return;
  }

  const fromIndex = tabs.tabs.findIndex((t) => t.id === draggedTabId.value);
  const toIndex = tabs.tabs.findIndex((t) => t.id === targetTabId);
  if (fromIndex >= 0 && toIndex >= 0) {
    tabs.reorderTab(fromIndex, toIndex);
  }
  onTabDragEnd();
}

// --- Keyboard navigation ---
function onKeyDown(e: KeyboardEvent) {
  // Cmd/Ctrl + Arrow keys to navigate between panes
  if ((e.metaKey || e.ctrlKey) && !e.shiftKey && !e.altKey) {
    if (e.key === "ArrowUp") {
      e.preventDefault();
      tabs.navigatePane("up");
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      tabs.navigatePane("down");
    } else if (e.key === "ArrowLeft") {
      e.preventDefault();
      tabs.navigatePane("left");
    } else if (e.key === "ArrowRight") {
      e.preventDefault();
      tabs.navigatePane("right");
    }
  }
}

onMounted(() => {
  window.addEventListener("keydown", onKeyDown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeyDown);
});
</script>

<template>
  <!-- Tab bar (hidden when a pane is fullscreen) -->
  <div v-show="!ui.fullscreenPaneId" class="flex h-11 items-center gap-0.5 border-b border-border bg-sidebar px-1.5 pl-11 md:pl-1.5 overflow-x-auto">
    <div
      v-for="tab in tabs.tabs"
      :key="tab.id"
      class="group flex h-8 items-center gap-1.5 rounded-md px-2 sm:px-2.5 text-[12px] cursor-pointer transition-colors duration-100 relative shrink-0"
      :class="cn(
        tab.id === tabs.activeTabId
          ? 'bg-background text-foreground'
          : 'text-muted-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground',
        dragOverTabId === tab.id && draggedTabId !== tab.id && 'ring-2 ring-primary ring-inset',
        draggedTabId === tab.id && 'opacity-50',
      )"
      draggable="true"
      @click="switchTab(tab.id)"
      @dragstart="onTabDragStart($event, tab.id)"
      @dragend="onTabDragEnd"
      @dragover="onTabDragOver($event, tab.id)"
      @drop="onTabDrop($event, tab.id)"
    >
      <GripHorizontal class="size-3 text-muted-foreground/40 opacity-0 group-hover:opacity-100 transition-opacity hidden sm:block" :stroke-width="1.75" />
      <span class="truncate max-w-[80px] sm:max-w-[120px]">{{ tab.title }}</span>
      <button
        class="opacity-0 group-hover:opacity-100 transition-opacity rounded p-0.5 hover:bg-muted"
        @click.stop="closeTab(tab.id)"
        aria-label="Close tab"
      >
        <X class="size-3" :stroke-width="1.75" />
      </button>
    </div>
    <div class="ml-auto px-1 shrink-0">
      <button
        class="flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground hover:bg-sidebar-accent hover:text-foreground transition-colors duration-100"
        @click="newTab"
        aria-label="New tab"
      >
        <Plus class="size-3.5" :stroke-width="1.75" />
      </button>
    </div>
  </div>

  <!-- Terminal area -->
  <div class="flex-1 relative overflow-hidden bg-black">
    <div v-if="tabs.activeTab" class="w-full h-full">
      <SplitView :node="tabs.activeTab.tree" :tab-id="tabs.activeTab.id" />
    </div>
    <div v-else class="flex flex-col items-center justify-center h-full gap-3 text-muted-foreground">
      <p class="text-[14px]">No active terminal</p>
      <button
        class="inline-flex h-8 items-center gap-1.5 rounded-md bg-primary px-3 text-[13px] font-medium text-primary-foreground hover:bg-primary/90 transition-colors duration-100"
        @click="newTab"
      >
        <Plus class="size-3.5" :stroke-width="1.75" />
        Open Local Terminal
      </button>
    </div>
  </div>
</template>
