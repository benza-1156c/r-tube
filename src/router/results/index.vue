<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from "vue";
import { useRoute } from "vue-router";
import { commands, type VideoItem, type PlaylistItem } from "@/bindings";
import VideoGrid from "@/components/video/VideoGrid.vue";
import { Search } from "@lucide/vue";

defineOptions({
  name: "SearchResultsPage",
});

const route = useRoute();
const query = computed(() => {
  return (
    (route.query.search_query as string) ||
    (route.query.q as string) ||
    ""
  ).trim();
});

const isLoading = ref(false);
const isLoadingMore = ref(false);
const videos = ref<VideoItem[]>([]);
const playlists = ref<PlaylistItem[]>([]);
const continuation = ref<string | null>(null);
const bottomTarget = ref<HTMLElement | null>(null);
let observer: IntersectionObserver | null = null;

async function loadInitialSearch() {
  if (!query.value) {
    videos.value = [];
    playlists.value = [];
    continuation.value = null;
    return;
  }

  isLoading.value = true;
  continuation.value = null;
  try {
    const res = await commands.search(query.value, null);
    if (res.status === "ok") {
      videos.value = res.data.videos ?? [];
      playlists.value = res.data.playlists ?? [];
      continuation.value = res.data.continuationToken ?? null;
    }
  } catch (err) {
    console.error("Search failed:", err);
  } finally {
    isLoading.value = false;
  }
}

async function loadMoreSearch() {
  if (
    !query.value ||
    !continuation.value ||
    isLoadingMore.value ||
    isLoading.value
  ) {
    return;
  }

  isLoadingMore.value = true;
  try {
    const res = await commands.search(query.value, continuation.value);
    if (res.status === "ok") {
      if (res.data.videos && res.data.videos.length > 0) {
        videos.value.push(...res.data.videos);
      }
      if (res.data.playlists && res.data.playlists.length > 0) {
        playlists.value.push(...res.data.playlists);
      }
      continuation.value = res.data.continuationToken ?? null;
    }
  } catch (err) {
    console.error("Failed to load more search results:", err);
  } finally {
    isLoadingMore.value = false;
  }
}

onMounted(() => {
  loadInitialSearch();

  observer = new IntersectionObserver(([entry]) => {
    if (
      entry.isIntersecting &&
      continuation.value &&
      !isLoading.value &&
      !isLoadingMore.value
    ) {
      loadMoreSearch();
    }
  });

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

watch(query, (newQ, oldQ) => {
  if (newQ !== oldQ) {
    loadInitialSearch();
  }
});
</script>

<template>
  <div class="flex flex-col min-h-full p-2">
    <!-- Header info -->
    <div v-if="query" class="px-6 pt-4 pb-2">
      <h1 class="text-lg font-semibold text-foreground">
        Results for <span class="text-primary font-bold">"{{ query }}"</span>
      </h1>
    </div>

    <!-- Empty State -->
    <div
      v-if="!isLoading && videos.length === 0 && playlists.length === 0"
      class="flex flex-col items-center justify-center py-20 text-center"
    >
      <div
        class="h-14 w-14 rounded-full bg-muted flex items-center justify-center mb-4"
      >
        <Search class="h-7 w-7 text-muted-foreground" />
      </div>
      <p class="text-base font-medium text-foreground">No results found</p>
      <p class="text-sm text-muted-foreground mt-1">
        Try different keywords or check spelling
      </p>
    </div>

    <!-- Results Grid -->
    <VideoGrid :videos="videos" :playlists="playlists" :loading="isLoading" />

    <!-- Infinite scroll bottom target -->
    <div
      ref="bottomTarget"
      class="h-12 w-full flex items-center justify-center"
    >
      <div
        v-if="isLoadingMore"
        class="h-6 w-6 rounded-full border-2 border-primary border-t-transparent animate-spin"
      />
    </div>
  </div>
</template>
