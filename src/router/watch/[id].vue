<script setup lang="ts">
import { ArrowLeft, ThumbsUp, Share2, Bookmark, CheckCircle2, Play } from "@lucide/vue";

const route = useRoute();
const router = useRouter();

const videoId = (route.params as Record<string, any>)?.id || "dQw4w9WgXcQ";

const relatedVideos = [
  {
    id: "kJQP7kiw5Fk",
    title: "Lofi Hip Hop Radio - Beats to Relax/Study to",
    thumbnailUrl: "https://images.unsplash.com/photo-1518709268805-4e9042af9f23?w=400&auto=format&fit=crop&q=60",
    channelName: "ChillHop Music",
    views: "4.8K watching",
    duration: "LIVE",
  },
  {
    id: "fJ9rUzIMcZQ",
    title: "Understanding YouTube InnerTube Protocol and Session Tokens in Depth",
    thumbnailUrl: "https://images.unsplash.com/photo-1550745165-9bc0b252726f?w=400&auto=format&fit=crop&q=60",
    channelName: "System Architecture",
    views: "89K views",
    duration: "25:10",
  },
  {
    id: "7wtfhZwyrcc",
    title: "Top 10 VS Code & NeoVim Productivity Shortcuts You Never Knew",
    thumbnailUrl: "https://images.unsplash.com/photo-1526374965328-7f61d4dc18c5?w=400&auto=format&fit=crop&q=60",
    channelName: "DevTips Daily",
    views: "310K views",
    duration: "12:15",
  },
];
</script>

<template>
  <div class="flex flex-col lg:flex-row gap-6 p-6 max-w-7xl mx-auto">
    <!-- Left Column: Video Player & Details -->
    <div class="flex-1 flex flex-col gap-4">
      <!-- Back Navigation Header -->
      <div class="flex items-center gap-3">
        <Button
          variant="outline"
          size="sm"
          class="gap-1.5 rounded-full"
          @click="router.push('/')"
        >
          <ArrowLeft class="h-4 w-4" />
          <span>Back to Home (Show Sidebar)</span>
        </Button>

        <span class="text-xs px-2.5 py-1 rounded-full bg-primary/10 text-primary font-medium">
          Sidebar is hidden here via meta: { hideSidebar: true }
        </span>
      </div>

      <!-- Video Player Frame Placeholder -->
      <div class="relative aspect-video w-full rounded-2xl overflow-hidden bg-black shadow-lg flex items-center justify-center">
        <div class="flex flex-col items-center gap-3 text-white">
          <div class="flex h-16 w-16 items-center justify-center rounded-full bg-red-600 shadow-md">
            <Play class="h-8 w-8 fill-white translate-x-0.5" />
          </div>
          <p class="text-sm font-medium text-white/80">
            Playing Video ID: {{ videoId }}
          </p>
        </div>
      </div>

      <!-- Video Title -->
      <h1 class="text-xl font-bold text-foreground">
        Building Modern Desktop Apps with Tauri 2.0, Rust, and Vue 3
      </h1>

      <!-- Channel & Action Buttons Row -->
      <div class="flex flex-wrap items-center justify-between gap-4 border-b border-border/40 pb-4">
        <div class="flex items-center gap-3">
          <img
            src="https://images.unsplash.com/photo-1534528741775-53994a69daeb?w=100&auto=format&fit=crop&q=60"
            alt="Channel Avatar"
            class="h-10 w-10 rounded-full object-cover"
          />
          <div>
            <div class="flex items-center gap-1 font-semibold text-sm text-foreground">
              <span>Rust & Web Studio</span>
              <CheckCircle2 class="h-3.5 w-3.5 text-muted-foreground fill-muted-foreground/30" />
            </div>
            <p class="text-xs text-muted-foreground">240K subscribers</p>
          </div>
          <Button class="ml-3 rounded-full bg-foreground text-background hover:bg-foreground/90 font-medium px-4 h-9 text-xs">
            Subscribe
          </Button>
        </div>

        <div class="flex items-center gap-2">
          <Button variant="secondary" size="sm" class="rounded-full gap-1.5 h-9">
            <ThumbsUp class="h-4 w-4" />
            <span>12K</span>
          </Button>
          <Button variant="secondary" size="sm" class="rounded-full gap-1.5 h-9">
            <Share2 class="h-4 w-4" />
            <span>Share</span>
          </Button>
          <Button variant="secondary" size="sm" class="rounded-full gap-1.5 h-9">
            <Bookmark class="h-4 w-4" />
            <span>Save</span>
          </Button>
        </div>
      </div>

      <!-- Description Box -->
      <div class="rounded-xl bg-muted/40 p-4 text-sm leading-relaxed text-foreground/90">
        <div class="font-semibold mb-1 flex items-center gap-2 text-xs text-muted-foreground">
          <span>142,390 views</span>
          <span>•</span>
          <span>2 days ago</span>
        </div>
        <p>
          Learn how to build production-grade desktop applications with Tauri v2, Vue 3, SQLite (SeaORM), and the YouTube InnerTube API for video streaming and playlist management.
        </p>
      </div>
    </div>

    <!-- Right Column: Up Next / Related Videos -->
    <div class="w-full lg:w-96 flex flex-col gap-3">
      <h2 class="font-semibold text-sm text-foreground">Up next</h2>
      <div
        v-for="item in relatedVideos"
        :key="item.id"
        class="flex gap-3 group cursor-pointer hover:bg-muted/40 p-2 rounded-xl transition-colors"
        @click="router.push(`/watch/${item.id}`)"
      >
        <div class="relative aspect-video w-40 shrink-0 overflow-hidden rounded-lg bg-muted">
          <img
            :src="item.thumbnailUrl"
            :alt="item.title"
            class="h-full w-full object-cover group-hover:scale-105 transition-transform"
          />
          <span class="absolute bottom-1 right-1 rounded-sm bg-black/80 px-1 text-[10px] text-white">
            {{ item.duration }}
          </span>
        </div>
        <div class="flex flex-col min-w-0">
          <h3 class="line-clamp-2 text-xs font-semibold leading-snug text-foreground group-hover:text-primary">
            {{ item.title }}
          </h3>
          <p class="text-[11px] text-muted-foreground mt-1 truncate">{{ item.channelName }}</p>
          <p class="text-[11px] text-muted-foreground/80">{{ item.views }}</p>
        </div>
      </div>
    </div>
  </div>
</template>
