<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import { useRouter } from "vue-router";
import { commands, type VideoComment_Deserialize } from "@/bindings";
import { Loader2, ThumbsUp, Pin, ChevronDown } from "@lucide/vue";

const props = defineProps<{ videoId: string }>();

const router = useRouter();

const comments = ref<VideoComment_Deserialize[]>([]);
const total = ref<string | null>(null);
const continuation = ref<string | null>(null);
const isLoading = ref(false);
const isLoadingMore = ref(false);

async function load(reset: boolean): Promise<void> {
  if (!props.videoId) return;

  if (reset) {
    isLoading.value = true;
    comments.value = [];
    continuation.value = null;
  } else {
    if (!continuation.value || isLoadingMore.value) return;
    isLoadingMore.value = true;
  }

  try {
    const res = await commands.getComments(
      props.videoId,
      reset ? null : continuation.value,
    );
    if (res.status !== "ok") return;

    comments.value = reset
      ? res.data.comments
      : [...comments.value, ...res.data.comments];
    total.value = res.data.total ?? total.value;
    continuation.value = res.data.continuation ?? null;
  } catch (err) {
    console.warn("Failed to load comments:", err);
  } finally {
    isLoading.value = false;
    isLoadingMore.value = false;
  }
}

function openChannel(comment: VideoComment_Deserialize): void {
  if (comment.authorChannelId) {
    void router.push(`/channel/${comment.authorChannelId}`);
  }
}

onMounted(() => {
  if (props.videoId) void load(true);
});

watch(
  () => props.videoId,
  (id) => {
    if (id) void load(true);
  },
);
</script>

<template>
  <section class="mt-6">
    <div class="mb-3 flex items-center gap-2">
      <h2 class="text-sm font-semibold text-foreground">Comments</h2>
      <span v-if="total" class="text-xs text-muted-foreground">{{ total }}</span>
    </div>

    <div v-if="isLoading" class="flex items-center gap-2 py-6 text-sm text-muted-foreground">
      <Loader2 class="h-4 w-4 animate-spin" />
      Loading comments...
    </div>

    <p
      v-else-if="comments.length === 0"
      class="py-4 text-sm text-muted-foreground"
    >
      No comments yet.
    </p>

    <ul v-else class="flex flex-col gap-4">
      <li
        v-for="comment in comments"
        :key="comment.id"
        class="flex gap-3"
      >
        <button
          type="button"
          class="h-8 w-8 shrink-0 overflow-hidden rounded-full bg-muted"
          :aria-label="`Open ${comment.author}'s channel`"
          @click="openChannel(comment)"
        >
          <img
            v-if="comment.authorThumbnail"
            :src="comment.authorThumbnail"
            :alt="comment.author"
            class="h-full w-full object-cover"
            loading="lazy"
          />
        </button>

        <div class="min-w-0 flex-1">
          <div class="flex flex-wrap items-center gap-2 text-xs">
            <button
              type="button"
              class="font-semibold text-foreground hover:text-primary cursor-pointer"
              @click="openChannel(comment)"
            >
              {{ comment.author }}
            </button>
            <span v-if="comment.publishedTime" class="text-muted-foreground">
              {{ comment.publishedTime }}
            </span>
            <span
              v-if="comment.isPinned"
              class="inline-flex items-center gap-1 rounded-full bg-muted px-2 py-0.5 text-[10px] text-muted-foreground"
            >
              <Pin class="h-3 w-3" /> Pinned
            </span>
            <span
              v-if="comment.isOwner"
              class="rounded-full bg-primary/10 px-2 py-0.5 text-[10px] text-primary"
            >
              Creator
            </span>
          </div>

          <p class="mt-1 whitespace-pre-line text-sm leading-relaxed text-foreground/90">
            {{ comment.text }}
          </p>

          <div class="mt-1.5 flex items-center gap-3 text-xs text-muted-foreground">
            <span v-if="comment.likes" class="inline-flex items-center gap-1">
              <ThumbsUp class="h-3 w-3" />
              {{ comment.likes }}
            </span>
            <span v-if="comment.replyCount">
              {{ comment.replyCount }} replies
            </span>
          </div>

          <ul
            v-if="comment.replies.length"
            class="mt-3 flex flex-col gap-3 border-l pl-3"
          >
            <li
              v-for="reply in comment.replies"
              :key="reply.id"
              class="flex gap-2"
            >
              <div class="h-6 w-6 shrink-0 overflow-hidden rounded-full bg-muted">
                <img
                  v-if="reply.authorThumbnail"
                  :src="reply.authorThumbnail"
                  :alt="reply.author"
                  class="h-full w-full object-cover"
                  loading="lazy"
                />
              </div>
              <div class="min-w-0">
                <p class="text-xs font-semibold text-foreground">
                  {{ reply.author }}
                </p>
                <p class="mt-0.5 whitespace-pre-line text-xs text-foreground/85">
                  {{ reply.text }}
                </p>
              </div>
            </li>
          </ul>
        </div>
      </li>
    </ul>

    <button
      v-if="continuation"
      type="button"
      class="mt-4 inline-flex items-center gap-1.5 rounded-full bg-muted px-4 py-2 text-xs font-medium text-foreground hover:bg-muted/70 disabled:opacity-60 cursor-pointer"
      :disabled="isLoadingMore"
      @click="load(false)"
    >
      <Loader2 v-if="isLoadingMore" class="h-3.5 w-3.5 animate-spin" />
      <ChevronDown v-else class="h-3.5 w-3.5" />
      More comments
    </button>
  </section>
</template>
