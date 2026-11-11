<script setup lang="ts">
import { useRoute } from "vue-router";
import { computed, onMounted, onBeforeUnmount } from "vue";
import AppTitleBar from "@/components/layout/AppTitleBar.vue";
import PlaybackModeOsd from "@/components/layout/PlaybackModeOsd.vue";
import GlobalVideoPlayer from "@/components/player/GlobalVideoPlayer.vue";
import GlobalAudioPlayer from "@/components/audio/GlobalAudioPlayer.vue";
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
  <div
    class="flex flex-col h-screen w-full overflow-hidden bg-background text-foreground"
  >
    <!-- Dedicated Top Titlebar separated from everything -->
    <AppTitleBar />

    <SidebarProvider
      :default-open="true"
      class="h-[calc(100svh-2rem)]! flex-1 min-h-0 w-full overflow-hidden"
    >
      <AppSidebar v-if="!shouldHideSidebar" />

      <SidebarInset class="flex flex-col flex-1 min-w-0 overflow-hidden">
        <AppHeader v-if="!shouldHideHeader" :hide-sidebar="shouldHideSidebar" />

        <main class="flex-1 overflow-y-auto">
          <router-view v-slot="{ Component, route: currentRoute }">
            <keep-alive exclude="WatchPage,ListenPage">
              <component :is="Component" :key="currentRoute.fullPath" />
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

      <!-- Audio-only engine: lives outside the router view so a track keeps
           playing while the user browses elsewhere. -->
      <GlobalAudioPlayer />

      <!-- Transient confirmation when the playback mode is switched. -->
      <PlaybackModeOsd />
    </SidebarProvider>
  </div>
</template>
