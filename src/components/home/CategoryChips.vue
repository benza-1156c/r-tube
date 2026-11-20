<script setup lang="ts">
import type { FilterChip } from "@/bindings";

const props = withDefaults(
  defineProps<{
    chips?: FilterChip[];
    selected?: string;
  }>(),
  {
    chips: () => [],
    selected: "All",
  },
);

const emit = defineEmits<{
  select: [chip: FilterChip];
}>();

function select(chip: FilterChip) {
  emit("select", chip);
}
</script>

<template>
  <div
    v-if="chips.length > 0"
    class="sticky top-0 z-30 flex items-center gap-2 overflow-x-auto py-3 px-6 bg-background/80 backdrop-blur-sm no-scrollbar border-b border-border/20"
  >
    <button
      v-for="chip in chips"
      :key="chip.text"
      type="button"
      :class="[
        'whitespace-nowrap rounded-lg px-3.5 py-1.5 text-xs font-medium transition-colors cursor-pointer',
        selected === chip.text
          ? 'bg-foreground text-background shadow-xs'
          : 'bg-muted/60 text-foreground/80 hover:bg-muted',
      ]"
      @click="select(chip)"
    >
      {{ chip.text }}
    </button>
  </div>
</template>

<style scoped>
.no-scrollbar::-webkit-scrollbar {
  display: none;
}
.no-scrollbar {
  -ms-overflow-style: none;
  scrollbar-width: none;
}
</style>
