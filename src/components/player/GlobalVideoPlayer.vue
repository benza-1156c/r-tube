<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useRouter } from "vue-router";
import { usePlayer } from "@/composables/usePlayer";
import "@videojs/html/video/player";
import "@videojs/html/video/skin";
import "@videojs/html/media/hlsjs-video";
import "@videojs/html/ui/container";
import "@videojs/html/ui/menu";
import "@videojs/html/ui/menu-content";
import "@videojs/html/ui/menu-radio-item";
import "@videojs/html/ui/menu-item-indicator";
import "@videojs/html/ui/quality-radio-group";
import { Play, Pause, Settings, ExternalLink, Minimize2, X } from "@lucide/vue";

defineOptions({ name: "GlobalVideoPlayer" });

const router = useRouter();
const player = usePlayer();

const videoRef = ref<HTMLVideoElement | null>(null);

onMounted(() => {
  if (videoRef.value) {
    player.attachVideo(videoRef.value);
  }
});

watch(
  () => videoRef.value,
  (el) => {
    if (el) {
      player.attachVideo(el);
    }
  },
);

function expandToWatch(): void {
  player.restoreFromMiniPlayer();
  if (player.currentVideo.value?.id) {
    router.push(`/watch/${player.currentVideo.value.id}`);
  }
}

const isMini = computed(
  () => player.isMiniPlayer.value || !player.watchAnchor.value,
);
</script>

<template>
  <!--
    Watch mode renders *inside* the page layout (teleported into the watch
    page's player slot), so the player scrolls with the content. Mini mode
    renders in place, pinned to the corner.
  -->
  <Teleport :to="player.watchAnchor.value ?? 'body'" :disabled="isMini">
    <div
      v-show="player.isOpen.value"
      id="global-player-root"
      class="select-none overflow-hidden bg-black"
      :class="
        isMini
          ? 'fixed bottom-6 right-6 z-50 w-96 rounded-xl shadow-2xl ring-1 ring-white/10'
          : 'w-full h-full rounded-2xl'
      "
    >
      <!-- VIDEO FRAME -->
      <div
        class="relative w-full aspect-video flex items-center justify-center overflow-hidden"
        :class="{ 'mini-player-skin': isMini }"
      >
        <video-player
          :content-title="player.currentVideo.value?.title"
          :poster="player.currentVideo.value?.thumbnailUrl"
          class="w-full h-full block"
        >
          <media-container class="relative w-full h-full">
            <video-skin
              style="
                width: 100%;
                height: 100%;
                aspect-ratio: 16 / 9;
                display: block;
              "
            >
              <!--
                HLS is the only playback path: YouTube serves it as one URL per
                segment, which is what makes seeking work at every quality.
              -->
              <hlsjs-video
                ref="videoRef"
                :src="player.currentSrc.value || undefined"
                playsinline
                autoplay
                @play="player.onPlay"
                @pause="player.onPause"
                @timeupdate="player.onTimeUpdate"
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
              </hlsjs-video>
            </video-skin>

            <!--
              Quality menu: Video.js owns the rendition list, the current value
              and the Auto option. We only place it at the top-right.
            -->
            <div
              v-if="!isMini"
              class="quality-menu-bar absolute top-3 right-3 z-30 flex items-center gap-2"
              @click.stop
            >
              <Button
                type="button"
                commandfor="rt-quality-menu"
                title="Change Resolution"
              >
                <Settings class="h-3.5 w-3.5" />
              </Button>

              <media-menu
                id="rt-quality-menu"
                side="bottom"
                align="end"
                class="quality-menu"
              >
                <media-menu-content>
                  <media-quality-radio-group
                    class="quality-group"
                    label="Quality"
                  />
                </media-menu-content>
              </media-menu>
            </div>

            <!-- Minimize (watch mode only) -->
            <button
              v-if="!isMini"
              class="p-1.5 rounded bg-black/60 hover:bg-black/80 text-white transition-colors backdrop-blur-sm border border-white/20 shadow-md absolute top-3 right-14 z-30"
              title="Minimize to Mini Player"
              @click.stop="player.minimizeToMiniPlayer"
            >
              <Minimize2 class="h-3.5 w-3.5" />
            </button>

            <!-- MINI-PLAYER HOVER OVERLAY -->
            <div
              v-if="isMini"
              class="absolute inset-0 z-20 flex items-center justify-center opacity-0 hover:opacity-100 transition-opacity duration-200 pointer-events-auto"
              @click.stop
            >
              <button
                class="absolute top-2 left-2 p-1.5 rounded-full bg-black/60 hover:bg-black/80 text-white transition-colors"
                title="Expand to Full Player"
                @click.stop="expandToWatch"
              >
                <ExternalLink class="h-4 w-4" />
              </button>

              <button
                class="absolute top-2 right-2 p-1.5 rounded-full bg-black/60 hover:bg-destructive/80 text-white transition-colors"
                title="Close Player"
                @click.stop="player.closePlayer"
              >
                <X class="h-4 w-4" />
              </button>

              <button
                class="flex h-12 w-12 items-center justify-center rounded-full bg-primary/90 text-primary-foreground shadow-lg hover:scale-105 active:scale-95 transition-all"
                @click.stop="player.togglePlay"
              >
                <Pause
                  v-if="player.isPlaying.value"
                  class="h-6 w-6 fill-current"
                />
                <Play v-else class="h-6 w-6 fill-current translate-x-0.5" />
              </button>
            </div>
          </media-container>
        </video-player>
      </div>

      <!-- MINI-PLAYER METADATA STRIP -->
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
  </Teleport>
</template>

<style>
.mini-player-skin video-skin [part="controls"],
.mini-player-skin video-skin media-controls,
.mini-player-skin video-skin .media-controls {
  display: none !important;
  pointer-events: none !important;
}

/*
 * Quality menu styling, following the Video.js quality guide. The markup and
 * behaviour come from Video.js; only the theme lives here.
 */
.quality-menu-bar:has([data-availability="unavailable"]) {
  display: none;
}

.quality-menu {
  --media-menu-side-offset: 8px;
  box-sizing: border-box;
  min-width: 10rem;
  max-height: 16rem;
  padding: 6px;
  overflow: auto;
  overscroll-behavior: none;
  font-size: 12px;
  color: hsl(var(--popover-foreground));
  background: hsl(var(--popover) / 0.97);
  border: 1px solid hsl(var(--border));
  border-radius: 12px;
  box-shadow: 0 12px 32px rgb(0 0 0 / 0.45);
  backdrop-filter: blur(12px);
}

.quality-group {
  display: grid;
  gap: 2px;
}

.quality-menu media-menu-radio-item,
.quality-group media-menu-radio-item {
  display: flex;
  gap: 8px;
  align-items: center;
  justify-content: space-between;
  min-height: 30px;
  padding: 0 10px;
  font: inherit;
  color: inherit;
  cursor: pointer;
  background: none;
  border: 0;
  border-radius: 8px;
}

.quality-menu media-menu-radio-item[data-highlighted],
.quality-menu media-menu-radio-item[aria-checked="true"] {
  background: hsl(var(--accent) / 0.6);
}

.quality-menu media-menu-item-indicator {
  opacity: 0;
  margin-left: auto;
}

.quality-menu
  media-menu-radio-item[aria-checked="true"]
  media-menu-item-indicator {
  opacity: 1;
}
</style>
