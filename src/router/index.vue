<script setup lang="ts">
import type { VideoItem } from "@/components/video/VideoCard.vue";

const isLoading = ref(false);

const mockVideos: VideoItem[] = [
  {
    id: "dQw4w9WgXcQ",
    title: "Building Modern Desktop Apps with Tauri 2.0, Rust, and Vue 3",
    thumbnailUrl:
      "https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?w=800&auto=format&fit=crop&q=60",
    duration: "18:42",
    channelName: "Rust & Web Studio",
    channelAvatarUrl:
      "https://images.unsplash.com/photo-1534528741775-53994a69daeb?w=100&auto=format&fit=crop&q=60",
    isVerified: true,
    views: "142K views",
    publishedAt: "2 days ago",
  },
  {
    id: "kJQP7kiw5Fk",
    title: "Lofi Hip Hop Radio - Beats to Relax/Study to [24/7 Live Stream]",
    thumbnailUrl:
      "https://images.unsplash.com/photo-1518709268805-4e9042af9f23?w=800&auto=format&fit=crop&q=60",
    duration: "LIVE",
    channelName: "ChillHop Music",
    channelAvatarUrl:
      "https://images.unsplash.com/photo-1535713875002-d1d0cf377fde?w=100&auto=format&fit=crop&q=60",
    isVerified: true,
    views: "4.8K watching",
    publishedAt: "Started streaming",
  },
  {
    id: "fJ9rUzIMcZQ",
    title:
      "Understanding YouTube InnerTube Protocol and Session Tokens in Depth",
    thumbnailUrl:
      "https://images.unsplash.com/photo-1550745165-9bc0b252726f?w=800&auto=format&fit=crop&q=60",
    duration: "25:10",
    channelName: "System Architecture",
    channelAvatarUrl:
      "https://images.unsplash.com/photo-1570295999919-56ceb5ecca61?w=100&auto=format&fit=crop&q=60",
    isVerified: false,
    views: "89K views",
    publishedAt: "5 days ago",
  },
  {
    id: "7wtfhZwyrcc",
    title: "Top 10 VS Code & NeoVim Productivity Shortcuts You Never Knew",
    thumbnailUrl:
      "https://images.unsplash.com/photo-1526374965328-7f61d4dc18c5?w=800&auto=format&fit=crop&q=60",
    duration: "12:15",
    channelName: "DevTips Daily",
    channelAvatarUrl:
      "https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?w=100&auto=format&fit=crop&q=60",
    isVerified: true,
    views: "310K views",
    publishedAt: "1 week ago",
  },
  {
    id: "L_LUpnjgPso",
    title:
      "SeaORM & SQLite in Rust: Clean Data Persistence and Migrations Guide",
    thumbnailUrl:
      "https://images.unsplash.com/photo-1555066931-4365d14bab8c?w=800&auto=format&fit=crop&q=60",
    duration: "34:50",
    channelName: "Backend Mastery",
    channelAvatarUrl:
      "https://images.unsplash.com/photo-1494790108377-be9c29b29330?w=100&auto=format&fit=crop&q=60",
    isVerified: true,
    views: "45K views",
    publishedAt: "2 weeks ago",
  },
  {
    id: "C0DPdy98e4c",
    title: "Why Tailwind CSS v4 and Reka UI are the Ultimate Tech Stack",
    thumbnailUrl:
      "https://images.unsplash.com/photo-1507238691740-187a5b1d37b8?w=800&auto=format&fit=crop&q=60",
    duration: "16:04",
    channelName: "Design & Code",
    channelAvatarUrl:
      "https://images.unsplash.com/photo-1522075469751-3a6694fb2f61?w=100&auto=format&fit=crop&q=60",
    isVerified: false,
    views: "198K views",
    publishedAt: "3 weeks ago",
  },
  {
    id: "9bZkp7q19f0",
    title: "Deep Focus Music - Binaural Beats for High Productivity",
    thumbnailUrl:
      "https://images.unsplash.com/photo-1511671782779-c97d3d27a1d4?w=800&auto=format&fit=crop&q=60",
    duration: "3:00:00",
    channelName: "Mind Waves",
    channelAvatarUrl:
      "https://images.unsplash.com/photo-1534528741775-53994a69daeb?w=100&auto=format&fit=crop&q=60",
    isVerified: true,
    views: "1.2M views",
    publishedAt: "1 month ago",
  },
  {
    id: "OPf0YbXqDm0",
    title: "How GPU Rendering and WebGPU Work Behind the Scenes",
    thumbnailUrl:
      "https://images.unsplash.com/photo-1542751371-adc38448a05e?w=800&auto=format&fit=crop&q=60",
    duration: "21:30",
    channelName: "Graphics Pipeline",
    channelAvatarUrl:
      "https://images.unsplash.com/photo-1500648767791-00dcc994a43e?w=100&auto=format&fit=crop&q=60",
    isVerified: true,
    views: "275K views",
    publishedAt: "1 month ago",
  },
];

function onCategorySelect(category: string) {
  console.log("Selected category:", category);
}
</script>

<template>
  <div class="flex flex-col min-h-full">
    <!-- Category Filter Chips -->
    <CategoryChips @select="onCategorySelect" />

    <!-- Video Grid Section -->
    <VideoGrid :videos="mockVideos" :loading="isLoading" />
  </div>
</template>
