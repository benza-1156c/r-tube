<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { AlertCircle, RefreshCw } from "@lucide/vue";
import { commands, type PlaylistDetailView } from "@/bindings";
import PlaylistSidebar from "@/components/playlist/PlaylistSidebar.vue";
import PlaylistVideoList from "@/components/playlist/PlaylistVideoList.vue";

const route = useRoute();
const router = useRouter();

const playlistId = computed(() => {
  const q = route.query.list ?? route.query.id;
  return typeof q === "string" ? q.trim() : "";
});

const playlist = ref<PlaylistDetailView | null>(null);
const isLoading = ref(true);
const isLoadingMore = ref(false);
const error = ref<string | null>(null);

async function fetchPlaylist(id: string) {
  if (!id) {
    error.value = "No playlist ID provided";
    isLoading.value = false;
    return;
  }

  isLoading.value = true;
  error.value = null;

  try {
    const res = await commands.getPlaylist(id);
    if (res.status === "ok") {
      playlist.value = res.data;
    } else {
      error.value = res.error?.message || "Failed to load playlist";
    }
  } catch (err: unknown) {
    error.value = err instanceof Error ? err.message : String(err);
  } finally {
    isLoading.value = false;
  }
}

async function handleLoadMore() {
  if (
    !playlist.value ||
    !playlist.value.continuationToken ||
    isLoadingMore.value ||
    isLoading.value
  ) {
    return;
  }

  isLoadingMore.value = true;
  try {
    const token = playlist.value.continuationToken;
    const res = await commands.getPlaylistContinuation(token);
    if (res.status === "ok") {
      if (res.data.videos && res.data.videos.length > 0) {
        playlist.value.videos.push(...res.data.videos);
      }
      playlist.value.continuationToken = res.data.continuationToken ?? null;
    }
  } catch (err) {
    console.error("Failed to load more playlist videos:", err);
  } finally {
    isLoadingMore.value = false;
  }
}

function handlePlayAll() {
  if (!playlist.value || playlist.value.videos.length === 0) return;
  const firstPlayable = playlist.value.videos.find((v) => v.isPlayable) || playlist.value.videos[0];
  if (!firstPlayable) return;

  router.push({
    path: `/watch/${firstPlayable.id}`,
    query: {
      list: playlist.value.id,
      index: "1",
    },
  });
}

function handleShuffle() {
  if (!playlist.value || playlist.value.videos.length === 0) return;
  const playableVideos = playlist.value.videos.filter((v) => v.isPlayable);
  const targetVideos = playableVideos.length > 0 ? playableVideos : playlist.value.videos;
  const randomIndex = Math.floor(Math.random() * targetVideos.length);
  const randomVideo = targetVideos[randomIndex];
  if (!randomVideo) return;

  router.push({
    path: `/watch/${randomVideo.id}`,
    query: {
      list: playlist.value.id,
      index: String(randomIndex + 1),
    },
  });
}

watch(
  playlistId,
  (newId) => {
    if (newId) {
      fetchPlaylist(newId);
    }
  },
  { immediate: true },
);
</script>

<template>
  <div class="min-h-full w-full">
    <!-- Error View -->
    <div
      v-if="error && !isLoading"
      class="max-w-md mx-auto my-24 p-6 rounded-2xl border border-destructive/30 bg-destructive/10 text-center flex flex-col items-center gap-4"
    >
      <AlertCircle class="h-10 w-10 text-destructive" />
      <div>
        <h2 class="text-base font-semibold text-foreground">
          Cannot load playlist
        </h2>
        <p class="text-xs text-muted-foreground mt-1">{{ error }}</p>
      </div>
      <button
        type="button"
        class="inline-flex items-center gap-2 px-4 py-2 rounded-xl bg-foreground text-background font-medium text-xs hover:bg-foreground/90 transition-all cursor-pointer"
        @click="fetchPlaylist(playlistId)"
      >
        <RefreshCw class="h-3.5 w-3.5" />
        <span>Try again</span>
      </button>
    </div>

    <!-- Main Content -->
    <div
      v-else-if="playlist || isLoading"
      class="max-w-7xl mx-auto px-4 sm:px-6 py-6 flex flex-col lg:flex-row gap-8 items-start"
    >
      <!-- Left Sidebar with Summary & Controls -->
      <PlaylistSidebar
        v-if="playlist"
        :playlist="playlist"
        @play-all="handlePlayAll"
        @shuffle="handleShuffle"
      />
      <!-- Loading Skeleton for Sidebar if not loaded yet -->
      <div
        v-else
        class="w-full lg:w-88 xl:w-96 p-5 rounded-2xl border border-border/40 bg-card/40 animate-pulse flex flex-col gap-4 shrink-0"
      >
        <div class="aspect-video w-full rounded-xl bg-muted/60" />
        <div class="h-6 w-3/4 rounded bg-muted/60" />
        <div class="h-4 w-1/2 rounded bg-muted/40" />
        <div class="flex gap-2 mt-2">
          <div class="h-10 flex-1 rounded-full bg-muted/60" />
          <div class="h-10 flex-1 rounded-full bg-muted/40" />
        </div>
      </div>

      <!-- Right Column Video List -->
      <PlaylistVideoList
        :videos="playlist?.videos ?? []"
        :playlist-id="playlist?.id ?? playlistId"
        :is-loading="isLoading"
        :is-loading-more="isLoadingMore"
        :has-more="Boolean(playlist?.continuationToken)"
        @load-more="handleLoadMore"
      />
    </div>
  </div>
</template>

