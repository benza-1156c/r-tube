<script setup lang="ts">
import { ref, computed, watch, nextTick } from "vue";
import { useRouter } from "vue-router";
import { Play, X, MoreVertical, ListMusic, ChevronDown } from "@lucide/vue";
import type { PlaylistPanelItem } from "@/bindings";

const props = defineProps<{
  playlistId: string;
  items: PlaylistPanelItem[];
  currentVideoId: string;
  currentTitle?: string | null;
}>();

const isCollapsed = ref(false);
const router = useRouter();
const activeItemRef = ref<HTMLElement | null>(null);

const isMix = computed(() => {
  return (
    props.playlistId.startsWith("RD") || props.playlistId.startsWith("VLRD")
  );
});

const queueTitle = computed(() => {
  if (isMix.value) {
    const seedTitle = props.items[0]?.title || props.currentTitle || "";
    return `มิกซ์ - ${seedTitle}`;
  }
  return props.currentTitle || "เพลย์ลิสต์";
});

const queueSubtitle = computed(() => {
  if (isMix.value) {
    return "มิกซ์คือเพลย์ลิสต์ที่ YouTube สร้างให้คุณ";
  }
  return `${props.items.length} วิดีโอ`;
});

const currentIndex = computed(() => {
  const idx = props.items.findIndex(
    (item) => item.videoId === props.currentVideoId || item.isSelected,
  );
  return idx >= 0 ? idx : 0;
});

function toggleCollapse() {
  isCollapsed.value = !isCollapsed.value;
}

function handleCloseButton(e: MouseEvent) {
  e.stopPropagation();
  isCollapsed.value = true;
}

function selectVideo(item: PlaylistPanelItem, idx: number) {
  if (item.videoId === props.currentVideoId) return;

  router.push({
    path: `/watch/${item.videoId}`,
    query: {
      list: props.playlistId,
      index: String(idx + 1),
    },
  });
}

// Auto-scroll to active video item when expanded
watch(
  [() => props.currentVideoId, isCollapsed],
  async ([, collapsed]) => {
    if (collapsed) return;
    await nextTick();
    if (activeItemRef.value) {
      activeItemRef.value.scrollIntoView({
        block: "nearest",
        behavior: "smooth",
      });
    }
  },
  { immediate: true },
);
</script>

<template>
  <div
    class="flex flex-col w-full rounded-2xl border border-border/50 bg-card/60 backdrop-blur-xs overflow-hidden shadow-sm transition-all duration-300"
  >
    <!-- Header (Clicking collapses or expands) -->
    <div
      class="flex flex-col gap-1 p-3.5 bg-muted/40 transition-colors select-none"
      :class="[
        isCollapsed
          ? 'cursor-pointer hover:bg-muted/70'
          : 'border-b border-border/40',
      ]"
      @click="isCollapsed && toggleCollapse()"
    >
      <div class="flex items-center justify-between gap-2">
        <div class="flex flex-col min-w-0 flex-1">
          <div
            class="flex items-center gap-1.5 text-xs text-primary font-medium mb-0.5"
          >
            <ListMusic class="h-3.5 w-3.5 shrink-0" />
            <span class="truncate">{{
              isMix ? "YouTube Mix" : "Playlist"
            }}</span>
            <span class="text-muted-foreground/60">•</span>
            <span class="text-muted-foreground font-normal">
              {{ currentIndex + 1 }} / {{ items.length }}
            </span>
          </div>

          <h2
            class="font-semibold text-sm leading-snug text-foreground line-clamp-1"
            :title="queueTitle"
          >
            {{ queueTitle }}
          </h2>

          <p
            v-if="!isCollapsed"
            class="text-[11px] text-muted-foreground leading-tight truncate mt-0.5"
          >
            {{ queueSubtitle }}
          </p>
        </div>

        <!-- Header Actions -->
        <div class="flex items-center gap-1 shrink-0 -mr-1">
          <!-- When collapsed: show expand button -->
          <button
            v-if="isCollapsed"
            type="button"
            class="p-1.5 rounded-full text-muted-foreground hover:text-foreground hover:bg-muted transition-colors cursor-pointer"
            @click.stop="toggleCollapse"
          >
            <ChevronDown class="h-4 w-4" />
          </button>

          <!-- When expanded: options + collapse (X) -->
          <template v-else>
            <button
              type="button"
              class="p-1 rounded-full text-muted-foreground hover:text-foreground hover:bg-muted transition-colors cursor-pointer"
              @click="handleCloseButton"
            >
              <X class="h-4 w-4" />
            </button>
          </template>
        </div>
      </div>
    </div>

    <!-- Video Queue List (Collapsible) -->
    <div
      v-show="!isCollapsed"
      class="flex flex-col overflow-y-auto max-h-115 divide-y divide-border/20 p-1.5 scrollbar-thin scrollbar-thumb-muted-foreground/20 transition-all duration-300"
    >
      <div
        v-for="(item, idx) in items"
        :key="`${item.videoId}-${idx}`"
        :ref="
          (el) => {
            if (item.videoId === currentVideoId || item.isSelected)
              activeItemRef = el as HTMLElement;
          }
        "
        class="flex items-center gap-2.5 p-2 rounded-xl transition-all cursor-pointer group"
        :class="
          item.videoId === currentVideoId || item.isSelected
            ? 'bg-accent/80 text-accent-foreground shadow-xs'
            : 'hover:bg-muted/50 text-foreground'
        "
        @click="selectVideo(item, idx)"
      >
        <!-- Playing Indicator / Index -->
        <div class="w-4 shrink-0 flex items-center justify-center text-xs">
          <Play
            v-if="item.videoId === currentVideoId || item.isSelected"
            class="h-3 w-3 fill-primary text-primary"
          />
          <span
            v-else
            class="text-[11px] text-muted-foreground font-mono group-hover:hidden"
          >
            {{ idx + 1 }}
          </span>
          <Play
            class="h-3 w-3 text-muted-foreground hidden group-hover:block"
          />
        </div>

        <!-- Thumbnail -->
        <div
          class="relative aspect-video w-24 shrink-0 overflow-hidden rounded-lg bg-muted shadow-2xs"
        >
          <img
            :src="item.thumbnailUrl"
            :alt="item.title"
            class="h-full w-full object-cover transition-transform group-hover:scale-105"
            loading="lazy"
          />
        </div>

        <!-- Title & Channel -->
        <div class="flex flex-col min-w-0 flex-1 justify-center py-0.5">
          <h3
            class="line-clamp-2 text-xs font-medium leading-snug"
            :class="
              item.videoId === currentVideoId || item.isSelected
                ? 'font-semibold text-foreground'
                : 'text-foreground/90 group-hover:text-foreground'
            "
            :title="item.title"
          >
            {{ item.title }}
          </h3>
          <p class="text-[11px] text-muted-foreground truncate mt-0.5">
            {{ item.author }}
          </p>
        </div>

        <!-- Item More Options -->
        <button
          type="button"
          class="p-1 rounded-full text-muted-foreground opacity-0 group-hover:opacity-100 hover:text-foreground hover:bg-muted transition-all cursor-pointer shrink-0"
          @click.stop
        >
          <MoreVertical class="h-3.5 w-3.5" />
        </button>
      </div>
    </div>
  </div>
</template>
