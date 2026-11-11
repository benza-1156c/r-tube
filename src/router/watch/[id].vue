<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useRoute, useRouter, onBeforeRouteLeave } from "vue-router";
import { usePlayer } from "@/composables/usePlayer";
import { useAudioPlayer } from "@/composables/useAudioPlayer";
import { useSettings } from "@/composables/useSettings";
import { useVideoPage, resumeFromQuery } from "@/composables/useVideoPage";
import WatchPlaylistQueue from "@/components/playlist/WatchPlaylistQueue.vue";
import {
  ThumbsUp,
  Share2,
  Bookmark,
  CheckCircle2,
  Loader2,
  ListMusic,
  ChevronDown,
  ChevronUp,
} from "@lucide/vue";

defineOptions({ name: "WatchPage" });

const route = useRoute();
const router = useRouter();
const player = usePlayer();
const audio = useAudioPlayer();
const { settings } = useSettings();

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
  load: loadWatchNext,
  selectRelated: selectRelatedVideo,
} = useVideoPage(videoId, playlistId, playlistIndex);

const isDescriptionExpanded = ref(false);
const watchSlotRef = ref<HTMLDivElement | null>(null);

function startPlayback(id: string): void {
  if (!id) return;

  // The player may already be showing this video (returning from the mini
  // player): reloading it would drop the position for no reason.
  const alreadyLoaded =
    player.activeEngine.value === "video" &&
    player.currentVideo.value?.id === id &&
    player.status.value !== "error" &&
    (player.status.value === "loading" || Boolean(player.currentSrc.value));

  if (!alreadyLoaded) {
    // The audio engine may still be holding the previous track: hand the
    // session back to the video engine before loading.
    audio.stopAudio();
    void player.loadVideo(id, {
      resumeAt: resumeFromQuery(route.query),
      autoplay: true,
    });
  }

  void loadWatchNext();
}

onMounted(() => {
  if (settings.value.playbackMode !== "normal" && videoId.value) {
    void router.replace({
      path: `/listen/${videoId.value}`,
      query: route.query,
    });
    return;
  }
  if (watchSlotRef.value) {
    player.setWatchAnchor(watchSlotRef.value);
  }
  if (videoId.value) {
    startPlayback(videoId.value);
  }
});

onBeforeRouteLeave(() => {
  player.setWatchAnchor(null);
});

onUnmounted(() => {
  player.setWatchAnchor(null);
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
</script>

<template>
  <div class="flex flex-col lg:flex-row gap-6 p-6 max-w-7xl mx-auto">
    <!-- Left Column: Video Player & Metadata -->
    <div class="flex-1 flex flex-col gap-4 min-w-0">
      <!-- VIDEO PLAYER ANCHOR SLOT (GlobalVideoPlayer docks here) -->
      <div
        id="watch-player-slot"
        ref="watchSlotRef"
        class="relative aspect-video w-full overflow-hidden rounded-(--rt-player-radius)"
      >
        <!-- Fallback loader before player teleports -->
        <div
          v-if="player.status.value === 'loading'"
          class="w-full h-full flex flex-col items-center justify-center gap-3 text-white"
        >
          <Loader2 class="h-10 w-10 text-primary animate-spin" />
          <p class="text-xs text-white/70">Loading stream...</p>
        </div>
      </div>

      <!-- Video Title -->
      <h1 class="text-xl font-bold text-foreground leading-snug">
        {{
          player.currentVideo.value?.title ||
          watchNext?.currentTitle ||
          "Loading video..."
        }}
      </h1>

      <!-- Channel & Action Buttons Row -->
      <div
        class="flex flex-wrap items-center justify-between gap-4 border-b border-border/40 pb-4"
      >
        <!-- Channel Info -->
        <div class="flex items-center gap-3">
          <img
            v-if="player.currentVideo.value?.channelAvatarUrl"
            :src="player.currentVideo.value.channelAvatarUrl"
            :alt="player.currentVideo.value?.channelName"
            class="h-10 w-10 rounded-full object-cover ring-1 ring-border"
          />
          <div
            v-else
            class="h-10 w-10 rounded-full bg-muted flex items-center justify-center font-bold text-sm text-muted-foreground ring-1 ring-border uppercase"
          >
            {{
              (
                player.currentVideo.value?.channelName ||
                watchNext?.currentAuthor ||
                "Y"
              ).charAt(0)
            }}
          </div>

          <div>
            <div
              class="flex items-center gap-1 font-semibold text-sm text-foreground"
            >
              <span>{{
                player.currentVideo.value?.channelName ||
                watchNext?.currentAuthor ||
                "Channel"
              }}</span>
              <CheckCircle2
                class="h-3.5 w-3.5 text-muted-foreground fill-muted-foreground/30"
              />
            </div>
            <p
              v-if="player.currentVideo.value?.viewCount"
              class="text-xs text-muted-foreground"
            >
              {{ player.currentVideo.value.viewCount }}
            </p>
          </div>

          <Button
            class="ml-3 rounded-full bg-foreground text-background hover:bg-foreground/90 font-medium px-4 h-9 text-xs"
          >
            Subscribe
          </Button>
        </div>

        <!-- Action Buttons -->
        <div class="flex items-center gap-2">
          <Button
            variant="secondary"
            size="sm"
            class="rounded-full gap-1.5 h-9"
          >
            <ThumbsUp class="h-4 w-4" />
            <span>Like</span>
          </Button>
          <Button
            variant="secondary"
            size="sm"
            class="rounded-full gap-1.5 h-9"
          >
            <Share2 class="h-4 w-4" />
            <span>Share</span>
          </Button>
          <Button
            variant="secondary"
            size="sm"
            class="rounded-full gap-1.5 h-9"
          >
            <Bookmark class="h-4 w-4" />
            <span>Save</span>
          </Button>
        </div>
      </div>

      <!-- Description Box -->
      <div
        class="rounded-xl bg-muted/40 p-4 text-sm leading-relaxed text-foreground/90 transition-all"
      >
        <div
          v-if="player.currentVideo.value?.viewCount"
          class="font-semibold mb-2 flex items-center gap-2 text-xs text-muted-foreground"
        >
          <span>{{ player.currentVideo.value.viewCount }}</span>
        </div>

        <p
          class="whitespace-pre-line text-xs text-foreground/80"
          :class="{ 'line-clamp-3': !isDescriptionExpanded }"
        >
          {{
            player.currentVideo.value?.description || "No description provided."
          }}
        </p>

        <button
          v-if="(player.currentVideo.value?.description?.length || 0) > 150"
          class="mt-2 text-xs font-semibold text-foreground hover:text-primary flex items-center gap-1 transition-colors"
          @click="isDescriptionExpanded = !isDescriptionExpanded"
        >
          <span>{{ isDescriptionExpanded ? "Show less" : "Show more" }}</span>
          <ChevronUp v-if="isDescriptionExpanded" class="h-3.5 w-3.5" />
          <ChevronDown v-else class="h-3.5 w-3.5" />
        </button>
      </div>
    </div>

    <!-- Right Column: Up Next / Related Videos & Playlist Queue -->
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
        <h2 class="font-semibold text-sm text-foreground">Up next</h2>
        <Loader2
          v-if="isLoadingRelated"
          class="h-4 w-4 text-muted-foreground animate-spin"
        />
      </div>

      <!-- Related Videos List -->
      <div class="flex flex-col gap-2.5">
        <div
          v-for="item in watchNext?.relatedVideos || []"
          :key="item.id"
          class="flex gap-3 group cursor-pointer hover:bg-muted/40 p-2 rounded-xl transition-colors"
          @click="selectRelatedVideo(item)"
        >
          <!-- Thumbnail -->
          <div
            class="relative aspect-video w-36 shrink-0 overflow-hidden rounded-lg bg-muted"
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
              <span v-if="item.viewCount && item.publishedAt">â€¢</span>
              <span v-if="item.publishedAt">{{ item.publishedAt }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
