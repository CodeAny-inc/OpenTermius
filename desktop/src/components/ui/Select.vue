<script setup lang="ts">
import { cn } from "../../lib/cn";

const props = withDefaults(
  defineProps<{
    modelValue?: string;
    disabled?: boolean;
    class?: string;
  }>(),
  {},
);

const emit = defineEmits<{
  "update:modelValue": [string];
}>();
</script>

<template>
  <select
    :value="props.modelValue"
    :disabled="props.disabled"
    :class="cn(
      'flex h-8 w-full rounded-md border border-input bg-background px-3 text-[13px] text-foreground',
      'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background',
      'disabled:cursor-not-allowed disabled:opacity-50',
      'transition-colors duration-100',
      props.class,
    )"
    @change="emit('update:modelValue', ($event.target as HTMLSelectElement).value)"
  >
    <slot />
  </select>
</template>
