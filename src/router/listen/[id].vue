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
import WatchPlaylistQueue from "@/components/playlist/WatchPlaylistQueue.vue";
import {
  Play,
  Pause,
  SkipBack,
  SkipForward,
  Volume2,
  VolumeX,
  Volume1,
  AudioLines,
  Clapperboard,
  ThumbsUp,
  Share2,
  Bookmark,
  CheckCircle2,
  Loader2,
  ListMusic,
  ChevronDown,
  ChevronUp,
} from "@lucide/vue";

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

const isSeeking = ref(false);
const seekPosition = ref(0);

const duration = computed(() => player.currentVideo.value?.duration || 0);
const currentTime = computed(() =>
  isSeeking.value ? seekPosition.value : player.currentTime.value,
);

const currentThumbnail = computed(() => {
  return (
    player.currentVideo.value?.thumbnailUrl ||
    watchNext.value?.playlistItems?.find((p) => p.videoId === videoId.value)
      ?.thumbnailUrl ||
    ""
  );
});

function formatTime(seconds: number): string {
  if (!seconds || isNaN(seconds) || seconds < 0) return "0:00";
  const sec = Math.floor(seconds);
  const m = Math.floor(sec / 60);
  const s = sec % 60;
  if (m >= 60) {
    const h = Math.floor(m / 60);
    const remM = m % 60;
    return `${h}:${remM.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
  }
  return `${m}:${s.toString().padStart(2, "0")}`;
}

async function loadListenData(id: string) {
  if (!id) return;

  // Make sure player is loading/playing this video
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
  }
}

onMounted(() => {
  // If settings indicate normal mode, route to watch page instead
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

function switchToVideo() {
  void updateSettings({ playbackMode: "normal" });
  router.push({
    path: `/watch/${videoId.value}`,
    query: route.query,
  });
}

function handleSeekInput(e: Event) {
  isSeeking.value = true;
  seekPosition.value = Number((e.target as HTMLInputElement).value);
}

function handleSeekChange(e: Event) {
  const targetTime = Number((e.target as HTMLInputElement).value);
  if (player.videoElement.value) {
    player.videoElement.value.currentTime = targetTime;
  }
  seekPosition.value = targetTime;
  isSeeking.value = false;
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

function handleVolumeChange(e: Event) {
  const val = Number((e.target as HTMLInputElement).value);
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

    <!-- Main Layout: Player Center + Queue/Related on Right -->
    <div class="flex flex-col lg:flex-row gap-8">
      <!-- Left Column: Audio Artwork & Controls -->
      <div class="flex-1 flex flex-col items-center max-w-2xl mx-auto w-full">
        <!-- Top Mode Banner -->
        <div class="w-full flex items-center justify-between mb-4">
          <div
            class="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-primary/10 border border-primary/20 text-primary text-xs font-semibold select-none"
          >
            <AudioLines class="h-3.5 w-3.5" />
            <span>โหมดฟังเพลงเท่านั้น</span>
          </div>

          <button
            type="button"
            class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-muted hover:bg-muted/80 text-foreground text-xs font-medium border border-border/50 transition-colors cursor-pointer select-none"
            @click="switchToVideo"
          >
            <Clapperboard class="h-3.5 w-3.5 text-muted-foreground" />
            <span>สลับไปดูวิดีโอ</span>
          </button>
        </div>

        <!-- Album Artwork (Square Cover with glow) -->
        <div class="relative w-64 h-64 sm:w-80 sm:h-80 md:w-96 md:h-96 my-4">
          <div
            v-if="currentThumbnail"
            class="absolute inset-0 rounded-3xl bg-cover bg-center blur-2xl opacity-40 scale-95 -z-10"
            :style="{ backgroundImage: `url(${currentThumbnail})` }"
          />

          <img
            v-if="currentThumbnail"
            :src="currentThumbnail"
            :alt="player.currentVideo.value?.title || 'Track Cover'"
            class="w-full h-full object-cover rounded-3xl shadow-2xl ring-1 ring-border/50 select-none"
            :class="{
              'scale-[1.01] transition-transform duration-500':
                player.isPlaying.value,
            }"
          />
          <div
            v-else
            class="w-full h-full rounded-3xl bg-muted flex items-center justify-center shadow-xl ring-1 ring-border/50"
          >
            <ListMusic class="h-16 w-16 text-muted-foreground/50" />
          </div>
        </div>

        <!-- Track Info -->
        <div class="w-full text-center mt-4">
          <h1
            class="text-xl sm:text-2xl font-bold text-foreground leading-snug line-clamp-2"
            :title="
              player.currentVideo.value?.title ||
              watchNext?.currentTitle ||
              'Loading...'
            "
          >
            {{
              player.currentVideo.value?.title ||
              watchNext?.currentTitle ||
              "Loading audio..."
            }}
          </h1>

          <div
            class="flex items-center justify-center gap-2 mt-2 text-sm text-muted-foreground"
          >
            <span class="font-medium hover:text-foreground transition-colors">{{
              player.currentVideo.value?.channelName ||
              watchNext?.currentAuthor ||
              "Artist"
            }}</span>
            <CheckCircle2
              class="h-3.5 w-3.5 text-muted-foreground fill-muted-foreground/30"
            />
          </div>
        </div>

        <!-- Action Buttons (Like / Share / Save) -->
        <div class="flex items-center justify-center gap-2 mt-4 select-none">
          <button
            type="button"
            class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-muted/60 hover:bg-muted text-xs font-medium text-foreground transition-colors cursor-pointer"
          >
            <ThumbsUp class="h-3.5 w-3.5" />
            <span>ถูกใจ</span>
          </button>
          <button
            type="button"
            class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-muted/60 hover:bg-muted text-xs font-medium text-foreground transition-colors cursor-pointer"
          >
            <Share2 class="h-3.5 w-3.5" />
            <span>แชร์</span>
          </button>
          <button
            type="button"
            class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-muted/60 hover:bg-muted text-xs font-medium text-foreground transition-colors cursor-pointer"
          >
            <Bookmark class="h-3.5 w-3.5" />
            <span>บันทึก</span>
          </button>
        </div>

        <!-- Scrubber Progress Bar -->
        <div class="w-full mt-6 px-2 select-none">
          <div class="relative flex items-center group">
            <input
              type="range"
              min="0"
              :max="duration || 100"
              :value="currentTime"
              step="0.5"
              class="w-full h-1.5 bg-muted rounded-lg appearance-none cursor-pointer accent-primary group-hover:h-2 transition-all"
              @input="handleSeekInput"
              @change="handleSeekChange"
            />
          </div>

          <div
            class="flex justify-between text-xs text-muted-foreground font-mono mt-1.5"
          >
            <span>{{ formatTime(currentTime) }}</span>
            <span>{{ formatTime(duration) }}</span>
          </div>
        </div>

        <!-- Audio Playback Controls -->
        <div
          class="flex items-center justify-center gap-6 mt-4 select-none w-full"
        >
          <!-- Previous Track -->
          <button
            type="button"
            class="p-2.5 rounded-full text-foreground/80 hover:text-foreground hover:bg-muted/60 transition-colors cursor-pointer"
            title="Previous"
            @click="handlePrevious"
          >
            <SkipBack class="h-5 w-5" />
          </button>

          <!-- Play / Pause Button -->
          <button
            type="button"
            class="h-14 w-14 rounded-full bg-primary text-primary-foreground flex items-center justify-center hover:scale-105 active:scale-95 transition-transform shadow-lg cursor-pointer"
            title="Play / Pause"
            @click="player.togglePlay"
          >
            <Loader2
              v-if="
                player.status.value === 'loading' ||
                player.status.value === 'buffering'
              "
              class="h-6 w-6 animate-spin text-primary-foreground"
            />
            <Pause
              v-else-if="player.isPlaying.value"
              class="h-6 w-6 fill-current"
            />
            <Play v-else class="h-6 w-6 fill-current translate-x-0.5" />
          </button>

          <!-- Next Track -->
          <button
            type="button"
            class="p-2.5 rounded-full text-foreground/80 hover:text-foreground hover:bg-muted/60 transition-colors cursor-pointer"
            title="Next"
            @click="handleNext"
          >
            <SkipForward class="h-5 w-5" />
          </button>
        </div>

        <!-- Volume Control Row -->
        <div class="flex items-center gap-3 mt-4 w-48 text-muted-foreground">
          <button
            type="button"
            class="p-1 hover:text-foreground transition-colors cursor-pointer"
            @click="toggleMute"
          >
            <VolumeX
              v-if="
                player.videoElement.value?.muted ||
                (settings.volume ?? 1) === 0
              "
              class="h-4 w-4"
            />
            <Volume1
              v-else-if="(settings.volume ?? 1) < 0.5"
              class="h-4 w-4"
            />
            <Volume2 v-else class="h-4 w-4" />
          </button>

          <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            :value="settings.volume ?? 1"
            class="w-full h-1 bg-muted rounded-lg appearance-none cursor-pointer accent-primary"
            @input="handleVolumeChange"
          />
        </div>

        <!-- Description Box -->
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

        <div class="flex items-center justify-between">
          <h2 class="font-semibold text-sm text-foreground">เพลงถัดไป</h2>
          <Loader2
            v-if="isLoadingRelated"
            class="h-4 w-4 text-muted-foreground animate-spin"
          />
        </div>

        <!-- Related Tracks List -->
        <div class="flex flex-col gap-2.5">
          <div
            v-for="item in watchNext?.relatedVideos || []"
            :key="item.id"
            class="flex gap-3 group cursor-pointer hover:bg-muted/40 p-2 rounded-xl transition-colors select-none"
            @click="selectRelated(item)"
          >
            <!-- Thumbnail -->
            <div
              class="relative aspect-video w-32 shrink-0 overflow-hidden rounded-lg bg-muted shadow-2xs"
            >
              <img
                :src="item.thumbnailUrl"
                :alt="item.title"
                class="h-full w-full object-cover group-hover:scale-105 transition-transform"
                loading="lazy"
              />
              <span
                v-if="item.duration"
                class="absolute bottom-1 right-1 rounded-sm bg-black/80 px-1 text-[10px] text-white flex items-center gap-0.5"
              >
                <ListMusic
                  v-if="item.playlistId || item.duration === 'Mix'"
                  class="h-2.5 w-2.5"
                />
                {{ item.duration }}
              </span>
            </div>

            <!-- Info -->
            <div class="flex flex-col min-w-0 flex-1 justify-center">
              <h3
                class="line-clamp-2 text-xs font-semibold leading-snug text-foreground group-hover:text-primary transition-colors"
                :title="item.title"
              >
                {{ item.title }}
              </h3>
              <p class="text-[11px] text-muted-foreground mt-1 truncate">
                {{ item.channelName }}
              </p>
              <div
                v-if="item.viewCount || item.publishedAt"
                class="text-[10px] text-muted-foreground/80 flex items-center gap-1 mt-0.5"
              >
                <span v-if="item.viewCount">{{ item.viewCount }}</span>
                <span v-if="item.viewCount && item.publishedAt">•</span>
                <span v-if="item.publishedAt">{{ item.publishedAt }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

