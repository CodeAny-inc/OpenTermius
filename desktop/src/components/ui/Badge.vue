<script setup lang="ts">
import { cn } from "../../lib/cn";

withDefaults(
  defineProps<{
    class?: string;
  }>(),
  {},
);
</script>

<template>
  <div
    :class="cn(
      'inline-flex items-center rounded-md border px-2 py-0.5 text-[11px] font-medium',
      'border-border bg-muted text-muted-foreground',
      $props.class,
    )"
  >
    <slot />
  </div>
</template>
