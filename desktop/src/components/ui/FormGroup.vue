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
  <div :class="cn('flex flex-col space-y-1.5', $props.class)">
    <slot />
  </div>
</template>
