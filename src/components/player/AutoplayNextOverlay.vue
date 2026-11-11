<script setup lang="ts">
import type { VideoItem_Deserialize } from "@/bindings";
import {
  Card,
  CardHeader,
  CardTitle,
  CardContent,
  CardFooter,
} from "@/lib/components/ui/card";
import { Button } from "@/lib/components/ui/button";
import { Progress } from "@/lib/components/ui/progress";
import { Play, X } from "@lucide/vue";

defineProps<{
  nextVideo: VideoItem_Deserialize;
  countdown: number;
}>();

defineEmits<{
  play: [];
  cancel: [];
}>();
</script>

<template>
  <div
    class="absolute inset-0 z-30 flex items-center justify-center bg-background/80 backdrop-blur-sm p-4 select-none animate-in fade-in duration-200"
    @click.stop
  >
    <Card class="max-w-xs sm:max-w-sm w-full p-4 gap-3.5 shadow-2xl">
      <CardHeader class="p-0 gap-1.5">
        <div class="flex items-center justify-between">
          <CardTitle class="text-xs uppercase tracking-wider text-muted-foreground">
            Up Next
          </CardTitle>
          <span class="text-xs font-semibold tabular-nums text-foreground">
            {{ countdown }}s
          </span>
        </div>
        <Progress
          :model-value="((4 - countdown) / 4) * 100"
          class="h-1.5"
        />
      </CardHeader>

      <CardContent class="p-0 flex items-center gap-3">
        <img
          v-if="nextVideo.thumbnailUrl"
          :src="nextVideo.thumbnailUrl"
          :alt="nextVideo.title"
          class="w-24 aspect-video object-cover rounded-md bg-muted shrink-0 border border-border"
        />
        <div class="min-w-0 flex-1">
          <h4 class="text-xs font-semibold text-foreground line-clamp-2 leading-snug">
            {{ nextVideo.title }}
          </h4>
          <p class="text-[11px] text-muted-foreground truncate mt-1">
            {{ nextVideo.channelName }}
          </p>
        </div>
      </CardContent>

      <CardFooter class="p-0 flex items-center justify-end gap-2 pt-1">
        <Button
          type="button"
          variant="secondary"
          size="sm"
          class="cursor-pointer"
          @click="$emit('cancel')"
        >
          <X class="h-3.5 w-3.5" />
          Cancel
        </Button>

        <Button
          type="button"
          variant="default"
          size="sm"
          class="cursor-pointer font-medium"
          @click="$emit('play')"
        >
          <Play class="h-3.5 w-3.5 fill-current" />
          Play Now
        </Button>
      </CardFooter>
    </Card>
  </div>
</template>
