<script setup lang="ts">
import { onMounted, ref } from "vue";
import { Flame, Loader2 } from "@lucide/vue";
import { commands, type TrendingFeed_Deserialize } from "@/bindings";
import VideoGrid from "@/components/video/VideoGrid.vue";

defineOptions({ name: "TrendingPage" });

const feed = ref<TrendingFeed_Deserialize | null>(null);
const isLoading = ref(false);
const errorMessage = ref<string | null>(null);

async function load(): Promise<void> {
  isLoading.value = true;
  errorMessage.value = null;
  try {
    const res = await commands.getTrending();
    if (res.status === "ok") {
      feed.value = res.data;
    } else {
      errorMessage.value = res.error.message || "Failed to load trending videos";
    }
  } catch (err) {
    errorMessage.value =
      err instanceof Error ? err.message : "Failed to load trending videos";
  } finally {
    isLoading.value = false;
  }
}

onMounted(load);
</script>

<template>
  <div class="mx-auto flex w-full max-w-7xl flex-col gap-2 py-6">
    <header class="flex items-center gap-3 border-b border-border px-6 pb-4">
      <div
        class="flex h-10 w-10 items-center justify-center rounded-xl bg-accent text-accent-foreground"
      >
        <Flame class="h-6 w-6" />
      </div>
      <div>
        <h1 class="text-xl font-bold text-foreground">Trending</h1>
        <p v-if="feed?.currentTab" class="text-xs text-muted-foreground">
          {{ feed.currentTab }}
        </p>
      </div>
    </header>

    <div
      v-if="isLoading && !feed"
      class="flex items-center justify-center py-24"
    >
      <Loader2 class="h-6 w-6 animate-spin text-muted-foreground" />
    </div>

    <div
      v-else-if="errorMessage"
      class="mx-6 mt-4 flex flex-col items-center gap-3 rounded-xl border border-border p-10 text-center"
    >
      <p class="text-sm text-muted-foreground">{{ errorMessage }}</p>
      <Button variant="secondary" size="sm" @click="load">Try again</Button>
    </div>

    <div
      v-else-if="!feed || feed.videos.length === 0"
      class="mx-6 mt-4 rounded-xl p-12 text-center text-sm text-muted-foreground"
    >
      Nothing is trending right now.
    </div>

    <VideoGrid v-else :videos="feed.videos" />
  </div>
</template>