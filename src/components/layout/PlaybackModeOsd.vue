<script setup lang="ts">
import { computed } from "vue";
import { AudioLines, Clapperboard } from "@lucide/vue";
import { usePlaybackModeNotice } from "@/composables/usePlaybackModeNotice";

const { visible, mode } = usePlaybackModeNotice();

const current = computed(() =>
  mode.value === "audio-only"
    ? { icon: AudioLines, label: "Audio only" }
    : { icon: Clapperboard, label: "Video" },
);
</script>

<template>
  <!-- Screen readers get the switch as text; the chip itself is decorative. -->
  <p class="sr-only" role="status" aria-live="polite">
    {{ visible ? `Playback mode: ${current.label}` : "" }}
  </p>

  <Transition name="rt-osd">
    <div
      v-if="visible"
      aria-hidden="true"
      class="pointer-events-none fixed inset-0 z-60 grid place-items-center"
    >
      <div
        class="rt-osd flex flex-col items-center gap-3 rounded-[1.75rem] border border-border/60 bg-background/85 px-9 py-7 text-center shadow-2xl backdrop-blur-xl"
      >
        <Transition name="rt-osd-icon" mode="out-in">
          <component
            :is="current.icon"
            :key="current.label"
            class="h-12 w-12 text-foreground"
            stroke-width="1.4"
          />
        </Transition>
        <div class="space-y-1">
          <p class="text-sm font-semibold tracking-tight text-foreground">
            {{ current.label }}
          </p>
          <!-- Neutral caption: the mode itself does not change playback yet. -->
          <p
            class="text-[10px] uppercase tracking-[0.18em] text-muted-foreground"
          >
            Playback mode
          </p>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
/*
 * Short, self-dismissing OSD: fade + a small scale so the change reads as an
 * event rather than another piece of static chrome.
 *
 * The scale lives on the card, never on the full-screen layer: a transformed
 * ancestor would change what `backdrop-blur` samples and the glass would flicker.
 */
.rt-osd-enter-active {
  transition: opacity 180ms ease-out;
}

.rt-osd-leave-active {
  transition: opacity 250ms ease-in;
}

.rt-osd-enter-from,
.rt-osd-leave-to {
  opacity: 0;
}

.rt-osd-enter-active .rt-osd {
  transition: transform 180ms cubic-bezier(0.2, 0.8, 0.2, 1);
}

.rt-osd-leave-active .rt-osd {
  transition: transform 250ms ease-in;
}

.rt-osd-enter-from .rt-osd {
  transform: scale(0.94);
}

.rt-osd-leave-to .rt-osd {
  transform: scale(1.03);
}

/* Swapping modes while the chip is still up: cross-fade just the icon. */
.rt-osd-icon-enter-active,
.rt-osd-icon-leave-active {
  transition: opacity 120ms ease;
}

.rt-osd-icon-enter-from,
.rt-osd-icon-leave-to {
  opacity: 0;
}

@media (prefers-reduced-motion: reduce) {
  .rt-osd-enter-active,
  .rt-osd-leave-active,
  .rt-osd-enter-active .rt-osd,
  .rt-osd-leave-active .rt-osd {
    transition-duration: 120ms;
  }

  .rt-osd-enter-from .rt-osd,
  .rt-osd-leave-to .rt-osd {
    transform: none;
  }
}
</style>
