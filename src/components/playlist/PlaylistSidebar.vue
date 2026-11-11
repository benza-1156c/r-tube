<script setup lang="ts">
import { ref, computed } from "vue";
import { Play, Shuffle, Share2, Check, ListMusic, User } from "@lucide/vue";
import type { PlaylistDetailView } from "@/bindings";

const props = defineProps<{
  playlist: PlaylistDetailView;
}>();

const emit = defineEmits<{
  (e: "playAll"): void;
  (e: "shuffle"): void;
}>();

const isCopied = ref(false);
const isDescExpanded = ref(false);

const videoCountDisplay = computed(() => {
  const count = props.playlist.videoCount ?? props.playlist.videos.length;
  return `${count.toLocaleString()} videos`;
});

function handlePlayAll() {
  emit("playAll");
}

function handleShuffle() {
  emit("shuffle");
}

async function handleShare() {
  const url = window.location.href;
  try {
    await navigator.clipboard.writeText(url);
    isCopied.value = true;
    setTimeout(() => {
      isCopied.value = false;
    }, 2000);
  } catch (err) {
    console.error("Failed to copy URL:", err);
  }
}
</script>

<template>
  <aside
    class="flex flex-col w-full lg:w-88 xl:w-96 shrink-0 lg:sticky lg:top-4 p-5 rounded-2xl bg-linear-to-b from-card/80 to-card/40 border border-border/40 backdrop-blur-md shadow-lg transition-all"
  >
    <!-- Playlist Big Cover -->
    <div
      class="relative aspect-video w-full overflow-hidden rounded-xl bg-muted shadow-md group"
    >
      <img
        v-if="playlist.thumbnail"
        :src="playlist.thumbnail"
        :alt="playlist.title"
        class="h-full w-full object-cover transition-transform duration-500 group-hover:scale-105"
      />
      <div
        v-else
        class="h-full w-full flex items-center justify-center bg-muted text-muted-foreground"
      >
        <ListMusic class="h-14 w-14" />
      </div>

      <!-- Quick Play overlay -->
      <div
        class="absolute inset-0 bg-black/30 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity cursor-pointer"
        @click="handlePlayAll"
      >
        <div
          class="flex h-12 w-12 items-center justify-center rounded-full bg-white text-black shadow-xl"
        >
          <Play class="h-6 w-6 fill-black translate-x-0.5" />
        </div>
      </div>
    </div>

    <!-- Title & Author Section -->
    <div class="mt-4 flex flex-col gap-2">
      <h1
        class="text-xl sm:text-2xl font-bold tracking-tight text-foreground leading-snug line-clamp-2"
        :title="playlist.title"
      >
        {{ playlist.title }}
      </h1>

      <!-- Author -->
      <div class="flex items-center gap-2 text-sm text-foreground/90 font-medium">
        <div
          class="h-6 w-6 rounded-full bg-primary/10 text-primary flex items-center justify-center overflow-hidden shrink-0"
        >
          <User class="h-3.5 w-3.5" />
        </div>
        <span class="truncate">{{ playlist.author || "Unknown Creator" }}</span>
      </div>

      <!-- Meta Info (Video count, views, updated) -->
      <div class="flex flex-wrap items-center gap-1.5 text-xs text-muted-foreground">
        <span>Playlist</span>
        <span>•</span>
        <span>{{ videoCountDisplay }}</span>
        <template v-if="playlist.viewCount">
          <span>•</span>
          <span>{{ playlist.viewCount }}</span>
        </template>
        <template v-if="playlist.lastUpdated">
          <span>•</span>
          <span>{{ playlist.lastUpdated }}</span>
        </template>
      </div>
    </div>

    <!-- Actions (Play All, Shuffle, Share) -->
    <div class="mt-4 flex items-center gap-2.5">
      <button
        type="button"
        class="flex-1 inline-flex items-center justify-center gap-2 h-10 px-4 rounded-full bg-foreground text-background hover:bg-foreground/90 font-semibold text-sm transition-all cursor-pointer shadow-xs active:scale-95"
        @click="handlePlayAll"
      >
        <Play class="h-4 w-4 fill-background translate-x-0.5" />
        <span>Play all</span>
      </button>

      <button
        type="button"
        class="flex-1 inline-flex items-center justify-center gap-2 h-10 px-4 rounded-full bg-secondary text-secondary-foreground hover:bg-secondary/80 font-semibold text-sm transition-all cursor-pointer border border-border/40 active:scale-95"
        @click="handleShuffle"
      >
        <Shuffle class="h-4 w-4" />
        <span>Shuffle</span>
      </button>

      <button
        type="button"
        :title="isCopied ? 'Copied link!' : 'Share playlist'"
        class="inline-flex items-center justify-center h-10 w-10 rounded-full bg-secondary text-secondary-foreground hover:bg-secondary/80 transition-all cursor-pointer border border-border/40 active:scale-95 shrink-0"
        @click="handleShare"
      >
        <Check v-if="isCopied" class="h-4 w-4 text-emerald-500" />
        <Share2 v-else class="h-4 w-4" />
      </button>
    </div>

    <!-- Description (Collapsible) -->
    <div
      v-if="playlist.description"
      class="mt-4 pt-4 border-t border-border/40 text-xs text-muted-foreground leading-relaxed whitespace-pre-wrap"
    >
      <p :class="{ 'line-clamp-3': !isDescExpanded }">
        {{ playlist.description }}
      </p>
      <button
        v-if="playlist.description.length > 120"
        type="button"
        class="mt-1 font-semibold text-foreground hover:underline cursor-pointer"
        @click="isDescExpanded = !isDescExpanded"
      >
        {{ isDescExpanded ? "Show less" : "Show more" }}
      </button>
    </div>
  </aside>
</template>
