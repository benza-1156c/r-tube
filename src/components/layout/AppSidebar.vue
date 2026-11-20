<script setup lang="ts">
import { useRoute, useRouter } from "vue-router";
import { ref, computed, onMounted } from "vue";
import {
  Compass,
  Flame,
  CirclePlay,
  History,
  Radio,
  Settings,
  UserRoundExclamationPoint,
} from "@lucide/vue";
import { commands } from "@/bindings";
import { useIsMobileDevice } from "@/composables/useIsMobileDevice";

const route = useRoute();
const router = useRouter();
const isMobile = useIsMobileDevice();

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
  { title: "Subscriptions", path: "/subscriptions", icon: CirclePlay },
];

const libraryItems = [
  { title: "History", path: "/history", icon: History },
];

interface ChannelItem {
  name: string;
  channelId: string | null;
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
      const feedRes = await commands.getSubscriptionsFeed(null);
      if (feedRes.status === "ok" && feedRes.data?.videos) {
        const map = new Map<string, ChannelItem>();
        for (const v of feedRes.data.videos) {
          const key = v.channelId || v.channelName;
          if (v.channelName && !map.has(key)) {
            map.set(key, {
              name: v.channelName,
              channelId: v.channelId ?? null,
              avatarUrl: v.channelAvatarUrl || "",
              isLive: Boolean(v.isLive),
              path: v.channelId
                ? `/channel/${v.channelId}`
                : "/subscriptions",
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
const moreItems = [{ title: "Settings", path: "/settings", icon: Settings }];
</script>

<template>
  <!-- `top-8` leaves room for the title bar, which mobile does not render. -->
  <Sidebar
    collapsible="icon"
    :class="[
      'border-r border-sidebar-border flex flex-col',
      isMobile ? '' : 'top-8 h-[calc(100svh-2rem)]',
    ]"
  >
    <!-- ── FIXED TOP AREA: ── -->
    <SidebarHeader
      class="p-0 gap-0 shrink-0 pt-[max(0.5rem,var(--app-safe-top))]"
    >
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
            <template v-if="isLoadingSubscriptions">
              <SidebarMenuItem
                v-for="i in 3"
                :key="i"
                class="group-data-[collapsible=icon]:hidden"
              >
                <SidebarMenuSkeleton show-icon />
              </SidebarMenuItem>
            </template>

            <!-- Empty / Not signed in state -->
            <SidebarMenuItem
              v-else-if="subscriptions.length === 0"
              class="group-data-[collapsible=icon]:hidden"
            >
              <div
                class="px-3 py-4 text-xs text-muted-foreground text-center flex flex-col items-center justify-center gap-2"
              >
                <template v-if="!isSignedIn">
                  <UserRoundExclamationPoint
                    class="h-7 w-7 text-muted-foreground/70"
                  />
                  <p>Sign in to see your subscriptions</p>
                  <button
                    type="button"
                    class="mt-0.5 rounded-full bg-primary px-3 py-1.5 text-[11px] font-semibold text-primary-foreground transition-opacity hover:opacity-90"
                    @click="navigateTo('/login')"
                  >
                    Sign in
                  </button>
                </template>

                <p v-else>No subscriptions yet</p>
              </div>
            </SidebarMenuItem>

            <!-- Real Channel Items from YouTube -->
            <SidebarMenuItem v-for="sub in subscriptions" :key="sub.name">
              <SidebarMenuButton
                :tooltip="sub.name"
                class="gap-3.5 h-10 px-3 rounded-lg transition-colors cursor-pointer text-sidebar-foreground"
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
    <SidebarFooter
      class="p-2 pb-[max(0.5rem,var(--app-safe-bottom))] gap-0 border-t border-border/40 shrink-0 mt-auto"
    >
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
