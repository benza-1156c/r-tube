<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from "vue";
import { usePlayer } from "@/composables/usePlayer";

// Must run before the Video.js modules below register their elements: it
// keeps a moved player from being treated as a reconnected one.
import "./playerElementMove";
import "@videojs/html/video/player";
import "@videojs/html/video/skin";
import "@videojs/html/media/hlsjs-video";

defineOptions({ name: "PlayerSkin" });

const player = usePlayer();

/**
 * `<hlsjs-video>` exposes the media element API (`play`, `pause`,
 * `currentTime`, `paused`, `error`, ...) that `usePlayer` drives, so it is
 * typed as the video element it stands in for.
 */
const videoRef = ref<HTMLVideoElement | null>(null);
const playerRef = ref<HTMLElement | null>(null);

/** Video.js elements expose `destroy()` to release their engine and store. */
interface Destroyable {
  destroy?: () => void;
}

onMounted(() => player.attachVideo(videoRef.value));

watch(videoRef, (el) => player.attachVideo(el));

onBeforeUnmount(() => {
  // `keep-alive` opts this tree out of Video.js' automatic teardown, so the
  // HLS engine, the store and every UI element in the skin are released here
  // instead. Skip this and a remounted player leaves its engine behind.
  (videoRef.value as unknown as Destroyable | null)?.destroy?.();
  (playerRef.value as unknown as Destroyable | null)?.destroy?.();
  player.attachVideo(null);
});
</script>

<template>
  <!--
    The player element owns the state and renders nothing: the skin is the
    visible player. `video-skin` keeps its own container, controls, menus and
    captions, so nothing here re-implements them.

    `keep-alive` keeps the player (and its HLS engine) alive while Teleport
    moves it between the watch page and the mini player. Without it, an
    element removed for more than two animation frames destroys itself.
  -->
  <video-player
    ref="playerRef"
    keep-alive
    :content-title="player.currentVideo.value?.title"
    :poster="player.currentVideo.value?.thumbnailUrl"
  >
    <video-skin class="rt-player__skin">
      <!--
        HLS is the only playback path: YouTube serves it as one URL per
        segment, which is what makes seeking work at every quality.
      -->
      <hlsjs-video
        ref="videoRef"
        keep-alive
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

      <!--
        App chrome is slotted into the skin, so it renders inside the skin's
        container: it is positioned against the video, shares its stacking
        context and receives the player through context.
      -->
      <slot />
    </video-skin>
  </video-player>
</template>
