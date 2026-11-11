<script setup lang="ts">
import { usePlayer } from "@/composables/usePlayer";
import { ListVideo, Minimize2, PictureInPicture2 } from "@lucide/vue";
import PlayerQualityMenu from "./PlayerQualityMenu.vue";

defineOptions({ name: "WatchPlayerChrome" });

const player = usePlayer();
</script>

<template>
  <!--
    Watch-mode chrome only, as a compact row of round buttons. It has no
    full-size hit area and stops clicks from reaching the player's
    click-to-pause gesture.
  -->
  <div class="absolute top-2.5 right-3 z-40 flex items-center gap-1.5">
    <!-- Autoplay next: compact icon toggle instead of a long pill -->
    <button
      type="button"
      class="rt-chrome-btn"
      :class="{ 'rt-chrome-btn--off': !player.isAutoplayEnabled.value }"
      :aria-pressed="player.isAutoplayEnabled.value"
      :title="
        player.isAutoplayEnabled.value ? 'Autoplay: on' : 'Autoplay: off'
      "
      :aria-label="
        player.isAutoplayEnabled.value
          ? 'Turn autoplay off'
          : 'Turn autoplay on'
      "
      @click.stop="player.toggleAutoplay()"
    >
      <ListVideo class="h-4 w-4" />
    </button>

    <button
      type="button"
      class="rt-chrome-btn"
      title="Picture in picture"
      aria-label="Picture in picture"
      @click.stop="void player.togglePictureInPicture()"
    >
      <PictureInPicture2 class="h-4 w-4" />
    </button>

    <button
      type="button"
      class="rt-chrome-btn"
      title="Minimize to mini player"
      aria-label="Minimize to mini player"
      @click.stop="player.minimizeToMiniPlayer"
    >
      <Minimize2 class="h-4 w-4" />
    </button>

    <PlayerQualityMenu />
  </div>
</template>
