<script setup lang="ts">
import { cn } from "../../lib/cn";
import { buttonVariants, type ButtonVariants } from "./button-variants";

const props = withDefaults(
  defineProps<{
    variant?: ButtonVariants["variant"];
    size?: ButtonVariants["size"];
    class?: string;
    disabled?: boolean;
    type?: "button" | "submit" | "reset";
  }>(),
  {
    variant: "default",
    size: "default",
    type: "button",
  },
);
</script>

<template>
  <button
    :type="props.type"
    :disabled="props.disabled"
    :class="cn(buttonVariants({ variant: props.variant, size: props.size }), props.class)"
  >
    <slot />
  </button>
</template>
