<script setup lang="ts">
import { useRoute } from "vue-router";
import { computed, onMounted, onBeforeUnmount } from "vue";
import GlobalVideoPlayer from "@/components/player/GlobalVideoPlayer.vue";
import { useMiniPlayerDraggable } from "@/composables/useMiniPlayerDraggable";

const route = useRoute();

const shouldHideSidebar = computed(() => Boolean(route.meta.hideSidebar));
const shouldHideHeader = computed(() => Boolean(route.meta.hideHeader));

const { position, hostStyle, clampToViewport } = useMiniPlayerDraggable();

onMounted(() => {
  window.addEventListener("resize", clampToViewport);
});

onBeforeUnmount(() => {
  window.removeEventListener("resize", clampToViewport);
});
</script>

<template>
  <SidebarProvider
    :default-open="true"
    class="min-h-screen w-full bg-background text-foreground"
  >
    <AppSidebar v-if="!shouldHideSidebar" />

    <SidebarInset class="flex flex-col flex-1 min-w-0 overflow-hidden">
      <AppHeader v-if="!shouldHideHeader" :hide-sidebar="shouldHideSidebar" />

      <main class="flex-1 overflow-y-auto">
        <router-view v-slot="{ Component }">
          <keep-alive exclude="WatchPage">
            <component :is="Component" />
          </keep-alive>
        </router-view>
      </main>
    </SidebarInset>

    <div
      id="mini-player-host"
      class="pointer-events-none fixed z-50 w-96 max-w-[calc(100vw-2.5rem)]"
      :class="{ 'bottom-5 right-5': !position }"
      :style="hostStyle"
    >
      <GlobalVideoPlayer />
    </div>
  </SidebarProvider>
</template>
