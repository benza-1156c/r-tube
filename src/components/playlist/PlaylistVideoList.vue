<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from "vue";
import { Loader2, Film } from "@lucide/vue";
import type { PlaylistVideoItem } from "@/bindings";
import PlaylistVideoRow from "./PlaylistVideoRow.vue";

const props = defineProps<{
  videos: PlaylistVideoItem[];
  playlistId: string;
  isLoading: boolean;
  isLoadingMore: boolean;
  hasMore: boolean;
}>();

const emit = defineEmits<{
  (e: "selectVideo", video: PlaylistVideoItem, index: number): void;
  (e: "loadMore"): void;
}>();

const bottomTarget = ref<HTMLElement | null>(null);
let observer: IntersectionObserver | null = null;

onMounted(() => {
  observer = new IntersectionObserver(
    (entries) => {
      const entry = entries[0];
      if (entry.isIntersecting && props.hasMore && !props.isLoading && !props.isLoadingMore) {
        emit("loadMore");
      }
    },
    { rootMargin: "300px" },
  );

  if (bottomTarget.value) {
    observer.observe(bottomTarget.value);
  }
});

onBeforeUnmount(() => {
  if (observer) {
    observer.disconnect();
    observer = null;
  }
});
</script>

<template>
  <div class="flex-1 min-w-0 flex flex-col gap-1">
    <!-- Initial Loading Skeletons -->
    <div v-if="isLoading" class="flex flex-col gap-2">
      <div
        v-for="i in 8"
        :key="i"
        class="flex items-center gap-3 p-2 rounded-xl animate-pulse"
      >
        <div class="w-6 h-4 bg-muted/60 rounded shrink-0" />
        <div class="aspect-video w-36 md:w-40 bg-muted/60 rounded-lg shrink-0" />
        <div class="flex-1 flex flex-col gap-2">
          <div class="h-4 w-3/4 bg-muted/60 rounded" />
          <div class="h-3 w-1/3 bg-muted/40 rounded" />
        </div>
      </div>
    </div>

    <!-- Empty State -->
    <div
      v-else-if="videos.length === 0"
      class="flex flex-col items-center justify-center py-20 text-center"
    >
      <div
        class="h-16 w-16 rounded-full bg-muted flex items-center justify-center text-muted-foreground mb-4"
      >
        <Film class="h-8 w-8" />
      </div>
      <h3 class="text-base font-semibold text-foreground">
        No videos in this playlist
      </h3>
      <p class="text-sm text-muted-foreground mt-1 max-w-sm">
        This playlist has no videos yet or they might be private.
      </p>
    </div>

    <!-- Video Rows -->
    <template v-else>
      <PlaylistVideoRow
        v-for="(video, idx) in videos"
        :key="`${video.id}-${idx}`"
        :video="video"
        :playlist-id="playlistId"
        :index-number="idx + 1"
        @click="emit('selectVideo', video, idx + 1)"
      />

      <!-- Infinite Scroll Trigger Element -->
      <div ref="bottomTarget" class="h-4 w-full" />

      <!-- Loading More Spinner -->
      <div
        v-if="isLoadingMore"
        class="flex items-center justify-center py-6 text-muted-foreground gap-2 text-sm"
      >
        <Loader2 class="h-4 w-4 animate-spin text-primary" />
        <span>Loading more videos...</span>
      </div>
    </template>
  </div>
</template>

