<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { Plus, RefreshCw, Trash2, User } from "@lucide/vue";
import { commands, type Model } from "@/bindings";

defineOptions({ name: "ConnectedAccounts" });

const router = useRouter();

const profiles = ref<Model[]>([]);
const activeProfile = ref<Model | null>(null);
const isLoadingProfiles = ref(false);

async function loadProfiles() {
  isLoadingProfiles.value = true;
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
    isLoadingProfiles.value = false;
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

async function handleDeleteProfile(id: number) {
  if (!confirm("Are you sure you want to remove this profile?")) return;
  try {
    const res = await commands.deleteProfile(id);
    if (res.status === "ok") {
      await loadProfiles();
    }
  } catch (err) {
    console.error("Failed to delete profile:", err);
  }
}

onMounted(loadProfiles);
</script>

<template>
  <section class="space-y-4">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <User class="h-4 w-4 text-primary" />
        <h2 class="text-sm font-semibold tracking-wider text-muted-foreground">
          Connected YouTube Accounts
        </h2>
      </div>

      <div class="flex items-center gap-2">
        <Button
          variant="ghost"
          size="sm"
          class="gap-1.5 text-xs"
          :disabled="isLoadingProfiles"
          @click="loadProfiles"
        >
          <RefreshCw
            :class="['h-3.5 w-3.5', isLoadingProfiles ? 'animate-spin' : '']"
          />
          <span>Refresh</span>
        </Button>

        <Button
          variant="default"
          size="sm"
          class="gap-1.5 text-xs"
          @click="router.push('/login')"
        >
          <Plus class="h-3.5 w-3.5" />
          <span>Add Account</span>
        </Button>
      </div>
    </div>

    <div
      class="rounded-xl border border-border bg-card p-4 sm:p-5 shadow-xs space-y-3"
    >
      <div
        v-if="profiles.length === 0"
        class="text-center py-8 text-sm text-muted-foreground"
      >
        <p>No Google or YouTube accounts connected yet.</p>
        <Button
          variant="outline"
          size="sm"
          class="mt-3 gap-1.5 text-xs"
          @click="router.push('/login')"
        >
          <Plus class="h-3.5 w-3.5" />
          <span>Sign in to YouTube</span>
        </Button>
      </div>

      <div v-else class="space-y-2">
        <div
          v-for="prof in profiles"
          :key="prof.id"
          class="flex items-center justify-between p-3 rounded-lg border border-border/50 bg-muted/20 hover:bg-muted/30 transition-colors"
        >
          <div class="flex items-center gap-3">
            <img
              v-if="prof.avatar_url"
              :src="prof.avatar_url"
              :alt="prof.name"
              class="h-9 w-9 rounded-full object-cover border border-border/40"
            />
            <div
              v-else
              class="h-9 w-9 rounded-full bg-primary/20 text-primary flex items-center justify-center font-bold text-xs"
            >
              {{ prof.name.slice(0, 2) }}
            </div>
            <div>
              <div class="flex items-center gap-2">
                <p class="text-sm font-semibold text-foreground">
                  {{ prof.name }}
                </p>
                <span
                  v-if="activeProfile?.id === prof.id"
                  class="text-[10px] font-medium bg-emerald-500/10 text-emerald-500 px-2 py-0.5 rounded-full"
                >
                  Active
                </span>
              </div>
              <p class="text-xs text-muted-foreground">
                {{ prof.email || prof.google_id }}
              </p>
            </div>
          </div>

          <div class="flex items-center gap-2">
            <Button
              v-if="activeProfile?.id !== prof.id"
              variant="outline"
              size="sm"
              class="text-xs h-8"
              @click="handleSwitchProfile(prof.id)"
            >
              Switch To
            </Button>
            <Button
              variant="ghost"
              size="icon"
              class="h-8 w-8 text-muted-foreground hover:text-destructive hover:bg-destructive/10"
              @click="handleDeleteProfile(prof.id)"
            >
              <Trash2 class="h-4 w-4" />
            </Button>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>