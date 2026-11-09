<script setup lang="ts">
import { History } from "@lucide/vue";
</script>

<template>
  <div class="p-6 max-w-7xl mx-auto flex flex-col gap-4">
    <div class="flex items-center gap-3 border-b border-border/40 pb-4">
      <div class="flex h-10 w-10 items-center justify-center rounded-xl bg-blue-500/10 text-blue-500">
        <History class="h-6 w-6" />
      </div>
      <div>
        <h1 class="text-xl font-bold text-foreground">History</h1>
        <p class="text-xs text-muted-foreground">Videos you have previously watched</p>
      </div>
    </div>
    <div class="rounded-xl border border-dashed border-border p-12 text-center text-sm text-muted-foreground">
      Your watch history will be saved and displayed here
    </div>
  </div>
</template>
