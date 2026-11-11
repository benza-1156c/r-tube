<script setup lang="ts">
import { computed } from "vue";
import { useRouter } from "vue-router";
import { Pause, Play, Maximize2, X } from "@lucide/vue";
import { usePlayer } from "@/composables/usePlayer";
import { useAudioPlayer } from "@/composables/useAudioPlayer";

defineOptions({ name: "AudioMiniBar" });

const router = useRouter();
const player = usePlayer();
const audio = useAudioPlayer();

const video = computed(() => player.currentVideo.value);
const isPlaying = computed(() => player.isPlaying.value);

const progress = computed(() => {
  const duration = video.value?.duration || 0;
  if (!duration) return 0;
  return Math.min(100, (player.currentTime.value / duration) * 100);
});

function expand(): void {
  const id = video.value?.id;
  if (id) void router.push(`/listen/${id}`);
}
</script>

<template>
  <!--
    Audio keeps playing outside the listen page, so it needs its own small
    transport: the Video.js mini player is gone while the audio engine owns
    playback.
  -->
  <div
    class="fixed bottom-5 right-5 z-50 w-80 max-w-[calc(100vw-2.5rem)] overflow-hidden rounded-xl border border-border bg-card shadow-2xl"
  >
    <div class="h-0.5 w-full bg-muted">
      <div
        class="h-full bg-primary transition-[width] duration-300"
        :style="{ width: `${progress}%` }"
      />
    </div>

    <div class="flex items-center gap-3 p-3">
      <button
        type="button"
        class="group relative h-12 w-12 shrink-0 overflow-hidden rounded-lg bg-muted cursor-pointer"
        aria-label="Open audio player"
        @click="expand"
      >
        <img
          v-if="video?.thumbnailUrl"
          :src="video.thumbnailUrl"
          :alt="video?.title || ''"
          class="h-full w-full object-cover"
        />
        <span
          class="absolute inset-0 flex items-center justify-center bg-black/40 opacity-0 transition-opacity group-hover:opacity-100"
        >
          <Maximize2 class="h-4 w-4 text-white" />
        </span>
      </button>

      <div class="min-w-0 flex-1">
        <p class="truncate text-xs font-semibold text-foreground">
          {{ video?.title || "Audio" }}
        </p>
        <p class="truncate text-[11px] text-muted-foreground">
          {{ video?.channelName }}
        </p>
      </div>

      <button
        type="button"
        class="rounded-full p-2 text-foreground transition-colors hover:bg-accent cursor-pointer"
        :title="isPlaying ? 'Pause' : 'Play'"
        :aria-label="isPlaying ? 'Pause' : 'Play'"
        @click="player.togglePlay"
      >
        <Pause v-if="isPlaying" class="h-4 w-4 fill-current" />
        <Play v-else class="h-4 w-4 fill-current" />
      </button>

      <button
        type="button"
        class="rounded p-1.5 text-muted-foreground transition-colors hover:bg-accent hover:text-foreground cursor-pointer"
        title="Close audio player"
        aria-label="Close audio player"
        @click="audio.stopAudio"
      >
        <X class="h-4 w-4" />
      </button>
    </div>
  </div>
</template>
