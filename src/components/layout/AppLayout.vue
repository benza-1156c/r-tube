<script setup lang="ts">
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
        <router-view />
      </main>
    </SidebarInset>
  </SidebarProvider>
</template>
