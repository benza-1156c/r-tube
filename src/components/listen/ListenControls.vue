<script setup lang="ts">
import { ref, computed } from "vue";
import {
  Play,
  Pause,
  SkipBack,
  SkipForward,
  Volume2,
  Volume1,
  VolumeX,
  Loader2,
} from "@lucide/vue";

defineOptions({ name: "ListenControls" });

const props = defineProps<{
  currentTime: number;
  duration: number;
  isPlaying: boolean;
  isLoading: boolean;
  volume: number;
  isMuted?: boolean;
}>();

const emit = defineEmits<{
  (e: "seek", time: number): void;
  (e: "togglePlay"): void;
  (e: "previous"): void;
  (e: "next"): void;
  (e: "volumeChange", val: number): void;
  (e: "toggleMute"): void;
}>();

const isSeeking = ref(false);
const seekPosition = ref(0);

const displayTime = computed(() =>
  isSeeking.value ? seekPosition.value : props.currentTime,
);

function formatTime(seconds: number): string {
  if (!seconds || isNaN(seconds) || seconds < 0) return "0:00";
  const sec = Math.floor(seconds);
  const m = Math.floor(sec / 60);
  const s = sec % 60;
  if (m >= 60) {
    const h = Math.floor(m / 60);
    const remM = m % 60;
    return `${h}:${remM.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
  }
  return `${m}:${s.toString().padStart(2, "0")}`;
}

function handleSeekInput(e: Event) {
  isSeeking.value = true;
  seekPosition.value = Number((e.target as HTMLInputElement).value);
}

function handleSeekChange(e: Event) {
  const targetTime = Number((e.target as HTMLInputElement).value);
  emit("seek", targetTime);
  seekPosition.value = targetTime;
  isSeeking.value = false;
}

function handleVolumeInput(e: Event) {
  const val = Number((e.target as HTMLInputElement).value);
  emit("volumeChange", val);
}
</script>

<template>
  <div class="mx-auto mt-4 w-full max-w-lg select-none">
    <!-- Scrubber: time, bar and duration share one line -->
    <div class="flex items-center gap-3">
      <span
        class="w-11 shrink-0 text-right text-[11px] tabular-nums text-muted-foreground"
      >
        {{ formatTime(displayTime) }}
      </span>

      <input
        type="range"
        min="0"
        :max="duration || 100"
        :value="displayTime"
        step="0.5"
        aria-label="Seek track position"
        class="h-1 min-w-0 flex-1 cursor-pointer appearance-none rounded-full bg-muted accent-primary transition-[height] hover:h-1.5"
        @input="handleSeekInput"
        @change="handleSeekChange"
      />

      <span
        class="w-11 shrink-0 text-[11px] tabular-nums text-muted-foreground"
      >
        {{ formatTime(duration) }}
      </span>
    </div>

    <!-- Transport (centered) and volume (right) share the same row -->
    <div class="mt-2 grid grid-cols-[1fr_auto_1fr] items-center gap-2">
      <div aria-hidden="true" />

      <div class="flex items-center gap-5">
        <button
          type="button"
          class="rounded-full p-2 text-foreground/75 transition-colors hover:bg-muted/60 hover:text-foreground cursor-pointer"
          title="Previous"
          aria-label="Previous track"
          @click="emit('previous')"
        >
          <SkipBack class="h-4 w-4" />
        </button>

        <button
          type="button"
          class="flex h-12 w-12 items-center justify-center rounded-full bg-primary text-primary-foreground shadow-md transition-transform hover:scale-105 active:scale-95 cursor-pointer"
          :title="isPlaying ? 'Pause' : 'Play'"
          :aria-label="isPlaying ? 'Pause' : 'Play'"
          @click="emit('togglePlay')"
        >
          <Loader2 v-if="isLoading" class="h-5 w-5 animate-spin" />
          <Pause v-else-if="isPlaying" class="h-5 w-5 fill-current" />
          <Play v-else class="h-5 w-5 translate-x-0.5 fill-current" />
        </button>

        <button
          type="button"
          class="rounded-full p-2 text-foreground/75 transition-colors hover:bg-muted/60 hover:text-foreground cursor-pointer"
          title="Next"
          aria-label="Next track"
          @click="emit('next')"
        >
          <SkipForward class="h-4 w-4" />
        </button>
      </div>

      <div class="flex items-center justify-end gap-1.5">
        <button
          type="button"
          class="rounded-full p-1.5 text-muted-foreground transition-colors hover:text-foreground cursor-pointer"
          :title="isMuted || volume === 0 ? 'Unmute' : 'Mute'"
          :aria-label="isMuted || volume === 0 ? 'Unmute' : 'Mute'"
          @click="emit('toggleMute')"
        >
          <VolumeX v-if="isMuted || volume === 0" class="h-4 w-4" />
          <Volume1 v-else-if="volume < 0.5" class="h-4 w-4" />
          <Volume2 v-else class="h-4 w-4" />
        </button>

        <input
          type="range"
          min="0"
          max="1"
          step="0.01"
          :value="isMuted ? 0 : volume"
          aria-label="Volume slider"
          class="hidden h-1 w-20 cursor-pointer appearance-none rounded-full bg-muted accent-primary sm:block lg:w-24"
          @input="handleVolumeInput"
        />
      </div>
    </div>
  </div>
</template>
