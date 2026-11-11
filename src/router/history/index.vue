<script setup lang="ts">
import { computed, onActivated, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { Clock, Heart, History } from "@lucide/vue";
import { commands, type VideoItem_Deserialize } from "@/bindings";
import VideoGrid from "@/components/video/VideoGrid.vue";
import { useLibrary } from "@/composables/useLibrary";

defineOptions({ name: "HistoryPage" });

type LibraryTab = "history" | "watch-later" | "liked";

const route = useRoute();
const router = useRouter();
const library = useLibrary();

const videos = ref<VideoItem_Deserialize[]>([]);
const isLoading = ref(false);
const errorMessage = ref<string | null>(null);
let activatedOnce = false;

const tab = computed<LibraryTab>(() => {
  const value = route.query.tab;
  return value === "watch-later" || value === "liked" ? value : "history";
});

const tabs: { value: LibraryTab; label: string; icon: typeof History }[] = [
  { value: "history", label: "History", icon: History },
  { value: "watch-later", label: "Watch later", icon: Clock },
  { value: "liked", label: "Liked", icon: Heart },
];

const heading = computed(() => {
  switch (tab.value) {
    case "watch-later":
      return "Watch later";
    case "liked":
      return "Liked videos";
    default:
      return "History";
  }
});

function savedToVideo(saved: {
  id: string;
  title: string;
  thumbnailUrl: string;
  channelName: string;
  duration: string;
}): VideoItem_Deserialize {
  return {
    id: saved.id,
    title: saved.title,
    thumbnailUrl: saved.thumbnailUrl,
    duration: saved.duration,
    channelName: saved.channelName,
    channelAvatarUrl: "",
    isVerified: false,
    isLive: false,
    viewCount: "",
    publishedAt: "",
    playlistId: undefined,
  };
}

const displayedVideos = computed<VideoItem_Deserialize[]>(() => {
  if (tab.value === "watch-later") {
    return library.watchLater.value.map(savedToVideo);
  }
  if (tab.value === "liked") {
    return library.liked.value.map(savedToVideo);
  }
  return videos.value;
});

const emptyMessage = computed(() => {
  if (tab.value === "watch-later") return "Videos you save will appear here";
  if (tab.value === "liked") return "Videos you like will appear here";
  return "Your watch history will be saved and displayed here";
});

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

function selectTab(next: LibraryTab): void {
  void router.replace({
    path: "/history",
    query: next === "history" ? {} : { tab: next },
  });
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
        <component :is="tabs.find((entry) => entry.value === tab)?.icon" class="h-6 w-6" />
      </div>
      <div>
        <h1 class="text-xl font-bold text-foreground">{{ heading }}</h1>
        <p class="text-xs text-muted-foreground">
          {{
            tab === "history"
              ? "Videos you have previously watched"
              : tab === "watch-later"
                ? "Saved for later, stored on this device"
                : "Videos you liked, stored on this device"
          }}
        </p>
      </div>
    </header>

    <!-- Tabs -->
    <div class="flex items-center gap-1 px-6">
      <button
        v-for="entry in tabs"
        :key="entry.value"
        type="button"
        class="inline-flex items-center gap-1.5 rounded-full px-3 py-1.5 text-xs font-medium transition-colors cursor-pointer"
        :class="
          tab === entry.value
            ? 'bg-foreground text-background'
            : 'text-muted-foreground hover:bg-muted'
        "
        @click="selectTab(entry.value)"
      >
        <component :is="entry.icon" class="h-3.5 w-3.5" />
        {{ entry.label }}
      </button>
    </div>

    <div
      v-if="tab === 'history' && errorMessage"
      class="mx-6 mt-4 flex flex-col items-center gap-3 rounded-xl border border-border p-10 text-center"
    >
      <p class="text-sm text-muted-foreground">{{ errorMessage }}</p>
      <Button variant="secondary" size="sm" @click="loadHistory">
        Try again
      </Button>
    </div>

    <div
      v-else-if="!isLoading && displayedVideos.length === 0"
      class="mx-6 mt-4 rounded-xl border border-dashed border-border p-12 text-center text-sm text-muted-foreground"
    >
      {{ emptyMessage }}
    </div>

    <VideoGrid
      v-else
      :videos="displayedVideos"
      :loading="tab === 'history' && isLoading"
    />
  </div>
</template>
