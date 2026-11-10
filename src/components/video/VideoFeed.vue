<script setup lang="ts">
import { ref, onMounted } from "vue";
import { commands, type FilterChip } from "@/bindings";
import type { FeedItem } from "@/components/video/VideoGrid.vue";

const props = defineProps<{
  categoryParams: string | null;
}>();

const emit = defineEmits<{
  (e: "update-chips", chips: FilterChip[]): void;
}>();

const isLoading = ref(false);
const continuation = ref<string | null>(null);
const items = ref<FeedItem[]>([]);
const bottomTarget = ref<HTMLElement | null>(null);

async function get_homefeed() {
  if (isLoading.value) return;
  try {
    isLoading.value = true;
    const res = await commands.getHomeFeed(
      continuation.value,
      props.categoryParams,
    );
    if (res.status === "ok") {
      const batchItems: FeedItem[] = [];

      if (res.data.playlists && res.data.playlists.length > 0) {
        for (const p of res.data.playlists) {
          batchItems.push({ kind: "playlist", data: p });
        }
      }

      if (res.data.videos && res.data.videos.length > 0) {
        for (const v of res.data.videos) {
          batchItems.push({ kind: "video", data: v });
        }
      }

      items.value.push(...batchItems);
      continuation.value = res.data.continuationToken ?? null;

      if (res.data.filterChips && res.data.filterChips.length > 0) {
        emit("update-chips", res.data.filterChips);
      }
    }
  } catch (_) {
  } finally {
    isLoading.value = false;
  }
}

onMounted(() => {
  get_homefeed();

  const observer = new IntersectionObserver(([entry]) => {
    if (entry.isIntersecting && continuation.value && !isLoading.value) {
      get_homefeed();
    }
  });

  if (bottomTarget.value) observer.observe(bottomTarget.value);
});
</script>

<template>
  <div class="flex-1 flex flex-col">
    <VideoGrid :items="items" :loading="isLoading" />
    <div ref="bottomTarget" class="h-10 w-full"></div>
  </div>
</template>
