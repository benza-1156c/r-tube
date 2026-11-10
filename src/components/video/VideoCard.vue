<script setup lang="ts">
import { VideoItem } from "@/bindings";
import { CheckCircle2, Play, ListMusic } from "@lucide/vue";

const props = defineProps<{
  video: VideoItem;
}>();

const router = useRouter();

function openVideo() {
  if (props.video.playlistId) {
    router.push({
      path: `/watch/${props.video.id}`,
      query: { list: props.video.playlistId },
    });
  } else {
    router.push(`/watch/${props.video.id}`);
  }
}
</script>

<template>
  <div
    class="group flex flex-col gap-3 cursor-pointer select-none"
    @click="openVideo"
  >
    <!-- Thumbnail Container -->
    <div
      class="relative aspect-video w-full overflow-hidden rounded-xl bg-muted shadow-xs"
    >
      <img
        :src="video.thumbnailUrl"
        :alt="video.title"
        class="h-full w-full object-cover transition-transform duration-300 group-hover:scale-105"
        loading="lazy"
      />

      <!-- Play Overlay on Hover -->
      <div
        class="absolute inset-0 flex items-center justify-center bg-black/20 opacity-0 transition-opacity duration-200 group-hover:opacity-100"
      >
        <div
          class="flex h-11 w-11 items-center justify-center rounded-full bg-black/60 backdrop-blur-xs text-white"
        >
          <Play class="h-5 w-5 fill-white translate-x-0.5" />
        </div>
      </div>

      <!-- Duration Badge -->
      <span
        v-if="video.duration"
        class="absolute bottom-2 right-2 flex items-center gap-1 rounded-md bg-black/80 px-1.5 py-0.5 text-xs font-semibold tracking-wide text-white"
      >
        <ListMusic
          v-if="video.playlistId || video.duration === 'Mix' || video.duration.toLowerCase().includes('video')"
          class="h-3 w-3"
        />
        {{ video.duration }}
      </span>
    </div>

    <!-- Metadata Section -->
    <div class="flex gap-3 px-0.5">
      <!-- Channel Avatar -->
      <div class="shrink-0">
        <img
          v-if="video.channelAvatarUrl"
          :src="video.channelAvatarUrl"
          :alt="video.channelName"
          class="h-9 w-9 rounded-full object-cover ring-1 ring-border"
          loading="lazy"
        />
        <div
          v-else
          class="flex h-9 w-9 items-center justify-center rounded-full bg-muted text-muted-foreground ring-1 ring-border"
        >
          <ListMusic
            v-if="video.playlistId || video.duration === 'Mix'"
            class="h-4 w-4"
          />
          <span v-else class="text-xs font-semibold uppercase">
            {{ video.channelName?.charAt(0) || "Y" }}
          </span>
        </div>
      </div>

      <!-- Info Details -->
      <div class="flex flex-col min-w-0 flex-1">
        <h3
          class="line-clamp-2 text-sm font-semibold leading-snug text-foreground group-hover:text-primary transition-colors"
          :title="video.title"
        >
          {{ video.title }}
        </h3>

        <div class="mt-1 flex items-center gap-1 text-xs text-muted-foreground">
          <span class="hover:text-foreground truncate">{{
            video.channelName
          }}</span>
          <CheckCircle2
            v-if="video.isVerified"
            class="h-3 w-3 shrink-0 text-muted-foreground fill-muted-foreground/30"
          />
        </div>

        <div
          v-if="video.viewCount || video.publishedAt"
          class="flex items-center gap-1 text-xs text-muted-foreground/80"
        >
          <span v-if="video.viewCount">{{ video.viewCount }}</span>
          <span v-if="video.viewCount && video.publishedAt">•</span>
          <span v-if="video.publishedAt">{{ video.publishedAt }}</span>
        </div>
      </div>
    </div>
  </div>
</template>
