<script setup lang="ts">
import { usePlayer } from "@/composables/usePlayer";
import { Minimize2 } from "@lucide/vue";
import PlayerQualityMenu from "./PlayerQualityMenu.vue";

defineOptions({ name: "WatchPlayerChrome" });

const player = usePlayer();
</script>

<template>
  <!--
    Watch-mode chrome only. It stays out of the way of the skin's own controls
    (it has no full-size hit area) and stops clicks from reaching the player's
    click-to-pause gesture.
  -->
  <div class="absolute top-2.5 right-3 z-40 flex items-center gap-2">
    <button
      type="button"
      class="rounded bg-black/60 p-1.5 text-white shadow-md ring-1 ring-white/20 backdrop-blur-sm transition-colors hover:bg-black/80"
      title="Minimize to mini player"
      aria-label="Minimize to mini player"
      @click.stop="player.minimizeToMiniPlayer"
    >
      <Minimize2 class="h-3.5 w-3.5" />
    </button>

    <PlayerQualityMenu />
  </div>
</template>
