<script setup lang="ts">
import { CheckCircle2, ThumbsUp, Share2, Bookmark } from "@lucide/vue";

defineOptions({ name: "ListenTrackInfo" });

defineProps<{
  title?: string;
  author?: string;
  avatarUrl?: string;
  isVerified?: boolean;
}>();
</script>

<template>
  <div class="w-full text-center mt-4 select-none">
    <h1
      class="text-xl sm:text-2xl font-bold text-foreground leading-snug line-clamp-2 px-2"
      :title="title || 'Loading audio...'"
    >
      {{ title || "Loading audio..." }}
    </h1>

    <div
      class="flex items-center justify-center gap-2 mt-2 text-sm text-muted-foreground"
    >
      <img
        v-if="avatarUrl"
        :src="avatarUrl"
        :alt="author"
        class="h-5 w-5 rounded-full object-cover ring-1 ring-border"
      />
      <span class="font-medium hover:text-foreground transition-colors">
        {{ author || "Artist" }}
      </span>
      <CheckCircle2
        v-if="isVerified"
        class="h-3.5 w-3.5 text-muted-foreground fill-muted-foreground/30"
      />
    </div>

    <!-- Action Buttons (Like / Share / Save) in English -->
    <div class="flex items-center justify-center gap-2 mt-4">
      <button
        type="button"
        class="inline-flex items-center gap-1.5 px-3.5 py-1.5 rounded-full bg-muted/60 hover:bg-muted text-xs font-medium text-foreground transition-colors cursor-pointer"
      >
        <ThumbsUp class="h-3.5 w-3.5" />
        <span>Like</span>
      </button>
      <button
        type="button"
        class="inline-flex items-center gap-1.5 px-3.5 py-1.5 rounded-full bg-muted/60 hover:bg-muted text-xs font-medium text-foreground transition-colors cursor-pointer"
      >
        <Share2 class="h-3.5 w-3.5" />
        <span>Share</span>
      </button>
      <button
        type="button"
        class="inline-flex items-center gap-1.5 px-3.5 py-1.5 rounded-full bg-muted/60 hover:bg-muted text-xs font-medium text-foreground transition-colors cursor-pointer"
      >
        <Bookmark class="h-3.5 w-3.5" />
        <span>Save</span>
      </button>
    </div>
  </div>
</template>

