<script setup lang="ts">
import { usePlayer } from "@/composables/usePlayer";
import { useMiniPlayerDraggable } from "@/composables/useMiniPlayerDraggable";
import { Maximize2, X } from "@lucide/vue";

defineOptions({ name: "MiniPlayerBar" });

const emit = defineEmits<{
  expand: [];
  close: [];
}>();

const player = usePlayer();
const { onPointerDown, shouldSuppressClick, isDragging } =
  useMiniPlayerDraggable();

function handleTitleClick(): void {
  if (shouldSuppressClick()) return;
  emit("expand");
}
</script>

<template>
  <!--
    The mini player keeps Video.js' own controls on the video: play, volume,
    time, captions and fullscreen are not re-implemented here. This strip sits
    below the video, acts as the drag handle to move the player freely, and
    carries navigation back to the watch page.
  -->
  <div
    class="flex items-center justify-between gap-2 border-t border-border/40 bg-card px-3 py-2 select-none cursor-grab active:cursor-grabbing transition-colors"
    :class="{ 'cursor-grabbing shadow-lg ring-1 ring-primary/40': isDragging }"
    title="Drag to move mini player"
    @pointerdown="onPointerDown"
  >
    <div class="flex items-center gap-2 min-w-0 flex-1">
      <button
        type="button"
        class="min-w-0 flex-1 text-left select-none cursor-pointer"
        @click="handleTitleClick"
      >
        <span class="block truncate text-xs font-semibold text-foreground">
          {{ player.currentVideo.value?.title || "Video" }}
        </span>
        <span class="block truncate text-[11px] text-muted-foreground">
          {{ player.currentVideo.value?.channelName }}
        </span>
      </button>
    </div>

    <div class="flex shrink-0 items-center gap-1">
      <button
        type="button"
        data-action="expand"
        class="rounded p-1.5 text-foreground transition-colors hover:bg-accent"
        aria-label="Open in watch page"
        @click.stop="$emit('expand')"
      >
        <Maximize2 class="h-4 w-4" />
      </button>

      <button
        type="button"
        data-action="close"
        class="rounded p-1.5 text-destructive transition-colors hover:bg-destructive/10"
        title="Close player"
        aria-label="Close player"
        @click.stop="$emit('close')"
      >
        <X class="h-4 w-4" />
      </button>
    </div>
  </div>
</template>
