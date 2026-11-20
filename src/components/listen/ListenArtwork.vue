<script setup lang="ts">
import { ListMusic } from "@lucide/vue";

defineOptions({ name: "ListenArtwork" });

defineProps<{
  thumbnailUrl?: string;
  title?: string;
  isPlaying?: boolean;
}>();
</script>

<template>
  <div class="relative w-64 h-64 sm:w-80 sm:h-80 md:w-96 md:h-96 my-4 select-none mx-auto">
    <!-- Ambient glow behind cover -->
    <div
      v-if="thumbnailUrl"
      class="absolute inset-0 rounded-3xl bg-cover bg-center blur-2xl opacity-40 scale-95 -z-10 transition-opacity duration-700"
      :style="{ backgroundImage: `url(${thumbnailUrl})` }"
    />

    <!-- Album Art Cover -->
    <img
      v-if="thumbnailUrl"
      :src="thumbnailUrl"
      :alt="title || 'Track Cover'"
      class="w-full h-full object-cover rounded-3xl shadow-2xl ring-1 ring-border/50 transition-transform duration-500"
      :class="{
        'scale-[1.01] shadow-primary/10': isPlaying,
      }"
    />
    <div
      v-else
      class="w-full h-full rounded-3xl bg-muted flex items-center justify-center shadow-xl ring-1 ring-border/50"
    >
      <ListMusic class="h-16 w-16 text-muted-foreground/50" />
    </div>
  </div>
</template>

