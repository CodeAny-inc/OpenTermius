<script setup lang="ts">
import { cn } from "../../lib/cn";

const props = withDefaults(
  defineProps<{
    modelValue?: string;
    placeholder?: string;
    rows?: number;
    disabled?: boolean;
    class?: string;
  }>(),
  {
    rows: 4,
  },
);

const emit = defineEmits<{
  "update:modelValue": [string];
  keydown: [KeyboardEvent];
}>();
</script>

<template>
  <textarea
    :value="props.modelValue"
    :placeholder="props.placeholder"
    :rows="props.rows"
    :disabled="props.disabled"
    :class="cn(
      'flex w-full rounded-md border border-input bg-background px-3 py-2 text-[13px] text-foreground',
      'placeholder:text-muted-foreground',
      'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background',
      'disabled:cursor-not-allowed disabled:opacity-50',
      'transition-colors duration-100 resize-none',
      props.class,
    )"
    @input="emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
    @keydown="emit('keydown', $event)"
  />
</template>
