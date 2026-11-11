<script setup lang="ts">
import { useRoute } from "vue-router";
import { computed, defineAsyncComponent, onMounted, onBeforeUnmount } from "vue";
import { previousPath } from "@/router";
import AppTitleBar from "@/components/layout/AppTitleBar.vue";
import PlaybackModeOsd from "@/components/layout/PlaybackModeOsd.vue";
import GlobalAudioPlayer from "@/components/audio/GlobalAudioPlayer.vue";
import { useMiniPlayerDraggable } from "@/composables/useMiniPlayerDraggable";
import { usePlayer } from "@/composables/usePlayer";
import { useMediaSession } from "@/composables/useMediaSession";

// The video player (Video.js + hls.js) is the heaviest dependency in the app;
// loading it on demand keeps it out of the startup bundle.
const GlobalVideoPlayer = defineAsyncComponent(
  () => import("@/components/player/GlobalVideoPlayer.vue"),
);

const route = useRoute();
const player = usePlayer();

useMediaSession();

const shouldHideSidebar = computed(() => Boolean(route.meta.hideSidebar));
const shouldHideHeader = computed(() => Boolean(route.meta.hideHeader));

const { position, hostStyle, clampToViewport } = useMiniPlayerDraggable();

const isPlayerRoute = (path: string): boolean =>
  path.startsWith("/watch/") || path.startsWith("/listen/");

/** Player → player swaps are instant; everything else uses the `page` fade. */
function pageTransitionName(to: string, from: string): string {
  return isPlayerRoute(to) && isPlayerRoute(from) ? "" : "page";
}

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

        <main class="flex-1 overflow-y-auto overflow-x-hidden">
          <router-view v-slot="{ Component, route: currentRoute }">
            <Transition
              :name="pageTransitionName(currentRoute.path, previousPath)"
              mode="out-in"
            >
              <keep-alive exclude="WatchPage,ListenPage">
                <component :is="Component" :key="currentRoute.fullPath" />
              </keep-alive>
            </Transition>
          </router-view>
        </main>
      </SidebarInset>

      <div
        id="mini-player-host"
        class="pointer-events-none fixed z-50 w-96 max-w-[calc(100vw-2.5rem)]"
        :class="{ 'bottom-5 right-5': !position }"
        :style="hostStyle"
      >
        <GlobalVideoPlayer v-if="player.isOpen.value" />
      </div>

      <!-- Audio-only engine: lives outside the router view so a track keeps
           playing while the user browses elsewhere. -->
      <GlobalAudioPlayer />

      <!-- Transient confirmation when the playback mode is switched. -->
      <PlaybackModeOsd />
    </SidebarProvider>
  </div>
</template>
