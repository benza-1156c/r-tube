<script setup lang="ts">
import { usePlayer } from "@/composables/usePlayer";
import { Maximize2, X } from "@lucide/vue";

defineOptions({ name: "MiniPlayerBar" });

defineEmits<{
  expand: [];
  close: [];
}>();

const player = usePlayer();
</script>

<template>
  <!--
    The mini player keeps Video.js' own controls on the video: play, volume,
    time, captions and fullscreen are not re-implemented here. This strip sits
    below the video and only carries what Video.js cannot know about — the
    page it belongs to, and the way back to it.
  -->
  <div
    class="flex items-center justify-between gap-3 border-t border-border/40 bg-card px-3 py-2"
  >
    <button
      type="button"
      class="min-w-0 flex-1 text-left"
      title="Open in watch page"
      @click="$emit('expand')"
    >
      <span class="block truncate text-xs font-semibold text-foreground">
        {{ player.currentVideo.value?.title || "Video" }}
      </span>
      <span class="block truncate text-[11px] text-muted-foreground">
        {{ player.currentVideo.value?.channelName }}
      </span>
    </button>

    <div class="flex shrink-0 items-center gap-1">
      <button
        type="button"
        class="rounded p-1.5 text-foreground transition-colors hover:bg-accent"
        title="Open in watch page"
        aria-label="Open in watch page"
        @click="$emit('expand')"
      >
        <Maximize2 class="h-4 w-4" />
      </button>

      <button
        type="button"
        class="rounded p-1.5 text-destructive transition-colors hover:bg-destructive/10"
        title="Close player"
        aria-label="Close player"
        @click="$emit('close')"
      >
        <X class="h-4 w-4" />
      </button>
    </div>
  </div>
</template>
