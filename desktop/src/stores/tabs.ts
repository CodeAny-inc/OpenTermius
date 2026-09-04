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

// Drop position for drag-and-drop pane repositioning
export type DropPosition = "top" | "bottom" | "left" | "right" | "center";

export const useTabsStore = defineStore("tabs", () => {
  const tabs = ref<Tab[]>([]);
  const activeTabId = ref<string | null>(null);
  const activePaneId = ref<string | null>(null);

  // Drag state
  const draggedPaneId = ref<string | null>(null);
  const dragOverPaneId = ref<string | null>(null);
  const dragOverPosition = ref<DropPosition | null>(null);

  const activeTab = computed(() =>
    tabs.value.find((t) => t.id === activeTabId.value) ?? null,
  );

  const activePane = computed(() => {
    if (!activeTab.value || !activePaneId.value) return null;
    return findPaneInTree(activeTab.value.tree, activePaneId.value);
  });

  function newTab(host?: Host): Tab {
    const pane = makePane(host);
    const tab: Tab = {
      id: tabId(),
      title: host?.label ?? "Local",
      tree: pane,
    };
    tabs.value.push(tab);
    activeTabId.value = tab.id;
    activePaneId.value = pane.id;
    return tab;
  }

  function closeTab(id: string) {
    const tab = tabs.value.find((t) => t.id === id);
    if (tab) {
      closeTreeSessions(tab.tree);
    }
    tabs.value = tabs.value.filter((t) => t.id !== id);
    if (activeTabId.value === id) {
      activeTabId.value = tabs.value[0]?.id ?? null;
      if (activeTabId.value) {
        const first = firstPane(tabs.value[0].tree);
        activePaneId.value = first?.id ?? null;
      } else {
        activePaneId.value = null;
      }
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
    const tab = tabs.value.find((t) => t.id === id);
    if (tab) {
      const first = firstPane(tab.tree);
      activePaneId.value = first?.id ?? null;
    }
  }

  function setActivePane(id: string) {
    activePaneId.value = id;
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
    activePaneId.value = newPane.id;
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

    // Find the sibling before removing (for focus)
    const sibling = findSibling(tab.tree, paneIdToClose);
    tab.tree = removeFromTree(tab.tree, paneIdToClose);
    if (sibling) {
      const first = firstPane(sibling);
      if (first) activePaneId.value = first.id;
    }
  }

  function setPaneConnected(paneIdStr: string, sessionId: string) {
    const tab = activeTab.value;
    if (!tab) return;
    tab.tree = updatePaneInTree(tab.tree, paneIdStr, (p) => ({
      ...p,
      sessionId,
      connected: true,
    }));
  }

  function setPaneTitle(paneIdStr: string, title: string) {
    const tab = activeTab.value;
    if (!tab) return;
    tab.tree = updatePaneInTree(tab.tree, paneIdStr, (p) => ({ ...p, title }));
  }

  function setRatio(splitId: string, ratio: number) {
    const tab = activeTab.value;
    if (!tab) return;
    tab.tree = updateSplitInTree(tab.tree, splitId, ratio);
  }

  // Find the first pane in a tree (for focusing)
  function firstPane(node: PaneTree): Pane | null {
    if (isPane(node)) return node;
    return firstPane(node.first) ?? firstPane(node.second);
  }

  // --- Drag and drop ---

  function startDrag(paneId: string) {
    draggedPaneId.value = paneId;
  }

  function endDrag() {
    draggedPaneId.value = null;
    dragOverPaneId.value = null;
    dragOverPosition.value = null;
  }

  function setDragOver(paneIdStr: string, position: DropPosition) {
    if (draggedPaneId.value === paneIdStr) return; // can't drop on self
    dragOverPaneId.value = paneIdStr;
    dragOverPosition.value = position;
  }

  function clearDragOver() {
    dragOverPaneId.value = null;
    dragOverPosition.value = null;
  }

  // Drop a pane onto another pane at a position.
  // "center" = swap the two panes.
  // "top/bottom" = split vertically, "left/right" = split horizontally.
  function dropPane(
    targetPaneId: string,
    position: DropPosition,
  ) {
    if (!draggedPaneId.value || draggedPaneId.value === targetPaneId) {
      endDrag();
      return;
    }

    const tab = activeTab.value;
    if (!tab) {
      endDrag();
      return;
    }

    const draggedId = draggedPaneId.value;

    if (position === "center") {
      // Swap the two panes in the tree
      tab.tree = swapPanesInTree(tab.tree, draggedId, targetPaneId);
    } else {
      // Remove dragged pane from tree, then insert at target position
      const draggedPane = findPaneInTree(tab.tree, draggedId);
      if (!draggedPane) {
        endDrag();
        return;
      }

      // Deep clone the dragged pane (so we can re-insert it)
      const paneCopy: Pane = { ...draggedPane };

      // Remove dragged pane from tree
      tab.tree = removeFromTree(tab.tree, draggedId);

      // If removing caused the target to no longer exist, abort
      const target = findPaneInTree(tab.tree, targetPaneId);
      if (!target) {
        endDrag();
        return;
      }

      // Insert the dragged pane at the target position
      const direction = position === "left" || position === "right"
        ? "horizontal" as const
        : "vertical" as const;

      tab.tree = replaceInTree(tab.tree, targetPaneId, (old) => {
        const newSplit: SplitNode = {
          id: paneId(),
          direction,
          ratio: 0.5,
          first: position === "left" || position === "top" ? paneCopy : old,
          second: position === "left" || position === "top" ? old : paneCopy,
        };
        return newSplit;
      });
    }

    activePaneId.value = draggedId;
    endDrag();
  }

  // Move a pane to a different tab (extract + create in target tab)
  function movePaneToTab(paneIdStr: string, targetTabId: string) {
    const sourceTab = activeTab.value;
    if (!sourceTab) return;

    const pane = findPaneInTree(sourceTab.tree, paneIdStr);
    if (!pane) return;

    // Deep clone
    const paneCopy: Pane = { ...pane };

    // Remove from source
    if (isPane(sourceTab.tree) && sourceTab.tree.id === paneIdStr) {
      // It's the only pane — just switch tabs
      setActiveTab(targetTabId);
      return;
    }
    sourceTab.tree = removeFromTree(sourceTab.tree, paneIdStr);

    // Add to target tab as a new split or root
    const targetTab = tabs.value.find((t) => t.id === targetTabId);
    if (!targetTab) return;

    if (isPane(targetTab.tree)) {
      // Target has single pane — split it
      targetTab.tree = {
        id: paneId(),
        direction: "horizontal",
        ratio: 0.5,
        first: targetTab.tree,
        second: paneCopy,
      };
    } else {
      // Target already has splits — add as new split at root
      targetTab.tree = {
        id: paneId(),
        direction: "horizontal",
        ratio: 0.5,
        first: targetTab.tree,
        second: paneCopy,
      };
    }

    setActiveTab(targetTabId);
    activePaneId.value = paneCopy.id;
  }

  // --- Keyboard navigation ---

  function navigatePane(direction: "up" | "down" | "left" | "right") {
    const tab = activeTab.value;
    if (!tab || !activePaneId.value) return;

    const allPanes = collectPanes(tab.tree);
    if (allPanes.length <= 1) return;

    // Simple navigation: find next pane in the given direction
    // This is a heuristic — we find all panes and pick the nearest one
    // in the requested direction based on tree structure.
    const currentId = activePaneId.value;
    const next = findAdjacentPane(tab.tree, currentId, direction);
    if (next) {
      activePaneId.value = next;
    }
  }

  // Reorder tabs
  function reorderTab(fromIndex: number, toIndex: number) {
    if (fromIndex === toIndex) return;
    if (fromIndex < 0 || fromIndex >= tabs.value.length) return;
    if (toIndex < 0 || toIndex >= tabs.value.length) return;

    const [moved] = tabs.value.splice(fromIndex, 1);
    tabs.value.splice(toIndex, 0, moved);
  }

  return {
    tabs,
    activeTabId,
    activePaneId,
    activeTab,
    activePane,
    // drag state
    draggedPaneId,
    dragOverPaneId,
    dragOverPosition,
    // actions
    newTab,
    closeTab,
    setActiveTab,
    setActivePane,
    splitPane,
    closePane,
    setPaneConnected,
    setPaneTitle,
    setRatio,
    firstPane,
    // drag-drop
    startDrag,
    endDrag,
    setDragOver,
    clearDragOver,
    dropPane,
    movePaneToTab,
    // navigation
    navigatePane,
    // tab reorder
    reorderTab,
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

function findPaneInTree(tree: PaneTree, id: string): Pane | null {
  if (isPane(tree)) {
    return tree.id === id ? tree : null;
  }
  return findPaneInTree(tree.first, id) ?? findPaneInTree(tree.second, id);
}

// Find the sibling subtree of a pane (for focus after removal)
function findSibling(tree: PaneTree, targetId: string): PaneTree | null {
  if (isPane(tree)) return null;

  if (isPane(tree.first) && tree.first.id === targetId) {
    return tree.second;
  }
  if (isPane(tree.second) && tree.second.id === targetId) {
    return tree.first;
  }

  return findSibling(tree.first, targetId) ?? findSibling(tree.second, targetId);
}

// Swap two panes in the tree (keeping the tree structure)
function swapPanesInTree(
  tree: PaneTree,
  idA: string,
  idB: string,
): PaneTree {
  if (isPane(tree)) {
    if (tree.id === idA) return { ...tree, id: idB };
    if (tree.id === idB) return { ...tree, id: idA };
    return tree;
  }
  return {
    ...tree,
    first: swapPanesInTree(tree.first, idA, idB),
    second: swapPanesInTree(tree.second, idA, idB),
  };
}

// Collect all panes in depth-first order
function collectPanes(tree: PaneTree): Pane[] {
  if (isPane(tree)) return [tree];
  return [...collectPanes(tree.first), ...collectPanes(tree.second)];
}

// Find an adjacent pane in a given direction (heuristic based on tree structure)
function findAdjacentPane(
  tree: PaneTree,
  currentId: string,
  direction: "up" | "down" | "left" | "right",
): string | null {
  const panes = collectPanes(tree);
  const currentIdx = panes.findIndex((p) => p.id === currentId);
  if (currentIdx < 0) return null;

  // Simple heuristic: navigate to next/prev pane
  // For a proper implementation we'd need spatial coordinates,
  // but this works reasonably for most split layouts.
  if (direction === "right" || direction === "down") {
    return panes[currentIdx + 1]?.id ?? null;
  } else {
    return panes[currentIdx - 1]?.id ?? null;
  }
}
