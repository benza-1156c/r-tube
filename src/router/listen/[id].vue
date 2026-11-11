<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { usePlayer } from "@/composables/usePlayer";
import { useSettings } from "@/composables/useSettings";
import {
  commands,
  type VideoItem_Deserialize,
  type WatchNextResults_Deserialize,
} from "@/bindings";

import ListenArtwork from "@/components/listen/ListenArtwork.vue";
import ListenTrackInfo from "@/components/listen/ListenTrackInfo.vue";
import ListenControls from "@/components/listen/ListenControls.vue";
import ListenRelated from "@/components/listen/ListenRelated.vue";

import { ChevronDown, ChevronUp } from "@lucide/vue";

defineOptions({ name: "ListenPage" });

const route = useRoute();
const router = useRouter();
const player = usePlayer();
const { settings, updateSettings } = useSettings();

const videoId = computed(
  () => ((route.params as Record<string, any>)?.id as string) || "",
);

const playlistId = computed(() => {
  const q = route.query.list;
  return typeof q === "string" ? q.trim() : null;
});

const playlistIndex = computed(() => {
  const idx = route.query.index;
  if (typeof idx === "string") {
    const num = parseInt(idx, 10);
    return isNaN(num) ? null : num;
  }
  return null;
});

const watchNext = ref<WatchNextResults_Deserialize | null>(null);
const isLoadingRelated = ref(false);
const isDescriptionExpanded = ref(false);

const duration = computed(() => player.currentVideo.value?.duration || 0);
const currentTime = computed(() => player.currentTime.value);

const currentThumbnail = computed(() => {
  return (
    player.currentVideo.value?.thumbnailUrl ||
    watchNext.value?.playlistItems?.find((p) => p.videoId === videoId.value)
      ?.thumbnailUrl ||
    ""
  );
});

function applyAudioOnlyOptimization() {
  const el = player.videoElement.value as any;
  if (!el) return;
  const renditions = el.videoRenditions;
  if (renditions && renditions.length > 0) {
    // Select lowest video rendition to minimize bandwidth and memory footprint
    renditions.selectedIndex = 0;
  }
}

async function loadListenData(id: string) {
  if (!id) return;

  const playerAlreadyHasVideo =
    player.isOpen.value &&
    player.currentVideo.value?.id === id &&
    player.status.value !== "error" &&
    (player.status.value === "loading" || Boolean(player.currentSrc.value));

  if (!playerAlreadyHasVideo) {
    player.loadVideo(id);
  }

  // Fetch Watch Next results (related videos & playlist queue)
  isLoadingRelated.value = true;
  try {
    const res = await commands.getWatchNext(
      id,
      playlistId.value,
      playlistIndex.value,
    );
    if (res.status === "ok") {
      watchNext.value = res.data;

      // If in playlist / mix, queue up the next song
      if (res.data.playlistItems && res.data.playlistItems.length > 0) {
        const curIdx = res.data.playlistItems.findIndex(
          (p) => p.videoId === id || p.isSelected,
        );
        const nextItem =
          curIdx >= 0 && curIdx + 1 < res.data.playlistItems.length
            ? res.data.playlistItems[curIdx + 1]
            : null;
        if (nextItem) {
          player.setUpNextVideo({
            id: nextItem.videoId,
            title: nextItem.title,
            thumbnailUrl: nextItem.thumbnailUrl,
            duration: "",
            channelName: nextItem.author,
            channelAvatarUrl: "",
            isVerified: false,
            isLive: false,
            viewCount: "",
            publishedAt: "",
            playlistId: playlistId.value || undefined,
          });
        }
      } else if (res.data.relatedVideos && res.data.relatedVideos.length > 0) {
        player.setUpNextVideo(res.data.relatedVideos[0]);
      }
    }
  } catch (err) {
    console.warn("Failed to load listen details:", err);
  } finally {
    isLoadingRelated.value = false;
    applyAudioOnlyOptimization();
  }
}

onMounted(() => {
  // If settings indicate normal mode, redirect to watch page
  if (settings.value.playbackMode === "normal" && videoId.value) {
    router.replace({
      path: `/watch/${videoId.value}`,
      query: route.query,
    });
    return;
  }

  if (videoId.value) {
    loadListenData(videoId.value);
  }
});

// React to route ID or playlist changes
watch(
  [() => videoId.value, () => route.query.list, () => route.query.index],
  ([newId]) => {
    if (newId) {
      loadListenData(newId);
    }
  },
);

// If playbackMode changes to normal, redirect to watch
watch(
  () => settings.value.playbackMode,
  (mode) => {
    if (mode === "normal" && videoId.value) {
      router.replace({
        path: `/watch/${videoId.value}`,
        query: route.query,
      });
    }
  },
);

function handleSeek(targetTime: number) {
  if (player.videoElement.value) {
    player.videoElement.value.currentTime = targetTime;
  }
}

function handlePrevious() {
  const el = player.videoElement.value;
  if (el && el.currentTime > 3) {
    el.currentTime = 0;
    return;
  }

  // Go to previous track in playlist if available
  if (watchNext.value?.playlistItems && playlistId.value) {
    const curIdx = watchNext.value.playlistItems.findIndex(
      (p) => p.videoId === videoId.value || p.isSelected,
    );
    if (curIdx > 0) {
      const prevItem = watchNext.value.playlistItems[curIdx - 1];
      router.push({
        path: `/listen/${prevItem.videoId}`,
        query: {
          list: playlistId.value,
          index: String(curIdx),
        },
      });
      return;
    }
  }

  if (el) {
    el.currentTime = 0;
  }
}

function handleNext() {
  // Go to next track in playlist if available
  if (watchNext.value?.playlistItems && playlistId.value) {
    const curIdx = watchNext.value.playlistItems.findIndex(
      (p) => p.videoId === videoId.value || p.isSelected,
    );
    if (curIdx >= 0 && curIdx + 1 < watchNext.value.playlistItems.length) {
      const nextItem = watchNext.value.playlistItems[curIdx + 1];
      router.push({
        path: `/listen/${nextItem.videoId}`,
        query: {
          list: playlistId.value,
          index: String(curIdx + 2),
        },
      });
      return;
    }
  }

  player.playNext();
}

function handleVolumeChange(val: number) {
  if (player.videoElement.value) {
    player.videoElement.value.volume = val;
  }
  void updateSettings({ volume: val });
}

function toggleMute() {
  const el = player.videoElement.value;
  if (!el) return;
  if (el.muted || el.volume === 0) {
    el.muted = false;
    el.volume = settings.value.volume || 0.75;
  } else {
    el.muted = true;
  }
}

function selectRelated(item: VideoItem_Deserialize) {
  if (item.playlistId) {
    router.push({
      path: `/listen/${item.id}`,
      query: { list: item.playlistId },
    });
  } else {
    router.push(`/listen/${item.id}`);
  }
}
</script>

<template>
  <div class="relative min-h-[calc(100vh-6rem)] p-6 max-w-7xl mx-auto">
    <!-- Ambient blurred background from thumbnail -->
    <div
      v-if="currentThumbnail"
      class="absolute inset-0 bg-cover bg-center blur-3xl opacity-20 pointer-events-none -z-10 scale-110 transition-all duration-700"
      :style="{ backgroundImage: `url(${currentThumbnail})` }"
    />

    <!-- Main Layout: Player Column + Queue/Related Column -->
    <div class="flex flex-col lg:flex-row gap-8">
      <!-- Left Column: Audio Artwork & Controls -->
      <div class="flex-1 flex flex-col items-center max-w-2xl mx-auto w-full">
        <!-- Album Artwork Cover -->
        <ListenArtwork
          :thumbnail-url="currentThumbnail"
          :title="player.currentVideo.value?.title"
          :is-playing="player.isPlaying.value"
        />

        <!-- Track Info & Actions (All in English) -->
        <ListenTrackInfo
          :title="
            (player.currentVideo.value?.title || watchNext?.currentTitle) ??
            undefined
          "
          :author="
            (player.currentVideo.value?.channelName ||
              watchNext?.currentAuthor) ??
            undefined
          "
          :avatar-url="player.currentVideo.value?.channelAvatarUrl"
          :is-verified="false"
        />

        <!-- Audio Scrubber & Playback Controls -->
        <ListenControls
          :current-time="currentTime"
          :duration="duration"
          :is-playing="player.isPlaying.value"
          :is-loading="
            player.status.value === 'loading' ||
            player.status.value === 'buffering'
          "
          :volume="settings.volume ?? 1"
          :is-muted="Boolean(player.videoElement.value?.muted)"
          @seek="handleSeek"
          @toggle-play="player.togglePlay"
          @previous="handlePrevious"
          @next="handleNext"
          @volume-change="handleVolumeChange"
          @toggle-mute="toggleMute"
        />

        <!-- Description Box (English Labels) -->
        <div
          v-if="player.currentVideo.value?.description"
          class="w-full rounded-2xl bg-muted/30 border border-border/40 p-4 mt-8 text-xs leading-relaxed text-foreground/80 transition-all select-none"
        >
          <div
            v-if="player.currentVideo.value?.viewCount"
            class="font-semibold mb-2 text-muted-foreground"
          >
            <span>{{ player.currentVideo.value.viewCount }}</span>
          </div>

          <p :class="{ 'line-clamp-3': !isDescriptionExpanded }">
            {{ player.currentVideo.value.description }}
          </p>

          <button
            v-if="player.currentVideo.value.description.length > 150"
            type="button"
            class="mt-2 font-semibold text-foreground hover:text-primary flex items-center gap-1 transition-colors cursor-pointer"
            @click="isDescriptionExpanded = !isDescriptionExpanded"
          >
            <span>{{ isDescriptionExpanded ? "Show less" : "Show more" }}</span>
            <ChevronUp v-if="isDescriptionExpanded" class="h-3.5 w-3.5" />
            <ChevronDown v-else class="h-3.5 w-3.5" />
          </button>
        </div>
      </div>

      <!-- Right Column: Playlist / Mix Queue & Up Next -->
      <div class="w-full lg:w-96 flex flex-col gap-3 shrink-0">
        <!-- Playlist Queue Panel (if playing inside a Mix / Playlist) -->
        <WatchPlaylistQueue
          v-if="
            watchNext?.playlistItems &&
            watchNext.playlistItems.length > 0 &&
            playlistId
          "
          :playlist-id="playlistId"
          :items="watchNext.playlistItems"
          :current-video-id="videoId"
          :current-title="watchNext.currentTitle"
        />

        <!-- Related Tracks (English Labels) -->
        <ListenRelated
          :items="watchNext?.relatedVideos || []"
          :is-loading="isLoadingRelated"
          @select="selectRelated"
        />
      </div>
    </div>
  </div>
</template>
