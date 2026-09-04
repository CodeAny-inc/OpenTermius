<script setup lang="ts">
withDefaults(
  defineProps<{
    for?: string;
  }>(),
  {},
);
</script>

<template>
  <label
    :for="$props.for"
    class="text-[12px] font-medium text-muted-foreground leading-none"
  >
    <slot />
  </label>
</template>
