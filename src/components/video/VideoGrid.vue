<script setup lang="ts">
import { computed } from "vue";
import { VideoItem, PlaylistItem } from "@/bindings";
import VideoCard from "@/components/video/VideoCard.vue";
import PlaylistCard from "@/components/video/PlaylistCard.vue";

export type FeedItem =
  | { kind: "video"; data: VideoItem }
  | { kind: "playlist"; data: PlaylistItem };

const props = defineProps<{
  items?: FeedItem[];
  videos?: VideoItem[];
  playlists?: PlaylistItem[];
  loading?: boolean;
}>();

const displayItems = computed<FeedItem[]>(() => {
  if (props.items && props.items.length > 0) {
    return props.items;
  }
  const result: FeedItem[] = [];
  if (props.playlists) {
    for (const p of props.playlists) {
      result.push({ kind: "playlist", data: p });
    }
  }
  if (props.videos) {
    for (const v of props.videos) {
      result.push({ kind: "video", data: v });
    }
  }
  return result;
});
</script>

<template>
  <div
    class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-x-4 gap-y-7 p-6"
  >
    <template
      v-for="item in displayItems"
      :key="item.kind === 'video' ? `v-${item.data.id}` : `p-${item.data.id}`"
    >
      <PlaylistCard
        v-if="item.kind === 'playlist'"
        :playlist="item.data"
      />
      <VideoCard
        v-else-if="item.kind === 'video'"
        :video="item.data"
      />
    </template>

    <!-- Skeleton Loaders -->
    <template v-if="loading">
      <div v-for="i in 8" :key="`skeleton-${i}`" class="flex flex-col gap-3">
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
  </div>
</template>
