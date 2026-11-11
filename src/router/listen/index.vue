<script setup lang="ts">
import { onMounted } from "vue";
import { useRouter } from "vue-router";
import { usePlayer } from "@/composables/usePlayer";

const router = useRouter();
const player = usePlayer();

onMounted(() => {
  if (player.currentVideo.value?.id) {
    router.replace(`/listen/${player.currentVideo.value.id}`);
  } else {
    router.replace("/");
  }
});
</script>

<template>
  <div class="p-8 text-center text-muted-foreground text-sm">
    Redirecting to listening view...
  </div>
</template>

