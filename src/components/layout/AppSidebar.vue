<script setup lang="ts">
import {
  Home,
  Flame,
  Tv,
  History,
  ThumbsUp,
  Settings,
  FolderHeart,
} from "@lucide/vue";

const route = useRoute();
const router = useRouter();

const mainItems = [
  { title: "Home", path: "/", icon: Home },
  { title: "Trending", path: "/trending", icon: Flame },
  { title: "Subscriptions", path: "/subscriptions", icon: Tv },
];

const libraryItems = [
  { title: "History", path: "/history", icon: History },
  { title: "Liked Videos", path: "/history", icon: ThumbsUp },
  { title: "Playlists", path: "/history", icon: FolderHeart },
];

const settingsItems = [
  { title: "Settings", path: "/settings", icon: Settings },
];

const currentPath = computed(() => route.path);

function navigateTo(path: string) {
  router.push(path);
}
</script>

<template>
  <Sidebar collapsible="icon" class="border-r border-border/40">
    <SidebarContent class="pt-2">
      <!-- Main Navigation Group -->
      <SidebarGroup>
        <SidebarGroupContent>
          <SidebarMenu>
            <SidebarMenuItem v-for="item in mainItems" :key="item.path">
              <SidebarMenuButton
                :is-active="currentPath === item.path"
                :tooltip="item.title"
                class="gap-3 transition-colors hover:bg-accent cursor-pointer"
                @click="navigateTo(item.path)"
              >
                <component :is="item.icon" class="h-4 w-4" />
                <span class="font-medium text-sm">{{ item.title }}</span>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarGroupContent>
      </SidebarGroup>

      <SidebarSeparator class="my-2" />

      <!-- Library Group -->
      <SidebarGroup>
        <SidebarGroupLabel class="text-xs uppercase tracking-wider text-muted-foreground font-semibold px-2">
          You
        </SidebarGroupLabel>
        <SidebarGroupContent>
          <SidebarMenu>
            <SidebarMenuItem v-for="item in libraryItems" :key="item.title">
              <SidebarMenuButton
                :is-active="currentPath === item.path"
                :tooltip="item.title"
                class="gap-3 transition-colors hover:bg-accent cursor-pointer"
                @click="navigateTo(item.path)"
              >
                <component :is="item.icon" class="h-4 w-4" />
                <span class="font-medium text-sm">{{ item.title }}</span>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarGroupContent>
      </SidebarGroup>

      <SidebarSeparator class="my-2" />

      <!-- Settings Group -->
      <SidebarGroup class="mt-auto">
        <SidebarGroupContent>
          <SidebarMenu>
            <SidebarMenuItem v-for="item in settingsItems" :key="item.path">
              <SidebarMenuButton
                :is-active="currentPath === item.path"
                :tooltip="item.title"
                class="gap-3 transition-colors hover:bg-accent cursor-pointer"
                @click="navigateTo(item.path)"
              >
                <component :is="item.icon" class="h-4 w-4" />
                <span class="font-medium text-sm">{{ item.title }}</span>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarGroupContent>
      </SidebarGroup>
    </SidebarContent>
  </Sidebar>
</template>
