<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import { usePlayer } from "@/composables/usePlayer";
import { useControlsHold } from "./useControlsHold";
import { usePlayerStage } from "./usePlayerStage";
import PlayerSkin from "./PlayerSkin.vue";
import WatchPlayerChrome from "./WatchPlayerChrome.vue";
import MiniPlayerBar from "./MiniPlayerBar.vue";
import AutoplayNextOverlay from "./AutoplayNextOverlay.vue";

defineOptions({ name: "GlobalVideoPlayer" });

const router = useRouter();
const player = usePlayer();

const rootRef = ref<HTMLElement | null>(null);
const stageRef = ref<HTMLElement | null>(null);

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
    router.push(`/watch/${videoId}`);
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
      v-show="player.isOpen.value"
      id="global-player-root"
      ref="rootRef"
      class="rt-player select-none bg-black"
      :class="
        isMini
          ? 'rt-player--mini pointer-events-auto w-full shadow-2xl ring-1 ring-black/10 dark:ring-white/10'
          : 'w-full h-full'
      "
    >
      <div class="rt-player__frame aspect-video w-full">
        <PlayerSkin>
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
        v-if="isMini"
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
 */
.rt-chrome-btn {
  display: inline-flex;
  flex: none;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  color: #fff;
  cursor: pointer;
  background: rgb(0 0 0 / 55%);
  border: 1px solid rgb(255 255 255 / 16%);
  border-radius: 9999px;
  box-shadow: 0 1px 3px rgb(0 0 0 / 35%);
  backdrop-filter: blur(8px);
  transition:
    background-color 0.15s ease,
    color 0.15s ease,
    transform 0.15s ease;
}

.rt-chrome-btn:hover {
  background: rgb(0 0 0 / 78%);
}

.rt-chrome-btn:active {
  transform: scale(0.94);
}

.rt-chrome-btn:focus-visible {
  outline: 2px solid #fff;
  outline-offset: 2px;
}

/* Autoplay reads as "off" by dimming its icon, not by changing its shape. */
.rt-chrome-btn--off {
  color: rgb(255 255 255 / 45%);
}

.rt-chrome-btn--off:hover {
  color: #fff;
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
