<script setup lang="ts">
import { Tv } from "@lucide/vue";
</script>

<template>
  <div class="p-6 max-w-7xl mx-auto flex flex-col gap-4">
    <div class="flex items-center gap-3 border-b border-border/40 pb-4">
      <div class="flex h-10 w-10 items-center justify-center rounded-xl bg-red-500/10 text-red-500">
        <Tv class="h-6 w-6" />
      </div>
      <div>
        <h1 class="text-xl font-bold text-foreground">Subscriptions</h1>
        <p class="text-xs text-muted-foreground">Latest videos from channels you follow</p>
      </div>
    </div>
    <div class="rounded-xl border border-dashed border-border p-12 text-center text-sm text-muted-foreground">
      Videos from your subscribed channels will appear here (powered by InnerTube get_subscriptions_feed)
    </div>
  </div>
</template>
