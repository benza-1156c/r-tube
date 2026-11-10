<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import { useRouter } from "vue-router";
import { usePlayer, type QualityOption } from "@/composables/usePlayer";
import "@videojs/html/video/player";
import "@videojs/html/video/skin";
import {
  Play,
  Pause,
  Settings,
  Check,
  ExternalLink,
  Minimize2,
  X,
} from "@lucide/vue";

defineOptions({ name: "GlobalVideoPlayer" });

const router = useRouter();
const player = usePlayer();

const videoRef = ref<HTMLVideoElement | null>(null);
const audioRef = ref<HTMLAudioElement | null>(null);
const showQualityMenu = ref(false);

function syncRefs(): void {
  if (videoRef.value) {
    player.attachVideo(videoRef.value);
  }
  if (audioRef.value) {
    player.attachAudio(audioRef.value);
  }
}

onMounted(() => {
  syncRefs();
  window.addEventListener("resize", player.updateAnchorRect);
});

onUnmounted(() => {
  window.removeEventListener("resize", player.updateAnchorRect);
});

watch(
  () => videoRef.value,
  () => {
    syncRefs();
  },
);

watch(
  () => audioRef.value,
  () => {
    syncRefs();
  },
);

function expandToWatch(): void {
  player.restoreFromMiniPlayer();
  if (player.currentVideo.value?.id) {
    router.push(`/watch/${player.currentVideo.value.id}`);
  }
}

function onSelectQuality(q: QualityOption): void {
  player.setQuality(q);
  showQualityMenu.value = false;
}

const isMini = computed(
  () => player.isMiniPlayer.value || !player.anchorRect.value,
);

const playerStyle = computed(() => {
  if (isMini.value) {
    return {
      position: "fixed" as const,
      bottom: "1.5rem",
      right: "1.5rem",
      width: "24rem",
      zIndex: 50,
    };
  }

  const rect = player.anchorRect.value!;
  return {
    position: "fixed" as const,
    top: `${rect.top}px`,
    left: `${rect.left}px`,
    width: `${rect.width}px`,
    height: `${rect.height}px`,
    zIndex: 30,
  };
});
</script>

<template>
  <div
    v-show="player.isOpen.value"
    id="global-player-root"
    class="select-none overflow-hidden transition-[top,left,width,height] duration-200"
    :class="[
      isMini
        ? 'rounded-2xl shadow-2xl border border-border/70 bg-card/95 backdrop-blur-md overflow-hidden'
        : 'rounded-2xl shadow-xl bg-black overflow-hidden',
    ]"
    :style="playerStyle"
    @click="showQualityMenu = false"
  >
    <!-- VIDEO FRAME -->
    <div
      class="relative w-full aspect-video bg-black flex items-center justify-center overflow-hidden"
      :class="{ 'mini-player-skin': isMini }"
    >
      <video-player
        :content-title="player.currentVideo.value?.title"
        :poster="player.currentVideo.value?.thumbnailUrl"
        class="w-full h-full block"
      >
        <video-skin
          style="
            width: 100%;
            height: 100%;
            aspect-ratio: 16 / 9;
            display: block;
          "
        >
          <video
            ref="videoRef"
            :src="player.currentSrc.value || undefined"
            referrerpolicy="no-referrer"
            playsinline
            autoplay
            @play="player.onPlay"
            @pause="player.onPause"
            @timeupdate="player.onTimeUpdate"
            @durationchange="player.onDurationChange"
            @volumechange="player.onVolumeChange"
            @waiting="player.onWaiting"
            @playing="player.onPlaying"
            @ended="player.onEnded"
            @error="player.onError"
          >
            <track
              v-for="cap in player.captionTracks.value"
              :key="cap.id"
              kind="captions"
              :src="cap.url"
              :srclang="cap.language"
              :label="cap.name"
            />
          </video>

          <!-- Hidden audio element for separate audio stream in adaptive 1080p+ -->
          <audio
            ref="audioRef"
            :src="player.currentAudioSrc.value || undefined"
            referrerpolicy="no-referrer"
            class="hidden"
          />
        </video-skin>
      </video-player>

      <!-- TOP-RIGHT CONTROLS OVERLAY (Watch Mode: Quality & Minimize) -->
      <div
        v-if="!isMini"
        class="absolute top-3 right-3 z-30 pointer-events-auto flex items-center gap-2"
        @click.stop
      >
        <!-- Quality Selector Button -->
        <div v-if="player.availableQualities.value.length > 0" class="relative">
          <button
            class="px-2.5 py-1 rounded-full bg-black/60 hover:bg-black/80 text-white text-xs font-medium flex items-center gap-1.5 backdrop-blur-sm border border-white/20 transition-all shadow-md"
            title="Change Resolution (สลับความละเอียด)"
            @click.stop="showQualityMenu = !showQualityMenu"
          >
            <Settings class="h-3.5 w-3.5" />
            <span>{{ player.currentQuality.value }}</span>
          </button>

          <!-- Dropdown Menu -->
          <div
            v-if="showQualityMenu"
            class="absolute top-9 right-0 min-w-36 bg-card/95 border border-border backdrop-blur-md rounded-xl p-1.5 shadow-2xl flex flex-col gap-0.5 text-xs text-foreground z-40 max-h-60 overflow-y-auto"
          >
            <div
              class="px-2 py-1 text-[10px] font-bold uppercase tracking-wider text-muted-foreground"
            >
              Quality
            </div>
            <button
              v-for="q in player.availableQualities.value"
              :key="q.itag"
              class="flex items-center justify-between px-2.5 py-1.5 rounded-lg hover:bg-accent text-left transition-colors"
              :class="{
                'font-bold text-primary':
                  player.currentQuality.value === q.label,
              }"
              @click.stop="onSelectQuality(q)"
            >
              <span>{{ q.label }}</span>
              <Check
                v-if="player.currentQuality.value === q.label"
                class="h-3.5 w-3.5"
              />
            </button>
          </div>
        </div>

        <!-- Minimize to Mini Player Button -->
        <button
          class="p-1.5 rounded-full bg-black/60 hover:bg-black/80 text-white transition-colors backdrop-blur-sm border border-white/20 shadow-md"
          title="Minimize to Mini Player (ย่อหน้าต่าง)"
          @click.stop="player.minimizeToMiniPlayer"
        >
          <Minimize2 class="h-3.5 w-3.5" />
        </button>
      </div>

      <!-- MINI-PLAYER HOVER OVERLAY (Shown only in Mini Player mode) -->
      <div
        v-if="isMini"
        class="absolute inset-0 flex items-center justify-center bg-black/40 opacity-0 hover:opacity-100 transition-opacity duration-200 z-20 pointer-events-auto group"
        @click.stop
      >
        <button
          class="p-1.5 rounded-full bg-black/60 hover:bg-black/80 text-white transition-colors absolute top-2 left-2"
          title="Expand to Full Player"
          @click.stop="expandToWatch"
        >
          <ExternalLink class="h-4 w-4" />
        </button>

        <button
          class="p-1.5 rounded-full bg-black/60 hover:bg-destructive/80 text-white transition-colors absolute top-2 right-2"
          title="Close Player"
          @click.stop="player.closePlayer"
        >
          <X class="h-4 w-4" />
        </button>

        <button
          class="flex h-12 w-12 items-center justify-center rounded-full bg-primary/90 text-primary-foreground shadow-lg hover:scale-105 active:scale-95 transition-all"
          @click.stop="player.togglePlay"
        >
          <Pause v-if="player.isPlaying.value" class="h-6 w-6 fill-current" />
          <Play v-else class="h-6 w-6 fill-current translate-x-0.5" />
        </button>
      </div>
    </div>

    <!-- MINI-PLAYER METADATA STRIP (Shown only in Mini Player mode) -->
    <div
      v-if="isMini"
      class="p-3 flex items-center justify-between gap-3 bg-card cursor-pointer border-t border-border/40"
      @click="expandToWatch"
    >
      <div class="flex flex-col min-w-0 flex-1">
        <h4
          class="text-xs font-semibold text-foreground truncate hover:text-primary transition-colors"
        >
          {{ player.currentVideo.value?.title || "Video" }}
        </h4>
        <p class="text-[11px] text-muted-foreground truncate">
          {{ player.currentVideo.value?.channelName }}
        </p>
      </div>

      <div class="flex items-center gap-1 shrink-0">
        <button
          class="p-1.5 rounded-full hover:bg-accent text-foreground transition-colors"
          @click.stop="player.togglePlay"
        >
          <Pause v-if="player.isPlaying.value" class="h-4 w-4 fill-current" />
          <Play v-else class="h-4 w-4 fill-current translate-x-0.5" />
        </button>

        <button
          class="p-1.5 rounded-full hover:bg-destructive/10 text-destructive transition-colors"
          title="Close Player"
          @click.stop="player.closePlayer"
        >
          <X class="h-4 w-4" />
        </button>
      </div>
    </div>
  </div>
</template>

<style>
.mini-player-skin video-skin [part="controls"],
.mini-player-skin video-skin media-controls,
.mini-player-skin video-skin .media-controls {
  display: none !important;
  pointer-events: none !important;
}
</style>
