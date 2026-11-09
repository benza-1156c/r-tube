<script setup lang="ts">
import { Flame } from "@lucide/vue";
</script>

<template>
  <div class="p-6 max-w-7xl mx-auto flex flex-col gap-4">
    <div class="flex items-center gap-3 border-b border-border/40 pb-4">
      <div class="flex h-10 w-10 items-center justify-center rounded-xl bg-orange-500/10 text-orange-500">
        <Flame class="h-6 w-6" />
      </div>
      <div>
        <h1 class="text-xl font-bold text-foreground">Trending</h1>
        <p class="text-xs text-muted-foreground">Most popular videos right now</p>
      </div>
    </div>
    <div class="rounded-xl border border-dashed border-border p-12 text-center text-sm text-muted-foreground">
      Trending videos feed will appear here (powered by InnerTube get_trending)
    </div>
  </div>
</template>
