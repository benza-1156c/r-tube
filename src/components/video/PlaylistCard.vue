<script setup lang="ts">
import { useRouter } from "vue-router";
import { Play, ListMusic } from "@lucide/vue";
import type { PlaylistItem } from "@/bindings";

const props = defineProps<{
  playlist: PlaylistItem;
}>();

const router = useRouter();

function openPlaylist() {
  router.push({
    path: "/playlist",
    query: { list: props.playlist.id },
  });
}

function playAll(e: MouseEvent) {
  e.stopPropagation();
  const videoId = props.playlist.firstVideoId || props.playlist.id;
  router.push({
    path: `/watch/${videoId}`,
    query: {
      list: props.playlist.id,
      index: "1",
    },
  });
}
</script>

<template>
  <div
    class="group flex flex-col gap-3 cursor-pointer select-none"
    @click="openPlaylist"
  >
    <!-- Thumbnail Container -->
    <div
      class="relative aspect-video w-full overflow-hidden rounded-xl bg-muted shadow-xs"
    >
      <img
        :src="playlist.thumbnailUrl"
        :alt="playlist.title"
        class="h-full w-full object-cover transition-transform duration-300 group-hover:scale-105"
        loading="lazy"
      />

      <!-- Play Overlay on Hover (Instant Play All) -->
      <div
        class="absolute inset-0 flex items-center justify-center bg-black/25 opacity-0 transition-opacity duration-200 group-hover:opacity-100"
        @click="playAll"
      >
        <div
          class="flex h-11 w-11 items-center justify-center rounded-full bg-black/75 backdrop-blur-xs text-white hover:scale-110 transition-transform shadow-lg"
          title="Play all"
        >
          <Play class="h-5 w-5 fill-white translate-x-0.5" />
        </div>
      </div>

      <!-- Playlist / Mix Overlay Bar on the Right Side (YouTube Style) -->
      <div
        class="absolute inset-y-0 right-0 w-2/5 bg-linear-to-l from-black/80 to-transparent flex flex-col items-center justify-center gap-1 text-white opacity-90 group-hover:opacity-100 transition-opacity"
      >
        <ListMusic class="h-5 w-5" />
        <span class="text-[11px] font-semibold tracking-wide">
          {{ playlist.badgeText }}
        </span>
      </div>

      <!-- Duration / Badge -->
      <span
        class="absolute bottom-2 right-2 flex items-center gap-1 rounded-md bg-black/90 px-1.5 py-0.5 text-xs font-semibold tracking-wide text-white"
      >
        <ListMusic class="h-3 w-3" />
        {{ playlist.badgeText }}
      </span>
    </div>

    <!-- Metadata Section -->
    <div class="flex gap-3 px-0.5">
      <!-- Author / Channel Avatar -->
      <div class="shrink-0">
        <img
          v-if="playlist.authorAvatarUrl"
          :src="playlist.authorAvatarUrl"
          :alt="playlist.authorName"
          class="h-9 w-9 rounded-full object-cover ring-1 ring-border"
          loading="lazy"
        />
        <div
          v-else
          class="flex h-9 w-9 items-center justify-center rounded-full bg-muted text-muted-foreground ring-1 ring-border"
        >
          <ListMusic class="h-4 w-4 text-foreground/80" />
        </div>
      </div>

      <!-- Info Details -->
      <div class="flex flex-col min-w-0 flex-1">
        <h3
          class="line-clamp-2 text-sm font-semibold leading-snug text-foreground group-hover:text-primary transition-colors"
          :title="playlist.title"
        >
          {{ playlist.title }}
        </h3>

        <div class="mt-1 flex items-center gap-1 text-xs text-muted-foreground">
          <span class="hover:text-foreground truncate">{{
            playlist.authorName
          }}</span>
        </div>

        <div
          v-if="playlist.viewCount || playlist.lastUpdated"
          class="flex items-center gap-1 text-xs text-muted-foreground/80"
        >
          <span v-if="playlist.viewCount">{{ playlist.viewCount }}</span>
          <span v-if="playlist.viewCount && playlist.lastUpdated">•</span>
          <span v-if="playlist.lastUpdated">{{ playlist.lastUpdated }}</span>
        </div>

        <div class="mt-1">
          <span
            class="text-[11px] font-semibold text-muted-foreground group-hover:text-foreground tracking-wide transition-colors"
          >
            View full playlist
          </span>
        </div>
      </div>
    </div>
  </div>
</template>
