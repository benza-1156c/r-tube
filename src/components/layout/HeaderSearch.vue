<script setup lang="ts">
import { ref, watch, nextTick } from "vue";
import { useRouter, useRoute } from "vue-router";
import { Search, History, X, ArrowLeft } from "@lucide/vue";
import { useDebounceFn, onClickOutside, useMediaQuery } from "@vueuse/core";
import { commands, type SearchSuggestionItem } from "@/bindings";

const router = useRouter();
const route = useRoute();

const searchQuery = ref("");
const suggestions = ref<SearchSuggestionItem[]>([]);
const showSuggestions = ref(false);
const selectedIndex = ref(-1);
const isInputFocused = ref(false);
const searchContainerRef = ref<HTMLElement | null>(null);

const isMobile = useMediaQuery("(max-width: 767px)");
const isMobileSearchOpen = ref(false);
const mobileInputRef = ref<HTMLInputElement | { $el: HTMLInputElement } | null>(null);

// Synchronize search input if route has search_query
watch(
  () => route.query.search_query,
  (newQuery) => {
    if (typeof newQuery === "string") {
      searchQuery.value = newQuery;
    }
  },
  { immediate: true },
);

const fetchSuggestions = useDebounceFn(async (text: string) => {
  const trimmed = text.trim();
  if (!trimmed || !isInputFocused.value) {
    suggestions.value = [];
    showSuggestions.value = false;
    return;
  }

  try {
    const res = await commands.getSearchSuggestions(trimmed);
    if (res.status === "ok" && isInputFocused.value) {
      suggestions.value = res.data ?? [];
      showSuggestions.value = suggestions.value.length > 0;
      selectedIndex.value = -1;
    }
  } catch (err) {
    console.error("Failed to fetch suggestions:", err);
  }
}, 150);

watch(searchQuery, (newVal) => {
  if (!isInputFocused.value) return;

  if (!newVal.trim()) {
    suggestions.value = [];
    showSuggestions.value = false;
    return;
  }
  fetchSuggestions(newVal);
});

onClickOutside(searchContainerRef, () => {
  showSuggestions.value = false;
  selectedIndex.value = -1;
});

function openMobileSearch() {
  isMobileSearchOpen.value = true;
  isInputFocused.value = true;
  if (searchQuery.value.trim()) {
    fetchSuggestions(searchQuery.value);
  }
  nextTick(() => {
    const el = (mobileInputRef.value as any)?.$el ?? mobileInputRef.value;
    if (el && typeof el.focus === "function") {
      el.focus();
    }
  });
}

function closeMobileSearch() {
  isMobileSearchOpen.value = false;
  isInputFocused.value = false;
  showSuggestions.value = false;
  fetchSuggestions.cancel();
}

function handleSearch(targetQuery?: string) {
  const target = (
    typeof targetQuery === "string" ? targetQuery : searchQuery.value
  ).trim();

  // 1. Cancel any pending debounced fetch so it doesn't reopen
  fetchSuggestions.cancel();

  // 2. Hide suggestions and clear state
  showSuggestions.value = false;
  suggestions.value = [];
  selectedIndex.value = -1;
  isInputFocused.value = false;
  isMobileSearchOpen.value = false;

  // 3. Blur the search input
  if (document.activeElement instanceof HTMLElement) {
    document.activeElement.blur();
  }

  if (!target) return;

  // 4. Update query and navigate
  searchQuery.value = target;
  router.push({
    path: "/results",
    query: { search_query: target },
  });
}

function selectSuggestion(sug: SearchSuggestionItem) {
  handleSearch(sug.text);
}

function clearSearch() {
  fetchSuggestions.cancel();
  searchQuery.value = "";
  suggestions.value = [];
  showSuggestions.value = false;
  selectedIndex.value = -1;
}

function handleFocus() {
  isInputFocused.value = true;
  if (suggestions.value.length > 0) {
    showSuggestions.value = true;
  } else if (searchQuery.value.trim()) {
    fetchSuggestions(searchQuery.value);
  }
}

function handleBlur() {
  setTimeout(() => {
    isInputFocused.value = false;
    showSuggestions.value = false;
  }, 120);
}

function handleKeyDown(e: KeyboardEvent) {
  if (!showSuggestions.value || suggestions.value.length === 0) {
    if (e.key === "Enter") {
      handleSearch();
    }
    return;
  }

  if (e.key === "ArrowDown") {
    e.preventDefault();
    selectedIndex.value = (selectedIndex.value + 1) % suggestions.value.length;
    searchQuery.value = suggestions.value[selectedIndex.value].text;
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    selectedIndex.value =
      selectedIndex.value <= 0
        ? suggestions.value.length - 1
        : selectedIndex.value - 1;
    searchQuery.value = suggestions.value[selectedIndex.value].text;
  } else if (e.key === "Enter") {
    e.preventDefault();
    if (selectedIndex.value >= 0 && suggestions.value[selectedIndex.value]) {
      handleSearch(suggestions.value[selectedIndex.value].text);
    } else {
      handleSearch();
    }
  } else if (e.key === "Escape") {
    fetchSuggestions.cancel();
    showSuggestions.value = false;
    suggestions.value = [];
    selectedIndex.value = -1;
    if (isMobile.value) {
      isMobileSearchOpen.value = false;
    }
    if (document.activeElement instanceof HTMLElement) {
      document.activeElement.blur();
    }
  }
}
</script>

<template>
  <!-- Desktop Search Bar (hidden on mobile, visible on md and up) -->
  <div
    ref="searchContainerRef"
    class="hidden md:flex relative flex-1 max-w-xl mx-4 items-center"
  >
    <div class="relative w-full flex items-center">
      <Input
        v-model="searchQuery"
        type="text"
        placeholder="Search"
        class="h-9 w-full rounded-md border-border bg-muted/40 pl-4 pr-9 text-sm focus-visible:ring-1 focus-visible:ring-primary"
        @focus="handleFocus"
        @blur="handleBlur"
        @keydown="handleKeyDown"
      />
      <button
        v-if="searchQuery"
        type="button"
        class="absolute right-2 text-muted-foreground hover:text-foreground cursor-pointer"
        @click="clearSearch"
      >
        <X class="h-4 w-4" />
      </button>
    </div>

    <!-- Desktop Suggestions Dropdown -->
    <div
      v-if="showSuggestions && suggestions.length > 0"
      class="absolute top-full left-0 right-0 mt-1.5 z-50 rounded-xl border border-border/40 bg-popover/95 text-popover-foreground shadow-2xl backdrop-blur-md overflow-hidden py-1.5"
    >
      <div
        v-for="(sug, idx) in suggestions"
        :key="`${sug.text}-${idx}`"
        class="flex items-center justify-between gap-3 px-3.5 py-2 text-sm cursor-pointer transition-colors"
        :class="
          selectedIndex === idx
            ? 'bg-accent text-accent-foreground font-medium'
            : 'hover:bg-accent/60 text-foreground'
        "
        @mousedown.prevent="selectSuggestion(sug)"
      >
        <!-- Left: Icon (History / Search) + Text & Subtitle -->
        <div class="flex items-center gap-3 min-w-0 flex-1">
          <History
            v-if="sug.isHistory"
            class="h-4 w-4 text-muted-foreground shrink-0"
          />
          <Search v-else class="h-4 w-4 text-muted-foreground shrink-0" />

          <div class="flex flex-col min-w-0 flex-1">
            <span class="truncate font-medium text-sm leading-tight">
              {{ sug.text }}
            </span>
            <span
              v-if="sug.subtitle"
              class="text-xs text-muted-foreground truncate leading-tight mt-0.5"
            >
              {{ sug.subtitle }}
            </span>
          </div>
        </div>

        <!-- Right: Thumbnail (Cover / Avatar / Video) -->
        <div v-if="sug.thumbnail" class="shrink-0 ml-3">
          <img
            :src="sug.thumbnail"
            :alt="sug.text"
            class="h-7 w-12 rounded object-cover shadow-xs border border-border/30 bg-muted"
            loading="lazy"
          />
        </div>
      </div>
    </div>
  </div>

  <!-- Mobile Search Trigger (visible on mobile, hidden on md) -->
  <div class="flex md:hidden flex-1 mx-2 items-center min-w-0">
    <button
      type="button"
      class="flex h-9 w-full items-center gap-2 rounded-full border border-border/60 bg-muted/40 px-3 text-sm text-muted-foreground active:bg-muted/70 transition-colors cursor-pointer"
      @click="openMobileSearch"
    >
      <Search class="h-4 w-4 shrink-0 text-muted-foreground" />
      <span class="truncate text-left flex-1 text-foreground/80">
        {{ searchQuery || "Search" }}
      </span>
      <button
        v-if="searchQuery"
        type="button"
        class="p-0.5 text-muted-foreground hover:text-foreground"
        @click.stop="clearSearch"
      >
        <X class="h-3.5 w-3.5" />
      </button>
    </button>
  </div>

  <!-- Mobile Fullscreen Search Overlay (YouTube Style) -->
  <Teleport to="body">
    <div
      v-if="isMobile && isMobileSearchOpen"
      class="fixed inset-0 z-100 bg-background flex flex-col pt-(--app-safe-top) pb-(--app-safe-bottom)"
    >
      <!-- Mobile Top Bar -->
      <div class="flex h-14 items-center gap-2 px-3 border-b border-border/30 bg-background/95 backdrop-blur-md shrink-0">
        <button
          type="button"
          class="inline-flex h-9 w-9 items-center justify-center rounded-full text-foreground/80 hover:bg-accent cursor-pointer shrink-0"
          @click="closeMobileSearch"
        >
          <ArrowLeft class="h-5 w-5" />
          <span class="sr-only">Back</span>
        </button>

        <div class="relative flex-1 flex items-center min-w-0">
          <Input
            ref="mobileInputRef"
            v-model="searchQuery"
            type="text"
            placeholder="Search YouTube"
            class="h-10 w-full rounded-full border-border bg-muted/40 pl-4 pr-9 text-base focus-visible:ring-1 focus-visible:ring-primary"
            @keydown="handleKeyDown"
          />
          <button
            v-if="searchQuery"
            type="button"
            class="absolute right-3 text-muted-foreground hover:text-foreground cursor-pointer"
            @click="clearSearch"
          >
            <X class="h-4 w-4" />
          </button>
        </div>
      </div>

      <!-- Mobile Suggestions List (Full Screen Width) -->
      <div class="flex-1 overflow-y-auto divide-y divide-border/10 py-1">
        <div
          v-for="(sug, idx) in suggestions"
          :key="`${sug.text}-${idx}`"
          class="flex items-center justify-between gap-3 px-4 py-3 active:bg-accent/70 cursor-pointer transition-colors"
          @click="selectSuggestion(sug)"
        >
          <!-- Left: Icon + Text -->
          <div class="flex items-center gap-3 min-w-0 flex-1">
            <History
              v-if="sug.isHistory"
              class="h-4 w-4 text-muted-foreground shrink-0"
            />
            <Search v-else class="h-4 w-4 text-muted-foreground shrink-0" />

            <div class="flex flex-col min-w-0 flex-1">
              <span class="truncate font-medium text-base leading-snug">
                {{ sug.text }}
              </span>
              <span
                v-if="sug.subtitle"
                class="text-xs text-muted-foreground truncate leading-tight mt-0.5"
              >
                {{ sug.subtitle }}
              </span>
            </div>
          </div>

          <!-- Right: Thumbnail (Avatar / Video) -->
          <div v-if="sug.thumbnail" class="shrink-0 ml-3">
            <img
              :src="sug.thumbnail"
              :alt="sug.text"
              class="h-10 w-16 rounded object-cover shadow-xs border border-border/30 bg-muted"
              loading="lazy"
            />
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>
