<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useRoute, useRouter } from "vue-router";
import {
  Compass,
  Flame,
  CirclePlay,
  SquarePlay,
  History,
  Clock,
  Heart,
  Radio,
  Settings,
  AlertCircle,
} from "@lucide/vue";
import { commands } from "@/bindings";

const route = useRoute();
const router = useRouter();

const currentPath = computed(() => route.path);

function navigateTo(path: string) {
  if (path.startsWith("#")) {
    return;
  }
  router.push(path);
}

const mainItems = [
  { title: "Explore", path: "/", icon: Compass },
  { title: "Trending", path: "/trending", icon: Flame },
  { title: "Subscription", path: "/subscriptions", icon: CirclePlay },
];

const libraryItems = [
  { title: "Library", path: "/history", icon: SquarePlay },
  { title: "History", path: "/history", icon: History },
  { title: "Watch later", path: "/history?tab=watch-later", icon: Clock },
  { title: "Likes videos", path: "/history?tab=liked", icon: Heart },
];

interface ChannelItem {
  name: string;
  avatarUrl: string;
  isLive: boolean;
  path: string;
}

const subscriptions = ref<ChannelItem[]>([]);
const isLoadingSubscriptions = ref(false);
const isSignedIn = ref(false);

async function loadRealSubscriptions() {
  isLoadingSubscriptions.value = true;
  try {
    const profileRes = await commands.getActiveProfile();
    isSignedIn.value = Boolean(profileRes.status === "ok" && profileRes.data);

    if (isSignedIn.value) {
      const feedRes = await commands.getSubscriptionsFeed();
      if (feedRes.status === "ok" && feedRes.data?.videos) {
        const map = new Map<string, ChannelItem>();
        for (const v of feedRes.data.videos) {
          if (v.channelName && !map.has(v.channelName)) {
            map.set(v.channelName, {
              name: v.channelName,
              avatarUrl: v.channelAvatarUrl || "",
              isLive: Boolean(v.isLive),
              path: "/subscriptions",
            });
          }
        }
        subscriptions.value = Array.from(map.values());
      }
    } else {
      subscriptions.value = [];
    }
  } catch (err) {
    console.error("Failed to load real subscriptions:", err);
  } finally {
    isLoadingSubscriptions.value = false;
  }
}

onMounted(() => {
  loadRealSubscriptions();
});

// ── 4. More from YouTube Section ──
const moreItems = [
  { title: "Settings", path: "/settings", icon: Settings },
  { title: "Send feedback", path: "#feedback", icon: AlertCircle },
];
</script>

<template>
  <Sidebar
    collapsible="icon"
    class="top-8 border-r border-sidebar-border flex flex-col"
  >
    <!-- ── FIXED TOP AREA: MAIN & LIBRARY (ไม่เลื่อน) ── -->
    <SidebarHeader class="p-0 gap-0 shrink-0 pt-2">
      <!-- Section 1: Main Navigation -->
      <SidebarGroup class="py-1">
        <SidebarGroupContent>
          <SidebarMenu class="space-y-0.5">
            <SidebarMenuItem v-for="item in mainItems" :key="item.path">
              <SidebarMenuButton
                :is-active="currentPath === item.path"
                :tooltip="item.title"
                class="gap-4 h-10 px-3 rounded-lg transition-colors cursor-pointer text-sidebar-foreground"
                @click="navigateTo(item.path)"
              >
                <component
                  :is="item.icon"
                  class="h-4.5 w-4.5 shrink-0 stroke-[1.8]"
                />
                <span class="font-medium text-sm truncate">{{
                  item.title
                }}</span>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarGroupContent>
      </SidebarGroup>

      <SidebarSeparator class="my-1.5 border-border/40" />

      <!-- Section 2: Library -->
      <SidebarGroup class="py-1">
        <SidebarGroupContent>
          <SidebarMenu class="space-y-0.5">
            <SidebarMenuItem v-for="item in libraryItems" :key="item.title">
              <SidebarMenuButton
                :is-active="currentPath === item.path"
                :tooltip="item.title"
                class="gap-4 h-10 px-3 rounded-lg transition-colors cursor-pointer text-sidebar-foreground"
                @click="navigateTo(item.path)"
              >
                <component
                  :is="item.icon"
                  class="h-4.5 w-4.5 shrink-0 stroke-[1.8]"
                />
                <span class="font-medium text-sm truncate">{{
                  item.title
                }}</span>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarGroupContent>
      </SidebarGroup>

      <SidebarSeparator class="my-1.5 border-border/40" />
    </SidebarHeader>

    <!-- ── SCROLLABLE MIDDLE AREA: SUBSCRIPTIONS ONLY (เลื่อนเฉพาะตรงนี้) ── -->
    <SidebarContent
      class="py-0 gap-0 overflow-y-auto overflow-x-hidden flex-1 min-h-0"
    >
      <SidebarGroup class="py-1">
        <SidebarGroupLabel
          class="text-[11px] uppercase tracking-wider text-muted-foreground font-semibold px-3 py-1.5"
        >
          Subscriptions
        </SidebarGroupLabel>
        <SidebarGroupContent>
          <SidebarMenu class="space-y-0.5">
            <!-- Loading state -->
            <div
              v-if="isLoadingSubscriptions"
              class="px-3 py-3 text-xs text-muted-foreground text-center"
            >
              Loading channels...
            </div>

            <!-- Empty / Not signed in state (ไม่มี mock data) -->
            <div
              v-else-if="subscriptions.length === 0"
              class="px-3 py-4 text-xs text-muted-foreground text-center"
            >
              <p v-if="!isSignedIn">Sign in to see channels</p>
              <p v-else>No subscriptions yet</p>
            </div>

            <!-- Real Channel Items from YouTube -->
            <SidebarMenuItem v-for="sub in subscriptions" :key="sub.name">
              <SidebarMenuButton
                :tooltip="sub.name"
                class="gap-3.5 h-10 px-3 rounded-lg transition-colors cursor-pointer text-sidebar-foreground group"
                @click="navigateTo(sub.path)"
              >
                <!-- Channel Avatar -->
                <img
                  v-if="sub.avatarUrl"
                  :src="sub.avatarUrl"
                  :alt="sub.name"
                  class="h-6 w-6 rounded-full object-cover shrink-0"
                />
                <div
                  v-else
                  class="h-6 w-6 rounded-full bg-primary/15 text-primary font-bold text-xs flex items-center justify-center shrink-0"
                >
                  {{ sub.name.slice(0, 1) }}
                </div>

                <!-- Channel Name -->
                <span class="font-medium text-sm truncate flex-1 text-left">
                  {{ sub.name }}
                </span>

                <!-- Live Indicator -->
                <span
                  v-if="sub.isLive"
                  class="shrink-0 flex items-center justify-center text-red-500"
                  title="Live"
                >
                  <Radio class="h-3.5 w-3.5 animate-pulse" />
                </span>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarGroupContent>
      </SidebarGroup>
    </SidebarContent>

    <!-- ── FIXED BOTTOM AREA: MORE FROM YOUTUBE (ไม่เลื่อน) ── -->
    <SidebarFooter class="p-2 gap-0 border-t border-border/40 shrink-0 mt-auto">
      <SidebarGroup class="p-0">
        <SidebarGroupLabel
          class="text-[11px] uppercase tracking-wider text-muted-foreground font-semibold px-3 py-1"
        >
          More from YouTube
        </SidebarGroupLabel>
        <SidebarGroupContent>
          <SidebarMenu class="space-y-0.5">
            <SidebarMenuItem v-for="item in moreItems" :key="item.path">
              <SidebarMenuButton
                :is-active="currentPath === item.path"
                :tooltip="item.title"
                class="gap-4 h-10 px-3 rounded-lg transition-colors cursor-pointer text-sidebar-foreground"
                @click="navigateTo(item.path)"
              >
                <component
                  :is="item.icon"
                  class="h-4.5 w-4.5 shrink-0 stroke-[1.8]"
                />
                <span class="font-medium text-sm truncate">{{
                  item.title
                }}</span>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarGroupContent>
      </SidebarGroup>
    </SidebarFooter>
  </Sidebar>
</template>
