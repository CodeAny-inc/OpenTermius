<script setup lang="ts">
import { cn } from "../../lib/cn";

const props = withDefaults(
  defineProps<{
    modelValue?: string | number;
    type?: string;
    placeholder?: string;
    disabled?: boolean;
    class?: string;
  }>(),
  {
    type: "text",
  },
);

const emit = defineEmits<{
  "update:modelValue": [string | number];
  keydown: [KeyboardEvent];
  focus: [];
  blur: [];
}>();
</script>

<template>
  <input
    :type="props.type"
    :value="props.modelValue"
    :placeholder="props.placeholder"
    :disabled="props.disabled"
    :class="cn(
      'flex h-8 w-full rounded-md border border-input bg-background px-3 text-[13px] text-foreground',
      'placeholder:text-muted-foreground',
      'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background',
      'disabled:cursor-not-allowed disabled:opacity-50',
      'transition-colors duration-100',
      props.class,
    )"
    @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
    @keydown="emit('keydown', $event)"
    @focus="emit('focus')"
    @blur="emit('blur')"
  />
</template>
