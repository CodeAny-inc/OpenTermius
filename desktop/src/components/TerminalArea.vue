<script setup lang="ts">
import { useTabsStore } from "../stores/tabs";
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
  <div class="tab-bar">
    <div
      v-for="tab in tabs.tabs"
      :key="tab.id"
      class="tab-item"
      :class="{ active: tab.id === tabs.activeTabId }"
      @click="switchTab(tab.id)"
    >
      <span>{{ tab.title }}</span>
      <span class="tab-close" @click.stop="closeTab(tab.id)">×</span>
    </div>
    <div class="tab-bar-actions">
      <button class="btn btn-sm" @click="newTab">+ New Tab</button>
    </div>
  </div>
  <div class="terminal-area">
    <div v-if="tabs.activeTab" style="width: 100%; height: 100%;">
      <SplitView :node="tabs.activeTab.tree" :tab-id="tabs.activeTab.id" />
    </div>
    <div v-else class="empty-state">
      <p>No active terminal</p>
      <button class="btn" @click="newTab">Open Local Terminal</button>
    </div>
  </div>
</template>
