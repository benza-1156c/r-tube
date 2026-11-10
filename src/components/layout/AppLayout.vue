<script setup lang="ts">
import { useRoute } from "vue-router";
import { computed } from "vue";
import GlobalVideoPlayer from "@/components/player/GlobalVideoPlayer.vue";

const route = useRoute();

const shouldHideSidebar = computed(() => Boolean(route.meta.hideSidebar));
const shouldHideHeader = computed(() => Boolean(route.meta.hideHeader));
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
      class="pointer-events-none fixed bottom-5 right-5 z-50 w-96 max-w-[calc(100vw-2.5rem)]"
    >
      <GlobalVideoPlayer />
    </div>
  </SidebarProvider>
</template>
