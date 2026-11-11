<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRouter } from "vue-router";
import { Menu, Moon, Sun, User, LogOut, Loader2 } from "@lucide/vue";
import { commands, type Model } from "@/bindings";
import { useSidebar } from "@/lib/components/ui/sidebar";
import HeaderSearch from "@/components/layout/HeaderSearch.vue";

defineProps<{
  hideSidebar?: boolean;
}>();

const router = useRouter();
const { toggleSidebar } = useSidebar();

const isDark = ref(true);
const activeProfile = ref<Model | null>(null);
const isLoggingIn = ref(false);

onMounted(async () => {
  document.documentElement.classList.add("dark");

  try {
    const res = await commands.getActiveProfile();
    if (res.status === "ok" && res.data) {
      activeProfile.value = res.data;
    }
  } catch (err) {
    console.error("Failed to fetch active profile:", err);
  }
});

function toggleTheme() {
  isDark.value = !isDark.value;
  if (isDark.value) {
    document.documentElement.classList.add("dark");
  } else {
    document.documentElement.classList.remove("dark");
  }
}

async function handleLogin() {
  router.push("/login");
}
</script>

<template>
  <header
    class="sticky top-0 z-40 flex h-14 w-full items-center justify-between border-b border-border/20 bg-background/95 px-4 backdrop-blur-md"
  >
    <!-- Left Section: Toggle & Logo -->
    <div class="flex items-center gap-3">
      <Button
        v-if="!hideSidebar"
        variant="ghost"
        size="icon"
        class="h-9 w-9 text-foreground/80 hover:bg-accent hover:text-foreground"
        @click="toggleSidebar"
      >
        <Menu class="h-5 w-5" />
        <span class="sr-only">Toggle Sidebar</span>
      </Button>
    </div>

    <!-- Center Section: Search Bar with Suggestions -->
    <HeaderSearch />

    <!-- Right Section: Actions & Profile -->
    <div class="flex items-center gap-2">
      <!-- Dark / Light Theme Toggle -->
      <Button
        variant="ghost"
        size="icon"
        class="h-9 w-9 text-muted-foreground hover:text-foreground"
        @click="toggleTheme"
      >
        <Sun v-if="isDark" class="h-4 w-4" />
        <Moon v-else class="h-4 w-4" />
        <span class="sr-only">Toggle theme</span>
      </Button>

      <!-- Authenticated Profile Dropdown or Sign In Button -->
      <template v-if="activeProfile">
        <DropdownMenu>
          <DropdownMenuTrigger as-child>
            <button
              class="flex items-center gap-2 rounded-full p-0.5 ring-2 ring-primary/20 hover:ring-primary/40 focus:outline-hidden cursor-pointer"
            >
              <img
                v-if="activeProfile.avatar_url"
                :src="activeProfile.avatar_url"
                :alt="activeProfile.name"
                class="h-8 w-8 rounded-full object-cover"
              />
              <div
                v-else
                class="flex h-8 w-8 items-center justify-center rounded-full bg-primary text-primary-foreground font-semibold text-xs"
              >
                {{ activeProfile.name.slice(0, 2).toUpperCase() }}
              </div>
            </button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" class="w-56">
            <DropdownMenuLabel class="font-normal">
              <div class="flex flex-col space-y-1">
                <p class="text-sm font-semibold leading-none text-foreground">
                  {{ activeProfile.name }}
                </p>
                <p class="text-xs leading-none text-muted-foreground">
                  {{ activeProfile.email || activeProfile.google_id }}
                </p>
              </div>
            </DropdownMenuLabel>
            <DropdownMenuSeparator />
            <DropdownMenuItem @click="router.push('/settings')">
              <User class="mr-2 h-4 w-4" />
              <span>Account Settings</span>
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem
              @click="
                async () => {
                  await commands.logout();
                  handleLogin();
                }
              "
            >
              <LogOut class="mr-2 h-4 w-4" />
              <span>Switch Account / Sign In</span>
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </template>

      <!-- Guest Login Button -->
      <template v-else>
        <Button
          size="sm"
          variant="outline"
          class="gap-1.5 rounded-full border-primary/30 text-primary hover:bg-primary/10"
          :disabled="isLoggingIn"
          @click="handleLogin"
        >
          <Loader2 v-if="isLoggingIn" class="h-4 w-4 animate-spin" />
          <User v-else class="h-4 w-4" />
          <span>{{ isLoggingIn ? "Signing in..." : "Sign in" }}</span>
        </Button>
      </template>
    </div>
  </header>
</template>
