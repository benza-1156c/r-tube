<script setup lang="ts">
import { ListMusic, Loader2 } from "@lucide/vue";
import type { VideoItem_Deserialize } from "@/bindings";

defineOptions({ name: "ListenRelated" });

defineProps<{
  items: VideoItem_Deserialize[];
  isLoading?: boolean;
}>();

const emit = defineEmits<{
  (e: "select", item: VideoItem_Deserialize): void;
}>();
</script>

<template>
  <div class="flex flex-col gap-3 select-none">
    <div class="flex items-center justify-between">
      <h2 class="font-semibold text-sm text-foreground">Up next</h2>
      <Loader2
        v-if="isLoading"
        class="h-4 w-4 text-muted-foreground animate-spin"
      />
    </div>

    <!-- Related Tracks List -->
    <div class="flex flex-col gap-2.5">
      <div
        v-for="item in items"
        :key="item.id"
        class="flex gap-3 group cursor-pointer hover:bg-muted/40 p-2 rounded-xl transition-colors"
        @click="emit('select', item)"
      >
        <!-- Thumbnail -->
        <div
          class="relative aspect-video w-32 shrink-0 overflow-hidden rounded-lg bg-muted shadow-2xs"
        >
          <img
            :src="item.thumbnailUrl"
            :alt="item.title"
            class="h-full w-full object-cover group-hover:scale-105 transition-transform"
            loading="lazy"
          />
          <span
            v-if="item.duration"
            class="absolute bottom-1 right-1 rounded-sm bg-black/80 px-1 text-[10px] text-white flex items-center gap-0.5"
          >
            <ListMusic
              v-if="item.playlistId || item.duration === 'Mix'"
              class="h-2.5 w-2.5"
            />
            {{ item.duration }}
          </span>
        </div>

        <!-- Info -->
        <div class="flex flex-col min-w-0 flex-1 justify-center">
          <h3
            class="line-clamp-2 text-xs font-semibold leading-snug text-foreground group-hover:text-primary transition-colors"
            :title="item.title"
          >
            {{ item.title }}
          </h3>
          <p class="text-[11px] text-muted-foreground mt-1 truncate">
            {{ item.channelName }}
          </p>
          <div
            v-if="item.viewCount || item.publishedAt"
            class="text-[10px] text-muted-foreground/80 flex items-center gap-1 mt-0.5"
          >
            <span v-if="item.viewCount">{{ item.viewCount }}</span>
            <span v-if="item.viewCount && item.publishedAt">•</span>
            <span v-if="item.publishedAt">{{ item.publishedAt }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

