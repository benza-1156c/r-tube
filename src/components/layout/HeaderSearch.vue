<script setup lang="ts">
import { ref, watch } from "vue";
import { useRouter, useRoute } from "vue-router";
import { Search, X } from "@lucide/vue";
import { useDebounceFn, onClickOutside } from "@vueuse/core";
import { commands } from "@/bindings";

const router = useRouter();
const route = useRoute();

const searchQuery = ref("");
const suggestions = ref<string[]>([]);
const showSuggestions = ref(false);
const selectedIndex = ref(-1);
const isInputFocused = ref(false);
const searchContainerRef = ref<HTMLElement | null>(null);

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

function selectSuggestion(sug: string) {
  handleSearch(sug);
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
    searchQuery.value = suggestions.value[selectedIndex.value];
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    selectedIndex.value =
      selectedIndex.value <= 0
        ? suggestions.value.length - 1
        : selectedIndex.value - 1;
    searchQuery.value = suggestions.value[selectedIndex.value];
  } else if (e.key === "Enter") {
    e.preventDefault();
    if (selectedIndex.value >= 0 && suggestions.value[selectedIndex.value]) {
      handleSearch(suggestions.value[selectedIndex.value]);
    } else {
      handleSearch();
    }
  } else if (e.key === "Escape") {
    fetchSuggestions.cancel();
    showSuggestions.value = false;
    suggestions.value = [];
    selectedIndex.value = -1;
    if (document.activeElement instanceof HTMLElement) {
      document.activeElement.blur();
    }
  }
}
</script>

<template>
  <div
    ref="searchContainerRef"
    class="relative flex flex-1 max-w-xl mx-4 items-center"
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

    <!-- Suggestions Dropdown -->
    <div
      v-if="showSuggestions && suggestions.length > 0"
      class="absolute top-full left-0 right-0 mt-1.5 z-50 rounded-xl border border-border/40 bg-popover/95 text-popover-foreground shadow-2xl backdrop-blur-md overflow-hidden py-1.5"
    >
      <div
        v-for="(sug, idx) in suggestions"
        :key="sug"
        class="flex items-center gap-3 px-3.5 py-2 text-sm cursor-pointer transition-colors"
        :class="
          selectedIndex === idx
            ? 'bg-accent text-accent-foreground font-medium'
            : 'hover:bg-accent/60'
        "
        @mousedown.prevent="selectSuggestion(sug)"
      >
        <Search class="h-4 w-4 text-muted-foreground shrink-0" />
        <span class="truncate">{{ sug }}</span>
      </div>
    </div>
  </div>
</template>
