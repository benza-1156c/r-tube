<script setup lang="ts">
const emit = defineEmits<{
  select: [category: string];
}>();

const categories = [
  "All",
  "Music",
  "Podcasts",
  "Gaming",
  "Technology",
  "Live",
  "Anime",
  "Lo-fi",
  "News",
  "Mixes",
  "Recently uploaded",
];

const selectedCategory = ref("All");

function select(cat: string) {
  selectedCategory.value = cat;
  emit("select", cat);
}
</script>

<template>
  <div class="sticky top-0 z-30 flex items-center gap-2 overflow-x-auto py-3 px-6 bg-background/80 backdrop-blur-sm no-scrollbar border-b border-border/20">
    <button
      v-for="cat in categories"
      :key="cat"
      type="button"
      :class="[
        'whitespace-nowrap rounded-lg px-3.5 py-1.5 text-xs font-medium transition-colors cursor-pointer',
        selectedCategory === cat
          ? 'bg-foreground text-background shadow-xs'
          : 'bg-muted/60 text-foreground/80 hover:bg-muted'
      ]"
      @click="select(cat)"
    >
      {{ cat }}
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
