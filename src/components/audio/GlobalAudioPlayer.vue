<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRoute } from "vue-router";
import { usePlayer } from "@/composables/usePlayer";
import { useAudioPlayer } from "@/composables/useAudioPlayer";
import AudioMiniBar from "./AudioMiniBar.vue";

defineOptions({ name: "GlobalAudioPlayer" });

const route = useRoute();
const player = usePlayer();
const audio = useAudioPlayer();

const audioRef = ref<HTMLAudioElement | null>(null);

/**
 * The audio element lives here, outside the router view: that is what keeps a
 * track playing after the listen page is left, and what makes a video → audio
 * hand-off a source swap instead of a component life cycle.
 */
const showMiniBar = computed(
  () =>
    player.activeEngine.value === "audio" &&
    player.isOpen.value &&
    !route.path.startsWith("/listen"),
);

onMounted(() => audio.attachAudio(audioRef.value));
</script>

<template>
  <audio ref="audioRef" preload="auto" class="hidden" aria-hidden="true" />
  <AudioMiniBar v-if="showMiniBar" />
</template>
