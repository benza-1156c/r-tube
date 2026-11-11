<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { commands, type PlaylistDetailView_Deserialize } from "@/bindings";
import { ListMusic, Loader2, Play } from "@lucide/vue";

defineOptions({ name: "PlaylistPage" });

const route = useRoute();
const router = useRouter();

const playlistId = computed(() =>
  String(
    (route.params as unknown as Record<string, string | undefined>).id ?? "",
  ),
);
const playlist = ref<PlaylistDetailView_Deserialize | null>(null);
const loading = ref(false);

async function load(): Promise<void> {
  if (!playlistId.value) return;
  loading.value = true;
  try {
    const res = await commands.getPlaylist(playlistId.value);
    if (res.status === "ok") playlist.value = res.data;
  } catch (err) {
    console.warn("Failed to load playlist:", err);
  } finally {
    loading.value = false;
  }
}

function playAt(index: number): void {
  const video = playlist.value?.videos[index];
  if (!video) return;
  void router.push({
    path: `/watch/${video.id}`,
    query: { list: playlistId.value, index: String(index + 1) },
  });
}

function playAll(): void {
  if (playlist.value?.videos.length) playAt(0);
}

onMounted(load);
watch(playlistId, load);
</script>

<template>
  <div class="flex flex-col min-h-full p-6">
    <div v-if="loading" class="flex items-center justify-center py-24">
      <Loader2 class="h-6 w-6 animate-spin text-muted-foreground" />
    </div>

    <template v-else-if="playlist">
      <!-- Header -->
      <div class="mb-6 flex flex-col gap-5 sm:flex-row">
        <div
          class="aspect-video w-full max-w-xs shrink-0 overflow-hidden rounded-xl bg-muted"
        >
          <img
            v-if="playlist.thumbnail"
            :src="playlist.thumbnail"
            :alt="playlist.title"
            class="h-full w-full object-cover"
          />
        </div>

        <div class="min-w-0 flex-1">
          <p
            class="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground"
          >
            {{ playlist.isMix ? "Mix" : "Playlist" }}
          </p>
          <h1 class="mt-1 text-xl font-bold text-foreground">
            {{ playlist.title }}
          </h1>
          <p class="mt-1 text-xs text-muted-foreground">
            <span v-if="playlist.author">{{ playlist.author }}</span>
            <span v-if="playlist.author && playlist.videos.length"> · </span>
            <span>{{ playlist.videos.length }} videos</span>
            <span v-if="playlist.viewCount"> · {{ playlist.viewCount }}</span>
          </p>
          <p
            v-if="playlist.description"
            class="mt-2 line-clamp-3 max-w-2xl text-xs text-muted-foreground"
          >
            {{ playlist.description }}
          </p>

          <button
            type="button"
            class="mt-4 inline-flex h-9 items-center gap-2 rounded-full bg-foreground px-4 text-xs font-medium text-background hover:bg-foreground/90 cursor-pointer"
            :disabled="playlist.videos.length === 0"
            @click="playAll"
          >
            <Play class="h-4 w-4 fill-current" />
            Play all
          </button>
        </div>
      </div>

      <!-- Videos -->
      <p
        v-if="playlist.videos.length === 0"
        class="text-sm text-muted-foreground"
      >
        This playlist is empty.
      </p>

      <ul v-else class="flex flex-col">
        <li
          v-for="(video, index) in playlist.videos"
          :key="`${video.id}-${index}`"
        >
          <button
            type="button"
            class="group flex w-full items-center gap-3 rounded-lg p-2 text-left transition-colors hover:bg-muted/60 cursor-pointer"
            @click="playAt(index)"
          >
            <span
              class="w-6 shrink-0 text-center text-xs tabular-nums text-muted-foreground"
            >
              {{ index + 1 }}
            </span>

            <span
              class="relative aspect-video w-28 shrink-0 overflow-hidden rounded-md bg-muted"
            >
              <img
                v-if="video.thumbnail"
                :src="video.thumbnail"
                :alt="video.title"
                class="h-full w-full object-cover"
                loading="lazy"
              />
              <span
                v-if="video.duration"
                class="absolute bottom-1 right-1 rounded bg-black/80 px-1 text-[10px] text-white"
              >
                {{ video.duration }}
              </span>
            </span>

            <span class="min-w-0 flex-1">
              <span
                class="line-clamp-2 text-xs font-semibold text-foreground group-hover:text-primary"
              >
                {{ video.title }}
              </span>
              <span
                class="mt-1 block truncate text-[11px] text-muted-foreground"
              >
                {{ video.author }}
              </span>
            </span>

            <ListMusic
              v-if="video.isPlayable === false"
              class="h-4 w-4 shrink-0 text-muted-foreground"
            />
          </button>
        </li>
      </ul>
    </template>

    <div v-else class="flex flex-col items-center justify-center py-24">
      <p class="text-sm text-muted-foreground">Playlist not found.</p>
    </div>
  </div>
</template>
