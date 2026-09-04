<script setup lang="ts">
import { useTabsStore } from "../stores/tabs";
import { Plus, X } from "lucide-vue-next";
import { cn } from "../lib/cn";
import SplitView from "./SplitView.vue";

const tabs = useTabsStore();

function newTab() {
  tabs.newTab();
}

function switchTab(id: string) {
  tabs.setActiveTab(id);
}

function closeTab(id: string) {
  tabs.closeTab(id);
}
</script>

<template>
  <!-- Tab bar -->
  <div class="flex h-11 items-center gap-0.5 border-b border-border bg-sidebar px-1.5 overflow-x-auto">
    <div
      v-for="tab in tabs.tabs"
      :key="tab.id"
      class="group flex h-8 items-center gap-1.5 rounded-md px-2.5 text-[12px] cursor-pointer transition-colors duration-100"
      :class="cn(
        tab.id === tabs.activeTabId
          ? 'bg-background text-foreground'
          : 'text-muted-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground',
      )"
      @click="switchTab(tab.id)"
    >
      <span class="truncate max-w-[120px]">{{ tab.title }}</span>
      <button
        class="opacity-0 group-hover:opacity-100 transition-opacity rounded p-0.5 hover:bg-muted"
        @click.stop="closeTab(tab.id)"
        aria-label="Close tab"
      >
        <X class="size-3" :stroke-width="1.75" />
      </button>
    </div>
    <div class="ml-auto px-1">
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
