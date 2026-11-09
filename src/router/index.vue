<script setup lang="ts">
import { commands } from "@/bindings";
import type { VideoItem } from "@/components/video/VideoCard.vue";

const isLoading = ref(false);
const continuation = ref<string | null>(null);
const video = ref<VideoItem[] | null>(null);

async function get_homefeed() {
  try {
    isLoading.value = true;
    const res = await commands.getHomeFeed(continuation.value);
    if (res.status === "ok") {
      video.value = res.data.videos;
      console.log(res.data);
      continuation.value = res.data.continuationToken;
    }
  } catch (_) {
  } finally {
    isLoading.value = false;
  }
}

onMounted(() => {
  get_homefeed();
});

function onCategorySelect(category: string) {
  console.log("Selected category:", category);
}
</script>

<template>
  <div class="flex flex-col min-h-full">
    <!-- Category Filter Chips -->
    <CategoryChips @select="onCategorySelect" />

    <!-- Video Grid Section -->
    <!-- <VideoGrid :videos="mockVideos" :loading="isLoading" /> -->
  </div>
</template>
