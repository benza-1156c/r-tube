<script setup lang="ts">
import { onMounted } from "vue";
import { useRoute, useRouter } from "vue-router";

const route = useRoute();
const router = useRouter();

onMounted(() => {
  const id = String(
    (route.params as unknown as Record<string, string | undefined>).id ?? "",
  );
  void router.replace(id ? `/channel/${id}` : "/subscriptions");
});
</script>

<template>
  <div class="flex items-center justify-center py-24 text-sm text-muted-foreground">
    Redirecting...
  </div>
</template>