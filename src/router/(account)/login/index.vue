<script setup lang="ts">
import { commands, type Model } from "@/bindings";
import { Plus, Check, Loader2, ArrowLeft, User } from "@lucide/vue";

const router = useRouter();
const profiles = ref<Model[]>([]);
const isLoggingIn = ref(false);
const isSwitchingId = ref<number | null>(null);
const errorMessage = ref<string | null>(null);

async function get_profiles() {
  const p = await commands.getAllProfiles();
  if (p.status === "ok") {
    // Older builds stored a placeholder when the account lookup failed.
    // Keep it out of the picker; signing in again repairs/reuses it.
    profiles.value = p.data.filter(
      (prof) => !(prof.name === "YouTube User" && !prof.email),
    );
  }
}

async function selectProfile(id: number) {
  isSwitchingId.value = id;
  try {
    const res = await commands.setActiveProfile(id);
    if (res.status === "ok") {
      router.push("/");
    }
  } catch (err) {
    console.error("Failed to set active profile:", err);
  } finally {
    isSwitchingId.value = null;
  }
}

async function handleLogin() {
  if (isLoggingIn.value) return;
  isLoggingIn.value = true;
  errorMessage.value = null;
  try {
    const res = await commands.loginYt();
    if (res.status === "ok") {
      await get_profiles();
      router.push("/");
    } else {
      const message = res.error.message || "Sign in failed";
      errorMessage.value = /cancelled/i.test(message) ? null : message;
    }
  } catch (err) {
    errorMessage.value = err instanceof Error ? err.message : "Sign in failed";
    console.error("Login failed:", err);
  } finally {
    isLoggingIn.value = false;
  }
}

async function cancelLogin() {
  try {
    await commands.cancelLogin();
  } catch {}
}

onMounted(() => {
  get_profiles();
});

definePage({
  meta: {
    title: "Sign in",
    hideSidebar: true,
    hideHeader: true,
  },
});
</script>

<template>
  <div class="flex min-h-screen items-center justify-center p-4">
    <Card class="w-full max-w-sm">
      <CardHeader class="text-center">
        <CardTitle class="text-2xl font-bold">Sign In</CardTitle>
        <CardDescription>
          Select an account or connect a new YouTube profile
        </CardDescription>
      </CardHeader>

      <CardContent class="flex flex-col gap-4">
        <!-- Existing profiles list -->
        <div v-if="profiles.length > 0" class="flex flex-col gap-2">
          <Button
            v-for="prof in profiles"
            :key="prof.id"
            variant="outline"
            class="h-auto w-full justify-between p-3"
            :disabled="isSwitchingId === prof.id"
            @click="selectProfile(prof.id)"
          >
            <div class="flex items-center gap-3 min-w-0 text-left">
              <img
                v-if="prof.avatar_url"
                :src="prof.avatar_url"
                :alt="prof.name"
                class="size-8 rounded-full object-cover shrink-0"
              />
              <div
                v-else
                class="size-8 rounded-full bg-muted flex items-center justify-center text-xs font-semibold shrink-0"
              >
                <User class="size-4 text-muted-foreground" />
              </div>

              <div class="flex flex-col min-w-0">
                <span class="truncate font-medium text-sm">{{
                  prof.name
                }}</span>
                <span
                  v-if="prof.email"
                  class="truncate text-xs text-muted-foreground"
                >
                  {{ prof.email }}
                </span>
              </div>
            </div>

            <div class="shrink-0 ml-2">
              <Loader2
                v-if="isSwitchingId === prof.id"
                class="size-4 animate-spin text-muted-foreground"
              />
              <Check v-else-if="prof.is_active" class="size-4 text-primary" />
            </div>
          </Button>

          <Separator class="my-2" />
        </div>

        <!-- Add / Sign in button -->
        <Button class="w-full" :disabled="isLoggingIn" @click="handleLogin">
          <Loader2 v-if="isLoggingIn" class="size-4 animate-spin mr-2" />
          <Plus v-else class="size-4 mr-2" />
          <span>{{
            isLoggingIn
              ? "Signing in..."
              : profiles.length > 0
                ? "Add another account"
                : "Sign in with YouTube"
          }}</span>
        </Button>

        <p
          v-if="isLoggingIn"
          class="text-center text-xs text-muted-foreground leading-relaxed"
        >
          Complete the sign-in in the YouTube window. If it shows the YouTube
          home page, click "Sign in" there first.
        </p>

        <Button
          v-if="isLoggingIn"
          variant="ghost"
          class="w-full"
          @click="cancelLogin"
        >
          Cancel sign in
        </Button>

        <p v-if="errorMessage" class="text-xs text-red-500 leading-relaxed">
          {{ errorMessage }}
        </p>
      </CardContent>

      <CardFooter>
        <Button
          variant="ghost"
          class="w-full text-xs text-muted-foreground"
          @click="router.push('/')"
        >
          <ArrowLeft class="size-3.5 mr-1.5" />
          <span>Back to Home</span>
        </Button>
      </CardFooter>
    </Card>
  </div>
</template>
