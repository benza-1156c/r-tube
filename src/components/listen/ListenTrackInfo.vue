<script setup lang="ts">
import { CheckCircle2 } from "@lucide/vue";

defineOptions({ name: "ListenTrackInfo" });

defineProps<{
  title?: string;
  author?: string;
  avatarUrl?: string;
  isVerified?: boolean;
}>();
</script>

<template>
  <div class="mt-3 w-full select-none text-center">
    <h1
      class="line-clamp-2 px-2 text-xl font-bold leading-snug text-foreground sm:text-2xl"
      :title="title || 'Loading audio...'"
    >
      {{ title || "Loading audio..." }}
    </h1>

    <div
      class="mt-1.5 flex items-center justify-center gap-2 text-sm text-muted-foreground"
    >
      <img
        v-if="avatarUrl"
        :src="avatarUrl"
        :alt="author"
        class="h-5 w-5 rounded-full object-cover ring-1 ring-border"
      />
      <span class="font-medium transition-colors hover:text-foreground">
        {{ author || "Artist" }}
      </span>
      <CheckCircle2
        v-if="isVerified"
        class="h-3.5 w-3.5 text-muted-foreground fill-muted-foreground/30"
      />
    </div>
  </div>
</template>
