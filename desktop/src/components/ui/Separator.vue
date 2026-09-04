<script setup lang="ts">
import { cn } from "../../lib/cn";

withDefaults(
  defineProps<{
    orientation?: "horizontal" | "vertical";
    class?: string;
  }>(),
  {
    orientation: "horizontal",
  },
);
</script>

<template>
  <div
    :class="cn(
      'bg-border',
      $props.orientation === 'horizontal' ? 'h-px w-full' : 'w-px h-full',
      $props.class,
    )"
    role="separator"
  />
</template>
