<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter, useRoute } from "vue-router";
import { usePlayer } from "@/composables/usePlayer";
import { useSettings } from "@/composables/useSettings";
import { useControlsHold } from "./useControlsHold";
import { usePlayerStage } from "./usePlayerStage";
import PlayerSkin from "./PlayerSkin.vue";
import WatchPlayerChrome from "./WatchPlayerChrome.vue";
import MiniPlayerBar from "./MiniPlayerBar.vue";
import AutoplayNextOverlay from "./AutoplayNextOverlay.vue";

defineOptions({ name: "GlobalVideoPlayer" });

const router = useRouter();
const route = useRoute();
const player = usePlayer();
const { settings } = useSettings();

const rootRef = ref<HTMLElement | null>(null);
const stageRef = ref<HTMLElement | null>(null);

const isListenPage = computed(() => route.path.startsWith("/listen"));
const isAudioOnly = computed(() => settings.value.playbackMode !== "normal");
/** Audio mode owns playback: the whole video engine is unmounted to free RAM. */
const isAudioMode = computed(() => player.activeEngine.value === "audio");

/**
 * Mini mode is "the player is not docked in a page": either the user
 * minimized it, or the current page has no player slot. In both cases it
 * stays in its stage (see AppLayout's player host).
 */
const isMini = computed(
  () => player.isMiniPlayer.value || !player.watchAnchor.value,
);

/** Element that should hold the player right now (`null` = stay in the stage). */
const playerHost = computed(() =>
  isMini.value ? null : player.watchAnchor.value,
);

usePlayerStage(rootRef, stageRef, playerHost);
useControlsHold(rootRef, player);

function expandToWatch(): void {
  player.restoreFromMiniPlayer();
  const videoId = player.currentVideo.value?.id;
  if (videoId) {
    if (settings.value.playbackMode === "normal") {
      router.push(`/watch/${videoId}`);
    } else {
      router.push(`/listen/${videoId}`);
    }
  }
}
</script>

<template>
  <!--
    One player for the whole app. `usePlayerStage` moves this element into the
    watch page's slot while that page is open and back into this stage for the
    mini player; it is never re-created, so playback, volume and the HLS engine
    survive the switch.
  -->
  <div ref="stageRef" class="rt-player-stage">
    <div
      v-show="player.isOpen.value && !isAudioMode"
      id="global-player-root"
      ref="rootRef"
      class="rt-player select-none bg-black"
      :class="[
        isListenPage
          ? 'fixed left-[-9999px] top-[-9999px] w-0 h-0 opacity-0 pointer-events-none overflow-hidden'
          : isMini
            ? 'rt-player--mini pointer-events-auto w-full shadow-2xl ring-1 ring-border'
            : 'w-full h-full',
      ]"
    >
      <div
        class="rt-player__frame aspect-video w-full"
        :class="{
          'sr-only opacity-0 h-0 aspect-auto overflow-hidden pointer-events-none':
            isListenPage || (isMini && isAudioOnly),
        }"
      >
        <PlayerSkin
          v-if="!isAudioMode && !player.videoReleased.value && player.isOpen.value"
        >
          <WatchPlayerChrome v-if="!isMini" />
        </PlayerSkin>

        <AutoplayNextOverlay
          v-if="
            player.status.value === 'ended' &&
            player.autoplayCountdown.value !== null &&
            player.upNextVideo.value
          "
          :next-video="player.upNextVideo.value"
          :countdown="player.autoplayCountdown.value"
          @play="player.playNext"
          @cancel="player.cancelAutoplay"
        />
      </div>

      <!-- Mini chrome lives below the video: the video itself is Video.js'. -->
      <MiniPlayerBar
        v-if="isMini && !isListenPage && !isAudioMode"
        @expand="expandToWatch"
        @close="player.closePlayer"
      />
    </div>
  </div>
</template>

<style>
/*
 * The player corner. The skin exposes the corner radius as
 * `--media-border-radius`, so the frame the skin draws and the clip this
 * component applies stay identical instead of fighting each other.
 */
.rt-player {
  --media-border-radius: var(--rt-player-radius);
  border-radius: var(--rt-player-radius);
  overflow: hidden;
}

.rt-player--mini {
  --rt-player-radius: 8px;
}

/*
 * Compact chrome buttons: one shared round button for the watch-mode overlay
 * (autoplay, minimize, quality) so they line up instead of mixing a long
 * switch pill with two differently shaped buttons.
 *
 * Colours come from the app's own sidebar palette (the surface these buttons
 * sit next to), never from values picked here.
 */
.rt-chrome-btn {
  display: inline-flex;
  flex: none;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  color: var(--sidebar-foreground);
  cursor: pointer;
  background: var(--sidebar);
  border: 1px solid var(--sidebar-border);
  border-radius: 9999px;
  box-shadow: var(--shadow-sm);
  transition:
    background-color 0.15s ease,
    color 0.15s ease,
    transform 0.15s ease;
}

.rt-chrome-btn:hover {
  background: var(--sidebar-accent);
  color: var(--sidebar-accent-foreground);
}

.rt-chrome-btn:active {
  transform: scale(0.94);
}

.rt-chrome-btn:focus-visible {
  outline: 2px solid var(--sidebar-ring);
  outline-offset: 2px;
}

/* Autoplay reads as "off" by dimming its icon, not by changing its shape. */
.rt-chrome-btn--off {
  opacity: 0.55;
}

.rt-chrome-btn--off:hover {
  opacity: 1;
}

.rt-player__frame {
  position: relative;
  width: 100%;
  aspect-ratio: 16 / 9;
}

/* Outranks the skin's own `:host` rule so the skin fills its frame. */
.rt-player__frame .rt-player__skin {
  display: block;
  width: 100%;
  height: 100%;
}
</style>
