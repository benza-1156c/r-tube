<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { commands, type ChannelPage_Deserialize } from "@/bindings";
import { Loader2, UserPlus, UserCheck } from "@lucide/vue";

defineOptions({ name: "ChannelPage" });

const route = useRoute();
const router = useRouter();

const channelId = computed(() =>
  String(
    (route.params as unknown as Record<string, string | undefined>).id ?? "",
  ),
);
const page = ref<ChannelPage_Deserialize | null>(null);
const loading = ref(false);
const subscribed = ref(false);
const savingSubscription = ref(false);

async function load(): Promise<void> {
  if (!channelId.value) return;
  loading.value = true;
  try {
    const res = await commands.getChannelPage(channelId.value);
    if (res.status === "ok") {
      page.value = res.data;
      subscribed.value = false;
    }
  } catch (err) {
    console.warn("Failed to load channel:", err);
  } finally {
    loading.value = false;
  }
}

async function toggleSubscription(): Promise<void> {
  if (!channelId.value || savingSubscription.value) return;
  savingSubscription.value = true;
  const next = !subscribed.value;
  try {
    const res = await commands.setSubscription(channelId.value, next);
    if (res.status === "ok") {
      subscribed.value = next;
      subscribed.value = next;
    }
  } catch (err) {
    console.warn("Failed to update subscription:", err);
  } finally {
    savingSubscription.value = false;
  }
}

function play(videoId: string): void {
  void router.push(`/watch/${videoId}`);
}

onMounted(load);
watch(channelId, load);
</script>

<template>
  <div class="flex flex-col min-h-full">
    <div v-if="loading" class="flex items-center justify-center py-24">
      <Loader2 class="h-6 w-6 animate-spin text-muted-foreground" />
    </div>

    <template v-else-if="page">
      <!-- Banner -->
      <div
        class="relative h-32 w-full bg-muted md:h-40"
        :style="
          page.info.banner
            ? {
                backgroundImage: `url(${page.info.banner})`,
                backgroundSize: 'cover',
                backgroundPosition: 'center',
              }
            : undefined
        "
      />

      <!-- Header -->
      <div class="flex flex-col gap-4 px-6 py-5 md:flex-row md:items-center">
        <div
          class="h-20 w-20 shrink-0 overflow-hidden rounded-full bg-muted ring-2 ring-background"
        >
          <img
            v-if="page.info.avatar"
            :src="page.info.avatar"
            :alt="page.info.name"
            class="h-full w-full object-cover"
          />
        </div>

        <div class="min-w-0 flex-1">
          <h1 class="truncate text-xl font-bold text-foreground">
            {{ page.info.name || "Channel" }}
          </h1>
          <p class="mt-1 text-xs text-muted-foreground">
            <span v-if="page.info.subscriberCount">{{
              page.info.subscriberCount
            }}</span>
            <span v-if="page.info.subscriberCount && page.info.videoCount">
              ·
            </span>
            <span v-if="page.info.videoCount">{{ page.info.videoCount }}</span>
          </p>
          <p
            v-if="page.info.description"
            class="mt-2 line-clamp-2 max-w-2xl text-xs text-muted-foreground"
          >
            {{ page.info.description }}
          </p>
        </div>

        <button
          type="button"
          class="inline-flex h-9 items-center gap-1.5 self-start rounded-full px-4 text-xs font-medium transition-colors cursor-pointer"
          :class="
            subscribed
              ? 'bg-muted text-foreground hover:bg-muted/70'
              : 'bg-foreground text-background hover:bg-foreground/90'
          "
          :disabled="savingSubscription"
          @click="toggleSubscription"
        >
          <UserCheck v-if="subscribed" class="h-4 w-4" />
          <UserPlus v-else class="h-4 w-4" />
          {{ subscribed ? "Subscribed" : "Subscribe" }}
        </button>
      </div>

      <!-- Videos -->
      <div class="px-6 pb-8">
        <h2 class="mb-3 text-sm font-semibold text-foreground">Videos</h2>

        <p
          v-if="page.videos.length === 0"
          class="text-sm text-muted-foreground"
        >
          This channel has no public videos.
        </p>

        <div
          v-else
          class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4"
        >
          <button
            v-for="video in page.videos"
            :key="video.id"
            type="button"
            class="group flex flex-col gap-2 text-left cursor-pointer"
            @click="play(video.id)"
          >
            <div
              class="aspect-video w-full overflow-hidden rounded-xl bg-muted"
            >
              <img
                v-if="video.thumbnail"
                :src="video.thumbnail"
                :alt="video.title"
                class="h-full w-full object-cover transition-transform duration-300 group-hover:scale-105"
                loading="lazy"
              />
            </div>
            <div class="min-w-0">
              <p
                class="line-clamp-2 text-xs font-semibold text-foreground group-hover:text-primary"
              >
                {{ video.title }}
              </p>
              <p class="mt-1 text-[11px] text-muted-foreground">
                <span v-if="video.views">{{ video.views }}</span>
                <span v-if="video.views && video.published"> · </span>
                <span v-if="video.published">{{ video.published }}</span>
              </p>
            </div>
          </button>
        </div>
      </div>
    </template>

    <div v-else class="flex flex-col items-center justify-center py-24">
      <p class="text-sm text-muted-foreground">Channel not found.</p>
    </div>
  </div>
</template>
