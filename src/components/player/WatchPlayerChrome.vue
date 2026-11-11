<script setup lang="ts">
import { usePlayer } from "@/composables/usePlayer";
import { Minimize2 } from "@lucide/vue";
import { Switch } from "@/lib/components/ui/switch";
import PlayerQualityMenu from "./PlayerQualityMenu.vue";

defineOptions({ name: "WatchPlayerChrome" });

const player = usePlayer();

function handleAutoplayChange(val: boolean): void {
  player.isAutoplayEnabled.value = val;
  if (!val) {
    player.cancelAutoplay();
  }
}
</script>

<template>
  <!--
    Watch-mode chrome only. It stays out of the way of the skin's own controls
    (it has no full-size hit area) and stops clicks from reaching the player's
    click-to-pause gesture.
  -->
  <div class="absolute top-2.5 right-3 z-40 flex items-center gap-2">
    <!-- Autoplay toggle switch (YouTube style) -->
    <div
      class="flex items-center gap-1.5 rounded-full bg-black/60 px-1.5 py-1 text-white shadow-md ring-1 ring-white/20 backdrop-blur-sm"
      title="Autoplay"
      @click.stop
    >
      <Switch
        :model-value="player.isAutoplayEnabled.value"
        class="h-4 w-7 data-[state=checked]:bg-primary cursor-pointer"
        aria-label="Toggle autoplay"
        @update:model-value="handleAutoplayChange"
      />
    </div>

    <button
      type="button"
      class="rounded bg-black/60 p-1.5 text-white shadow-md ring-1 ring-white/20 backdrop-blur-sm transition-colors hover:bg-black/80 cursor-pointer"
      title="Minimize to mini player"
      aria-label="Minimize to mini player"
      @click.stop="player.minimizeToMiniPlayer"
    >
      <Minimize2 class="h-3.5 w-3.5" />
    </button>

    <PlayerQualityMenu />
  </div>
</template>
