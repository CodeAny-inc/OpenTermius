<script setup lang="ts">
withDefaults(
  defineProps<{
    label: string;
    side?: "top" | "bottom" | "left" | "right";
  }>(),
  {
    side: "top",
  },
);
</script>

<template>
  <div class="relative group inline-flex">
    <slot />
    <span
      :class="[
        'absolute z-50 pointer-events-none',
        'rounded-md border border-border bg-popover px-2 py-1',
        'text-[11px] text-popover-foreground shadow-popover',
        'opacity-0 group-hover:opacity-100 transition-opacity duration-140',
        'whitespace-nowrap',
        $props.side === 'top' && 'bottom-full left-1/2 -translate-x-1/2 mb-1.5',
        $props.side === 'bottom' && 'top-full left-1/2 -translate-x-1/2 mt-1.5',
        $props.side === 'left' && 'right-full top-1/2 -translate-y-1/2 mr-1.5',
        $props.side === 'right' && 'left-full top-1/2 -translate-y-1/2 ml-1.5',
      ]"
    >
      {{ $props.label }}
    </span>
  </div>
</template>
