<script setup lang="ts">
import { computed } from "vue";
import { useRouter } from "vue-router";
import { Play } from "@lucide/vue";
import type { PlaylistVideoItem } from "@/bindings";

const props = defineProps<{
  video: PlaylistVideoItem;
  playlistId: string;
  indexNumber: number;
}>();

const emit = defineEmits<{
  (e: "click", video: PlaylistVideoItem): void;
}>();

const router = useRouter();

const displayIndex = computed(() => {
  return props.video.index ?? props.indexNumber;
});

function handleClick() {
  if (!props.video.isPlayable) return;
  emit("click", props.video);
  router.push({
    path: `/watch/${props.video.id}`,
    query: {
      list: props.playlistId,
      index: String(displayIndex.value),
    },
  });
}
</script>

<template>
  <div
    class="group flex items-center gap-3 p-2 rounded-xl transition-colors cursor-pointer select-none"
    :class="
      video.isPlayable
        ? 'hover:bg-accent/60 active:bg-accent'
        : 'opacity-50 cursor-not-allowed'
    "
    @click="handleClick"
  >
    <!-- Index / Play icon on hover -->
    <div
      class="w-6 shrink-0 flex items-center justify-center text-xs font-semibold text-muted-foreground"
    >
      <span class="group-hover:hidden">{{ displayIndex }}</span>
      <Play
        class="h-3.5 w-3.5 fill-foreground text-foreground hidden group-hover:block translate-x-0.5"
      />
    </div>

    <!-- Thumbnail -->
    <div
      class="relative aspect-video w-32 sm:w-36 md:w-40 rounded-lg overflow-hidden bg-muted shrink-0 shadow-xs border border-border/20"
    >
      <img
        v-if="video.thumbnail"
        :src="video.thumbnail"
        :alt="video.title"
        class="h-full w-full object-cover transition-transform duration-300 group-hover:scale-105"
        loading="lazy"
      />
      <div
        v-else
        class="h-full w-full flex items-center justify-center bg-muted text-muted-foreground text-xs"
      >
        No Preview
      </div>

      <!-- Duration badge -->
      <span
        v-if="video.duration"
        class="absolute bottom-1 right-1 rounded-sm bg-black/85 px-1 py-0.5 text-[10px] font-semibold text-white tracking-wide leading-none"
      >
        {{ video.duration }}
      </span>
    </div>

    <!-- Info -->
    <div class="flex flex-col min-w-0 flex-1 justify-center gap-1">
      <h3
        class="text-sm font-medium text-foreground line-clamp-2 leading-snug group-hover:text-primary transition-colors"
        :title="video.title"
      >
        {{ video.title }}
      </h3>

      <div class="flex items-center gap-1 text-xs text-muted-foreground">
        <span class="hover:text-foreground truncate">{{ video.author }}</span>
        <span v-if="!video.isPlayable" class="text-destructive font-medium">
          • Unavailable
        </span>
      </div>
    </div>
  </div>
</template>

