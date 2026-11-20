<script setup lang="ts">
import { nextTick, onActivated, onBeforeUnmount, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { CirclePlay, Loader2 } from "@lucide/vue";
import {
  commands,
  type BrowseFeed_Deserialize,
  type PlaylistItem_Deserialize,
  type VideoItem_Deserialize,
} from "@/bindings";
import VideoGrid from "@/components/video/VideoGrid.vue";

defineOptions({ name: "SubscriptionsPage" });

const router = useRouter();

const videos = ref<VideoItem_Deserialize[]>([]);
const playlists = ref<PlaylistItem_Deserialize[]>([]);
const continuation = ref<string | null>(null);
const isSignedIn = ref<boolean | null>(null);
const isLoading = ref(false);
const isLoadingMore = ref(false);
const errorMessage = ref<string | null>(null);
const bottomTarget = ref<HTMLElement | null>(null);
let observer: IntersectionObserver | null = null;
let activatedOnce = false;

async function loadPage(reset: boolean): Promise<void> {
  if (isLoading.value || isLoadingMore.value) return;

  const token = reset ? null : continuation.value;
  if (!reset && !token) return;

  if (reset) {
    isLoading.value = true;
    errorMessage.value = null;
  } else {
    isLoadingMore.value = true;
  }

  try {
    const res = await commands.getSubscriptionsFeed(token);
    if (res.status !== "ok") {
      errorMessage.value =
        res.error.message || "Failed to load subscriptions";
      return;
    }

    const feed: BrowseFeed_Deserialize = res.data;
    if (reset) {
      videos.value = feed.videos ?? [];
      playlists.value = feed.playlists ?? [];
    } else {
      videos.value.push(...(feed.videos ?? []));
      playlists.value.push(...(feed.playlists ?? []));
    }
    continuation.value = feed.continuationToken ?? null;
  } catch (err) {
    errorMessage.value =
      err instanceof Error ? err.message : "Failed to load subscriptions";
  } finally {
    isLoading.value = false;
    isLoadingMore.value = false;
  }
}

async function load(): Promise<void> {
  try {
    const profile = await commands.getActiveProfile();
    isSignedIn.value = Boolean(profile.status === "ok" && profile.data);
  } catch (err) {
    isSignedIn.value = false;
    errorMessage.value =
      err instanceof Error ? err.message : "Failed to load subscriptions";
    return;
  }

  if (!isSignedIn.value) {
    videos.value = [];
    playlists.value = [];
    continuation.value = null;
    return;
  }
  await loadPage(true);
}

function observeBottom(): void {
  observer?.disconnect();
  observer = new IntersectionObserver(([entry]) => {
    if (entry.isIntersecting && continuation.value) {
      void loadPage(false);
    }
  });
  if (bottomTarget.value) observer.observe(bottomTarget.value);
}

onMounted(async () => {
  await load();
  await nextTick();
  observeBottom();
});

onActivated(() => {
  if (activatedOnce) {
    void load();
  } else {
    activatedOnce = true;
  }
});

onBeforeUnmount(() => {
  observer?.disconnect();
  observer = null;
});
</script>

<template>
  <div class="mx-auto flex w-full max-w-7xl flex-col gap-2 py-6">
    <header class="flex items-center gap-3 border-b border-border px-6 pb-4">
      <div
        class="flex h-10 w-10 items-center justify-center rounded-xl bg-accent text-accent-foreground"
      >
        <CirclePlay class="h-6 w-6" />
      </div>
      <div>
        <h1 class="text-xl font-bold text-foreground">Subscriptions</h1>
      </div>
    </header>

    <div
      v-if="isSignedIn === null"
      class="flex items-center justify-center py-24"
    >
      <Loader2 class="h-6 w-6 animate-spin text-muted-foreground" />
    </div>

    <div
      v-else-if="isSignedIn === false"
      class="mx-6 mt-4 flex flex-col items-center gap-3 rounded-xl border border-border p-10 text-center"
    >
      <p class="text-sm text-muted-foreground">
        Sign in to see videos from the channels you follow.
      </p>
      <Button variant="secondary" size="sm" @click="router.push('/login')">
        Sign in
      </Button>
    </div>

    <div
      v-else-if="errorMessage"
      class="mx-6 mt-4 flex flex-col items-center gap-3 rounded-xl border border-border p-10 text-center"
    >
      <p class="text-sm text-muted-foreground">{{ errorMessage }}</p>
      <Button variant="secondary" size="sm" @click="load">Try again</Button>
    </div>

    <div
      v-else-if="isLoading && videos.length === 0 && playlists.length === 0"
      class="flex items-center justify-center py-24"
    >
      <Loader2 class="h-6 w-6 animate-spin text-muted-foreground" />
    </div>

    <div
      v-else-if="videos.length === 0 && playlists.length === 0"
      class="mx-6 mt-4 rounded-xl p-12 text-center text-sm text-muted-foreground"
    >
      New videos from the channels you follow will appear here.
    </div>

    <template v-else>
      <VideoGrid :videos="videos" :playlists="playlists" />
      <div
        ref="bottomTarget"
        class="flex h-12 w-full items-center justify-center"
      >
        <Loader2
          v-if="isLoadingMore"
          class="h-5 w-5 animate-spin text-muted-foreground"
        />
      </div>
    </template>
  </div>
</template>