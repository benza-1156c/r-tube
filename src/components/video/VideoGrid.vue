<script setup lang="ts">
import type { VideoItem } from "./VideoCard.vue";

defineProps<{
  videos: VideoItem[];
  loading?: boolean;
}>();
</script>

<template>
  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-x-4 gap-y-7 p-6">
    <!-- Skeleton Loaders -->
    <template v-if="loading">
      <div v-for="i in 8" :key="i" class="flex flex-col gap-3">
        <Skeleton class="aspect-video w-full rounded-xl" />
        <div class="flex gap-3">
          <Skeleton class="h-9 w-9 rounded-full shrink-0" />
          <div class="flex-1 space-y-2">
            <Skeleton class="h-4 w-5/6" />
            <Skeleton class="h-3 w-1/2" />
          </div>
        </div>
      </div>
    </template>

    <!-- Video Cards -->
    <template v-else>
      <VideoCard
        v-for="video in videos"
        :key="video.id"
        :video="video"
      />
    </template>
  </div>
</template>

