<script setup lang="ts">
import { onMounted } from "vue";
import { useRouter } from "vue-router";
import { ArrowLeft } from "@lucide/vue";
import { useSettings } from "@/composables/useSettings";
import SettingsPreferences from "@/components/settings/SettingsPreferences.vue";
import ConnectedAccounts from "@/components/settings/ConnectedAccounts.vue";

definePage({
  meta: {
    hideSidebar: false,
  },
});

const router = useRouter();
const { loadSettings } = useSettings();

onMounted(loadSettings);
</script>

<template>
  <div class="max-w-4xl mx-auto p-6 space-y-8">
    <!-- Header -->
    <div class="flex items-center justify-between border-b border-border/40 pb-4">
      <div class="flex items-center gap-3">
        <Button
          variant="ghost"
          size="icon"
          class="h-9 w-9 rounded-lg"
          @click="router.back()"
        >
          <ArrowLeft class="h-5 w-5" />
        </Button>

        <div>
          <h1 class="text-xl font-bold text-foreground">Settings</h1>
          <p class="text-xs text-muted-foreground">
            Manage application preferences, playback, and accounts
          </p>
        </div>
      </div>
    </div>

    <SettingsPreferences />
    <ConnectedAccounts />
  </div>
</template>