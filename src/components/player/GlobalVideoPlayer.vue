<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import { usePlayer } from "@/composables/usePlayer";
import { useControlsHold } from "./useControlsHold";
import { usePlayerStage } from "./usePlayerStage";
import PlayerSkin from "./PlayerSkin.vue";
import WatchPlayerChrome from "./WatchPlayerChrome.vue";
import MiniPlayerBar from "./MiniPlayerBar.vue";

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
