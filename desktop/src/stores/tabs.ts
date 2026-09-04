import { defineStore } from "pinia";
import { ref, computed } from "vue";
import * as api from "../api";
import type { Host, PaneLayout } from "../types";

export interface Pane {
  id: string;
  sessionId: string | null;
  hostId: string | null;
  terminalType: "ssh" | "local";
  title: string;
  connected: boolean;
  closing: boolean;
}

export interface SplitNode {
  id: string;
  direction: "horizontal" | "vertical";
  ratio: number;
  first: PaneTree;
  second: PaneTree;
}

export type PaneTree = Pane | SplitNode;

export function isPane(node: PaneTree): node is Pane {
  return !("direction" in node);
}

export function isSplit(node: PaneTree): node is SplitNode {
  return "direction" in node;
}

export function paneId(): string {
  return crypto.randomUUID();
}

export interface Tab {
  id: string;
  title: string;
  tree: PaneTree;
}

function tabId(): string {
  return crypto.randomUUID();
}

function makePane(host?: Host): Pane {
  return {
    id: paneId(),
    sessionId: null,
    hostId: host?.id ?? null,
    terminalType: host ? "ssh" : "local",
    title: host?.label ?? "Local Terminal",
    connected: false,
    closing: false,
  };
}

export const useTabsStore = defineStore("tabs", () => {
  const tabs = ref<Tab[]>([]);
  const activeTabId = ref<string | null>(null);

  const activeTab = computed(() =>
    tabs.value.find((t) => t.id === activeTabId.value) ?? null,
  );

  function newTab(host?: Host): Tab {
    const pane = makePane(host);
    const tab: Tab = {
      id: tabId(),
      title: host?.label ?? "Local",
      tree: pane,
    };
    tabs.value.push(tab);
    activeTabId.value = tab.id;
    return tab;
  }

  function closeTab(id: string) {
    // Close all sessions in the tab
    const tab = tabs.value.find((t) => t.id === id);
    if (tab) {
      closeTreeSessions(tab.tree);
    }
    tabs.value = tabs.value.filter((t) => t.id !== id);
    if (activeTabId.value === id) {
      activeTabId.value = tabs.value[0]?.id ?? null;
    }
  }

  function closeTreeSessions(node: PaneTree) {
    if (isPane(node)) {
      if (node.sessionId) {
        api.closeSession(node.sessionId);
      }
    } else {
      closeTreeSessions(node.first);
      closeTreeSessions(node.second);
    }
  }

  function setActiveTab(id: string) {
    activeTabId.value = id;
  }

  // Split the active pane in the active tab
  function splitPane(
    paneIdToSplit: string,
    direction: "horizontal" | "vertical",
    host?: Host,
  ): Pane | null {
    const tab = activeTab.value;
    if (!tab) return null;

    const newPane = makePane(host);
    tab.tree = replaceInTree(tab.tree, paneIdToSplit, (old) => ({
      id: paneId(),
      direction,
      ratio: 0.5,
      first: old,
      second: newPane,
    }));
    return newPane;
  }

  function closePane(paneIdToClose: string) {
    const tab = activeTab.value;
    if (!tab) return;

    // If the tree is a single pane, close the tab
    if (isPane(tab.tree) && tab.tree.id === paneIdToClose) {
      closeTab(tab.id);
      return;
    }

    // Find and remove the pane, replacing the parent split with the sibling
    tab.tree = removeFromTree(tab.tree, paneIdToClose);
  }

  function setPaneConnected(paneIdStr: string, sessionId: string) {
    const tab = activeTab.value;
    if (!tab) return;
    updatePaneInTree(tab.tree, paneIdStr, (p) => ({
      ...p,
      sessionId,
      connected: true,
    }));
  }

  function setPaneTitle(paneIdStr: string, title: string) {
    const tab = activeTab.value;
    if (!tab) return;
    updatePaneInTree(tab.tree, paneIdStr, (p) => ({ ...p, title }));
  }

  function setRatio(splitId: string, ratio: number) {
    const tab = activeTab.value;
    if (!tab) return;
    updateSplitInTree(tab.tree, splitId, ratio);
  }

  // Find the first pane in a tree (for focusing)
  function firstPane(node: PaneTree): Pane | null {
    if (isPane(node)) return node;
    return firstPane(node.first) ?? firstPane(node.second);
  }

  return {
    tabs,
    activeTabId,
    activeTab,
    newTab,
    closeTab,
    setActiveTab,
    splitPane,
    closePane,
    setPaneConnected,
    setPaneTitle,
    setRatio,
    firstPane,
  };
});

// --- tree helpers ---

function replaceInTree(
  tree: PaneTree,
  targetId: string,
  replace: (old: Pane) => PaneTree,
): PaneTree {
  if (isPane(tree)) {
    if (tree.id === targetId) return replace(tree);
    return tree;
  }
  return {
    ...tree,
    first: replaceInTree(tree.first, targetId, replace),
    second: replaceInTree(tree.second, targetId, replace),
  };
}

function removeFromTree(tree: PaneTree, targetId: string): PaneTree {
  if (isPane(tree)) return tree;

  // Check if either child is the target
  if (isPane(tree.first) && tree.first.id === targetId) {
    // Close session if any
    if (tree.first.sessionId) api.closeSession(tree.first.sessionId);
    return tree.second;
  }
  if (isPane(tree.second) && tree.second.id === targetId) {
    if (tree.second.sessionId) api.closeSession(tree.second.sessionId);
    return tree.first;
  }

  // Recurse
  return {
    ...tree,
    first: removeFromTree(tree.first, targetId),
    second: removeFromTree(tree.second, targetId),
  };
}

function updatePaneInTree(
  tree: PaneTree,
  targetId: string,
  update: (p: Pane) => Pane,
): PaneTree {
  if (isPane(tree)) {
    if (tree.id === targetId) return update(tree);
    return tree;
  }
  return {
    ...tree,
    first: updatePaneInTree(tree.first, targetId, update),
    second: updatePaneInTree(tree.second, targetId, update),
  };
}

function updateSplitInTree(tree: PaneTree, targetId: string, ratio: number): PaneTree {
  if (isPane(tree)) return tree;
  if (tree.id === targetId) return { ...tree, ratio };
  return {
    ...tree,
    first: updateSplitInTree(tree.first, targetId, ratio),
    second: updateSplitInTree(tree.second, targetId, ratio),
  };
}
