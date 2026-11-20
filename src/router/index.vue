<script setup lang="ts">
import { ref } from "vue";
import type { FilterChip } from "@/bindings";

const filterChips = ref<FilterChip[]>([]);
const currentCategoryParams = ref<string | null>(null);
const selectedCategoryText = ref("All");

function onCategorySelect(chip: FilterChip) {
  currentCategoryParams.value = chip.params ?? null;
  selectedCategoryText.value = chip.text;
}

function onUpdateChips(chips: FilterChip[]) {
  filterChips.value = chips;

  const active = chips.find((c) => c.isSelected);
  if (active && currentCategoryParams.value === null) {
    selectedCategoryText.value = active.text;
  }
}
</script>

<template>
  <div class="flex flex-col min-h-full relative">
    <CategoryChips
      :chips="filterChips"
      :selected="selectedCategoryText"
      @select="onCategorySelect"
    />

    <KeepAlive>
      <VideoFeed
        :key="currentCategoryParams ?? 'all'"
        :category-params="currentCategoryParams"
        @update-chips="onUpdateChips"
      />
    </KeepAlive>
  </div>
</template>
