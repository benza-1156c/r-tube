<script setup lang="ts">
import { useRoute } from "vue-router";
import {
  Compass,
  Flame,
  CirclePlay,
  History,
  Settings,
} from "@lucide/vue";

const route = useRoute();

const items = [
  { title: "Explore", path: "/", icon: Compass },
  { title: "Trending", path: "/trending", icon: Flame },
  { title: "Subscriptions", path: "/subscriptions", icon: CirclePlay },
  { title: "History", path: "/history", icon: History },
  { title: "Settings", path: "/settings", icon: Settings },
];

function isActive(path: string): boolean {
  return path === "/" ? route.path === "/" : route.path.startsWith(path);
}
</script>

<template>
  <nav
    class="fixed inset-x-0 bottom-0 z-40 border-t border-border/40 bg-background/95 backdrop-blur-md"
    aria-label="Primary"
  >
    <div
      class="mx-auto flex h-16 max-w-2xl items-stretch px-1 pb-[var(--app-safe-bottom)]"
    >
      <RouterLink
        v-for="item in items"
        :key="item.path"
        :to="item.path"
        class="flex flex-1 flex-col items-center justify-center gap-0.5 rounded-lg text-[10px] font-medium transition-colors"
        :class="
          isActive(item.path)
            ? 'text-primary'
            : 'text-muted-foreground hover:text-foreground'
        "
        :aria-current="isActive(item.path) ? 'page' : undefined"
      >
        <component
          :is="item.icon"
          class="h-5 w-5"
          :class="isActive(item.path) ? 'stroke-[2.2]' : 'stroke-[1.8]'"
        />
        <span class="truncate">{{ item.title }}</span>
      </RouterLink>
    </div>
  </nav>
</template>