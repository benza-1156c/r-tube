<script setup lang="ts">
import { onActivated, onMounted, ref } from "vue";
import { History } from "@lucide/vue";
import { commands, type VideoItem } from "@/bindings";
import VideoGrid from "@/components/video/VideoGrid.vue";

defineOptions({ name: "HistoryPage" });

const videos = ref<VideoItem[]>([]);
const isLoading = ref(false);
const errorMessage = ref<string | null>(null);
let activatedOnce = false;

async function loadHistory(): Promise<void> {
  isLoading.value = true;
  errorMessage.value = null;
  try {
    const res = await commands.getHistory();
    if (res.status === "ok") {
      videos.value = res.data;
    } else {
      errorMessage.value = res.error.message || "Failed to load watch history";
    }
  } catch (err) {
    errorMessage.value =
      err instanceof Error ? err.message : "Failed to load watch history";
  } finally {
    isLoading.value = false;
  }
}

onMounted(loadHistory);

// The layout keeps this page alive, so refresh it every time it comes back.
onActivated(() => {
  if (activatedOnce) {
    void loadHistory();
  } else {
    activatedOnce = true;
  }
});
</script>

<template>
  <div class="mx-auto flex w-full max-w-7xl flex-col gap-2 py-6">
    <header class="flex items-center gap-3 border-b border-border px-6 pb-4">
      <div
        class="flex h-10 w-10 items-center justify-center rounded-xl bg-accent text-accent-foreground"
      >
        <History class="h-6 w-6" />
      </div>
      <div>
        <h1 class="text-xl font-bold text-foreground">History</h1>
        <p class="text-xs text-muted-foreground">
          Videos you have previously watched
        </p>
      </div>
    </header>

    <div
      v-if="errorMessage"
      class="mx-6 mt-4 flex flex-col items-center gap-3 rounded-xl border border-border p-10 text-center"
    >
      <p class="text-sm text-muted-foreground">{{ errorMessage }}</p>
      <Button variant="secondary" size="sm" @click="loadHistory">
        Try again
      </Button>
    </div>

    <div
      v-else-if="!isLoading && videos.length === 0"
      class="mx-6 mt-4 rounded-xl border border-dashed border-border p-12 text-center text-sm text-muted-foreground"
    >
      Your watch history will be saved and displayed here
    </div>

    <VideoGrid v-else :videos="videos" :loading="isLoading" />
  </div>
</template>
