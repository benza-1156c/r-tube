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
  <div class="w-full flex flex-col items-center mt-6 select-none max-w-lg mx-auto">
    <!-- Scrubber Progress Bar -->
    <div class="w-full px-2">
      <div class="relative flex items-center group">
        <input
          type="range"
          min="0"
          :max="duration || 100"
          :value="displayTime"
          step="0.5"
          aria-label="Seek track position"
          class="w-full h-1.5 bg-muted rounded-lg appearance-none cursor-pointer accent-primary group-hover:h-2 transition-all"
          @input="handleSeekInput"
          @change="handleSeekChange"
        />
      </div>

      <div
        class="flex justify-between text-xs text-muted-foreground font-mono mt-1.5"
      >
        <span>{{ formatTime(displayTime) }}</span>
        <span>{{ formatTime(duration) }}</span>
      </div>
    </div>

    <!-- Playback Control Buttons -->
    <div class="flex items-center justify-center gap-6 mt-4 w-full">
      <!-- Previous Track -->
      <button
        type="button"
        class="p-2.5 rounded-full text-foreground/80 hover:text-foreground hover:bg-muted/60 transition-colors cursor-pointer"
        title="Previous"
        aria-label="Previous track"
        @click="emit('previous')"
      >
        <SkipBack class="h-5 w-5" />
      </button>

      <!-- Play / Pause Button -->
      <button
        type="button"
        class="h-14 w-14 rounded-full bg-primary text-primary-foreground flex items-center justify-center hover:scale-105 active:scale-95 transition-transform shadow-lg cursor-pointer"
        :title="isPlaying ? 'Pause' : 'Play'"
        :aria-label="isPlaying ? 'Pause' : 'Play'"
        @click="emit('togglePlay')"
      >
        <Loader2
          v-if="isLoading"
          class="h-6 w-6 animate-spin text-primary-foreground"
        />
        <Pause
          v-else-if="isPlaying"
          class="h-6 w-6 fill-current"
        />
        <Play
          v-else
          class="h-6 w-6 fill-current translate-x-0.5"
        />
      </button>

      <!-- Next Track -->
      <button
        type="button"
        class="p-2.5 rounded-full text-foreground/80 hover:text-foreground hover:bg-muted/60 transition-colors cursor-pointer"
        title="Next"
        aria-label="Next track"
        @click="emit('next')"
      >
        <SkipForward class="h-5 w-5" />
      </button>
    </div>

    <!-- Volume Control -->
    <div class="flex items-center gap-3 mt-4 w-44 text-muted-foreground">
      <button
        type="button"
        class="p-1 hover:text-foreground transition-colors cursor-pointer"
        :title="isMuted || volume === 0 ? 'Unmute' : 'Mute'"
        :aria-label="isMuted || volume === 0 ? 'Unmute' : 'Mute'"
        @click="emit('toggleMute')"
      >
        <VolumeX
          v-if="isMuted || volume === 0"
          class="h-4 w-4"
        />
        <Volume1
          v-else-if="volume < 0.5"
          class="h-4 w-4"
        />
        <Volume2
          v-else
          class="h-4 w-4"
        />
      </button>

      <input
        type="range"
        min="0"
        max="1"
        step="0.01"
        :value="isMuted ? 0 : volume"
        aria-label="Volume slider"
        class="w-full h-1 bg-muted rounded-lg appearance-none cursor-pointer accent-primary"
        @input="handleVolumeInput"
      />
    </div>
  </div>
</template>

