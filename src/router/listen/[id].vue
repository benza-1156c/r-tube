<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { usePlayer } from "@/composables/usePlayer";
import { useAudioPlayer } from "@/composables/useAudioPlayer";
import { useSettings } from "@/composables/useSettings";
import { useVideoPage, resumeFromQuery } from "@/composables/useVideoPage";

import ListenArtwork from "@/components/listen/ListenArtwork.vue";
import ListenTrackInfo from "@/components/listen/ListenTrackInfo.vue";
import ListenControls from "@/components/listen/ListenControls.vue";
import ListenRelated from "@/components/listen/ListenRelated.vue";

import { ChevronDown, ChevronUp, AlertCircle, Clapperboard } from "@lucide/vue";

defineOptions({ name: "ListenPage" });

const route = useRoute();
const router = useRouter();
const player = usePlayer();
const audio = useAudioPlayer();
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

const {
  watchNext,
  loading: isLoadingRelated,
  load: loadListenData,
  selectRelated,
} = useVideoPage(videoId, playlistId, playlistIndex);

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

function startPlayback(id: string): void {
  const resumeAt = resumeFromQuery(route.query);
  if (!id) return;

  // Re-entering the same track (for example after a re-render) must not restart
  // it; anything else replaces the current audio source.
  const alreadyLoaded =
    player.activeEngine.value === "audio" &&
    player.currentVideo.value?.id === id &&
    player.status.value !== "error";
  if (!alreadyLoaded) {
    void audio.loadAudio(id, {
      resumeAt: resumeAt > 0 ? resumeAt : undefined,
      autoplay: true,
    });
  }

  // The queue and the related list are per-route, not per-engine.
  void loadListenData();
}

onMounted(() => {
  if (settings.value.playbackMode === "normal" && videoId.value) {
    void router.replace({
      path: `/watch/${videoId.value}`,
      query: route.query,
    });
    return;
  }
  if (videoId.value) {
    startPlayback(videoId.value);
  }
});

// React to route ID or playlist changes
watch(
  [() => videoId.value, () => route.query.list, () => route.query.index],
  ([newId]) => {
    if (newId) {
      startPlayback(newId);
    }
  },
);

function handleSeek(targetTime: number): void {
  player.seek(targetTime);
}

function handlePrevious(): void {
  const el = player.activeMedia.value;
  if (el && el.currentTime > 3) {
    player.seek(0);
    return;
  }

  // Go to previous track in playlist if available
  if (watchNext.value?.playlistItems && playlistId.value) {
    const curIdx = watchNext.value.playlistItems.findIndex(
      (p) => p.videoId === videoId.value || p.isSelected,
    );
    if (curIdx > 0) {
      const prevItem = watchNext.value.playlistItems[curIdx - 1];
      void router.push({
        path: `/listen/${prevItem.videoId}`,
        query: {
          list: playlistId.value,
          index: String(curIdx),
        },
      });
      return;
    }
  }

  player.seek(0);
}

function handleNext(): void {
  if (watchNext.value?.playlistItems && playlistId.value) {
    const curIdx = watchNext.value.playlistItems.findIndex(
      (p) => p.videoId === videoId.value || p.isSelected,
    );
    if (curIdx >= 0 && curIdx + 1 < watchNext.value.playlistItems.length) {
      const nextItem = watchNext.value.playlistItems[curIdx + 1];
      void router.push({
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

function handleVolumeChange(val: number): void {
  player.setVolume(val);
}

function toggleMute(): void {
  player.toggleMute();
}

/** A live stream or a broken audio URL still plays in video mode. */
function switchToVideo(): void {
  void updateSettings({ playbackMode: "normal" });
  void router.replace({
    path: `/watch/${videoId.value}`,
    query: route.query,
  });
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
          :is-muted="player.isMuted.value"
          @seek="handleSeek"
          @toggle-play="player.togglePlay"
          @previous="handlePrevious"
          @next="handleNext"
          @volume-change="handleVolumeChange"
          @toggle-mute="toggleMute"
        />

        <!-- Audio-only failure: live streams and unplayable releases still have video. -->
        <div
          v-if="player.status.value === 'error'"
          class="w-full rounded-2xl border border-destructive/30 bg-destructive/5 p-4 mt-6 text-sm"
        >
          <div class="flex items-start gap-3">
            <AlertCircle class="h-4 w-4 mt-0.5 shrink-0 text-destructive" />
            <div class="flex-1 min-w-0">
              <p class="font-semibold text-foreground">
                Audio-only playback is unavailable
              </p>
              <p class="text-xs text-muted-foreground mt-1 wrap-break-word">
                {{ player.errorMessage.value }}
              </p>
              <button
                type="button"
                class="mt-3 inline-flex items-center gap-1.5 rounded-full bg-foreground px-3 py-1.5 text-xs font-medium text-background cursor-pointer"
                @click="switchToVideo"
              >
                <Clapperboard class="h-3.5 w-3.5" />
                Watch video instead
              </button>
            </div>
          </div>
        </div>

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
