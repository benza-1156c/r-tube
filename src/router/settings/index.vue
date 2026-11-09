<script setup lang="ts">
import { Settings, User, RefreshCw } from "@lucide/vue";
import { commands, type Model } from "@/bindings";

const profiles = ref<Model[]>([]);
const activeProfile = ref<Model | null>(null);
const isLoading = ref(false);

async function loadProfiles() {
  isLoading.value = true;
  try {
    const [allRes, activeRes] = await Promise.all([
      commands.getAllProfiles(),
      commands.getActiveProfile(),
    ]);

    if (allRes.status === "ok") {
      profiles.value = allRes.data;
    }
    if (activeRes.status === "ok") {
      activeProfile.value = activeRes.data;
    }
  } catch (err) {
    console.error("Failed to load profiles:", err);
  } finally {
    isLoading.value = false;
  }
}

async function handleSwitchProfile(id: number) {
  try {
    const res = await commands.setActiveProfile(id);
    if (res.status === "ok") {
      await loadProfiles();
    }
  } catch (err) {
    console.error("Failed to switch profile:", err);
  }
}

onMounted(() => {
  loadProfiles();
});
</script>

<template>
  <div class="p-6 max-w-4xl mx-auto flex flex-col gap-6">
    <div class="flex items-center gap-3 border-b border-border/40 pb-4">
      <div class="flex h-10 w-10 items-center justify-center rounded-xl bg-primary/10 text-primary">
        <Settings class="h-6 w-6" />
      </div>
      <div>
        <h1 class="text-xl font-bold text-foreground">Settings</h1>
        <p class="text-xs text-muted-foreground">Manage connected profiles and account sessions</p>
      </div>
    </div>

    <!-- Profile Management Section -->
    <div class="rounded-xl border border-border bg-card p-6 shadow-xs space-y-4">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <User class="h-5 w-5 text-primary" />
          <h2 class="text-base font-semibold text-foreground">Connected YouTube Profiles</h2>
        </div>
        <Button variant="ghost" size="sm" class="gap-1.5" @click="loadProfiles">
          <RefreshCw :class="['h-3.5 w-3.5', isLoading ? 'animate-spin' : '']" />
          <span>Refresh</span>
        </Button>
      </div>

      <div v-if="profiles.length === 0" class="text-center py-6 text-sm text-muted-foreground">
        No accounts connected yet. Use the Sign In button in the header.
      </div>

      <div v-else class="space-y-2">
        <div
          v-for="prof in profiles"
          :key="prof.id"
          class="flex items-center justify-between p-3 rounded-lg border border-border/50 bg-muted/20"
        >
          <div class="flex items-center gap-3">
            <img
              v-if="prof.avatar_url"
              :src="prof.avatar_url"
              :alt="prof.name"
              class="h-9 w-9 rounded-full object-cover"
            />
            <div
              v-else
              class="h-9 w-9 rounded-full bg-primary/20 flex items-center justify-center font-bold text-xs"
            >
              {{ prof.name.slice(0, 2) }}
            </div>
            <div>
              <p class="text-sm font-semibold text-foreground">{{ prof.name }}</p>
              <p class="text-xs text-muted-foreground">{{ prof.email || prof.google_id }}</p>
            </div>
          </div>

          <div class="flex items-center gap-2">
            <span
              v-if="prof.is_active"
              class="px-2 py-0.5 rounded-full text-xs font-semibold bg-green-500/10 text-green-500"
            >
              Active
            </span>
            <Button
              v-else
              size="sm"
              variant="outline"
              @click="handleSwitchProfile(prof.id)"
            >
              Switch to this profile
            </Button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
