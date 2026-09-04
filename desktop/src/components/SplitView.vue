<script setup lang="ts">
import { ref } from "vue";
import { useTabsStore, isPane, isSplit, type PaneTree, type SplitNode } from "../stores/tabs";
import TerminalPane from "./TerminalPane.vue";

const props = defineProps<{
  node: PaneTree;
  tabId: string;
}>();

const tabs = useTabsStore();
const dragging = ref(false);

function startDrag(e: MouseEvent) {
  e.preventDefault();
  dragging.value = true;
  const node = props.node as SplitNode;
  const container = (e.currentTarget as HTMLElement).parentElement!;
  const isHorizontal = node.direction === "horizontal";
  const startPos = isHorizontal ? e.clientX : e.clientY;
  const containerSize = isHorizontal ? container.offsetWidth : container.offsetHeight;
  const startRatio = node.ratio;

  function onMove(ev: MouseEvent) {
    const pos = isHorizontal ? ev.clientX : ev.clientY;
    const delta = pos - startPos;
    const deltaRatio = delta / containerSize;
    let newRatio = startRatio + deltaRatio;
    newRatio = Math.max(0.1, Math.min(0.9, newRatio));
    tabs.setRatio(node.id, newRatio);
  }

  function onUp() {
    dragging.value = false;
    window.removeEventListener("mousemove", onMove);
    window.removeEventListener("mouseup", onUp);
  }

  window.addEventListener("mousemove", onMove);
  window.addEventListener("mouseup", onUp);
}

function split(direction: "horizontal" | "vertical") {
  if (isPane(props.node)) {
    tabs.splitPane(props.node.id, direction);
  }
}

function close() {
  if (isPane(props.node)) {
    tabs.closePane(props.node.id);
  }
}
</script>

<template>
  <div v-if="isSplit(node)" class="flex w-full h-full" :class="node.direction === 'horizontal' ? 'flex-row' : 'flex-col'">
    <div class="flex-1 overflow-hidden min-w-[30px] min-h-[30px]" :style="{ flex: node.ratio }">
      <SplitView :node="node.first" :tab-id="tabId" />
    </div>
    <div
      class="flex-shrink-0 bg-border transition-colors duration-100 hover:bg-ring/50"
      :class="[
        node.direction === 'horizontal' ? 'w-1 cursor-col-resize' : 'h-1 cursor-row-resize',
        dragging ? 'bg-ring' : '',
      ]"
      @mousedown="startDrag"
    ></div>
    <div class="flex-1 overflow-hidden min-w-[30px] min-h-[30px]" :style="{ flex: 1 - node.ratio }">
      <SplitView :node="node.second" :tab-id="tabId" />
    </div>
  </div>
  <TerminalPane
    v-else
    :pane="node"
    :tab-id="tabId"
    @split-h="split('horizontal')"
    @split-v="split('vertical')"
    @close="close"
  />
</template>
