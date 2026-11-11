<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRouter } from "vue-router";
import {
  ArrowLeft,
  Settings as SettingsIcon,
  Sliders,
  Volume2,
  VolumeX,
  User,
  Plus,
  RefreshCw,
  Trash2,
  Check,
} from "@lucide/vue";
import { commands, type Model } from "@/bindings";
import {
  useSettings,
  type AppSettings,
  type PlaybackMode,
} from "@/composables/useSettings";
import { Button } from "@/lib/components/ui/button";

definePage({
  meta: {
    hideSidebar: false,
  },
});

const router = useRouter();
const { settings, loadSettings, updateSettings } = useSettings();

// ── Types for Type-safe Settings Configuration ──
export interface SettingOption<T> {
  label: string;
  value: T;
  description?: string;
  badge?: string;
}

export type SettingType = "select" | "segmented" | "slider";

export interface SettingDefinition<K extends keyof AppSettings> {
  key: K;
  title: string;
  description: string;
  category: "general" | "playback" | "audio";
  type: SettingType;
  options?: SettingOption<AppSettings[K]>[];
  min?: number;
  max?: number;
  step?: number;
  unit?: string;
  /**
   * Callback invoked when the user selects a new value.
   * Passes the newly selected value and the updated settings object.
   */
  onChange?: (
    value: AppSettings[K],
    current: AppSettings,
  ) => Promise<void> | void;
}

// Helper function to enforce type safety per item
function defineSetting<K extends keyof AppSettings>(
  def: SettingDefinition<K>,
): SettingDefinition<K> {
  return def;
}

export type AnySettingDefinition =
  | SettingDefinition<"audioLanguage">
  | SettingDefinition<"defaultResolution">
  | SettingDefinition<"playbackMode">
  | SettingDefinition<"volume">;

// ── Save Feedback Indicator ──
const lastSavedKey = ref<string | null>(null);
let saveTimer: ReturnType<typeof setTimeout> | null = null;

function flashSaved(key: string) {
  lastSavedKey.value = key;
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(() => {
    lastSavedKey.value = null;
  }, 1800);
}

// ── Generic Selection Handler ──
async function handleValueChange(item: AnySettingDefinition, newValue: any) {
  // 1. Persist to store with type safety
  await updateSettings({ [item.key]: newValue });
  flashSaved(item.key);

  // 2. Invoke post-selection callback
  if (item.onChange) {
    try {
      await (item.onChange as any)(newValue, settings.value);
    } catch (err) {
      console.error(`Error in onChange handler for ${String(item.key)}:`, err);
    }
  }
}

// ── Setting Items Definitions (All English & Type-Safe) ──
const settingDefinitions: AnySettingDefinition[] = [
  // 1. Preferred Audio Language (Video Audio / Dub track)
  defineSetting({
    key: "audioLanguage",
    title: "Preferred Audio Language",
    description:
      "Default audio track or dubbing language when multi-audio streams are available",
    category: "playback",
    type: "select",
    options: [
      {
        label: "Original / Default",
        value: "original",
        description: "Use the video's original audio track",
      },
      {
        label: "Thai (ภาษาไทย)",
        value: "th",
        description: "Prefer Thai audio track or dub when available",
      },
      {
        label: "English",
        value: "en",
        description: "Prefer English audio track when available",
      },
      {
        label: "Japanese (日本語)",
        value: "ja",
        description: "Prefer Japanese audio track when available",
      },
      {
        label: "Korean (한국어)",
        value: "ko",
        description: "Prefer Korean audio track when available",
      },
      {
        label: "Spanish (Español)",
        value: "es",
        description: "Prefer Spanish audio track when available",
      },
    ],
    onChange: async (val, current) => {
      console.log(`[Settings] Audio language changed to: ${val}`, current);
    },
  }),

  // 2. Default Resolution
  defineSetting({
    key: "defaultResolution",
    title: "Default Video Quality",
    description:
      "Preferred video resolution automatically selected when opening a new stream",
    category: "playback",
    type: "select",
    options: [
      {
        label: "1080p",
        value: "1080p",
        badge: "HD",
        description:
          "Full High Definition 1080p (Recommended for fast networks)",
      },
      {
        label: "720p ",
        value: "720p",
        description: "High Definition 720p (Fast and smooth buffering)",
      },
      {
        label: "480p",
        value: "480p",
        description: "Standard Definition (Saves bandwidth)",
      },
      {
        label: "360p",
        value: "360p",
        description: "Low bandwidth streaming",
      },
      {
        label: "Auto",
        value: "auto",
        description: "Automatically adapt based on available connection speed",
      },
    ],
    onChange: async (val, current) => {
      console.log(`[Settings] Default resolution changed to: ${val}`, current);
    },
  }),

  // 3. Playback Mode
  defineSetting({
    key: "playbackMode",
    title: "Playback Mode",
    description:
      "Choose between standard video playback or background audio-only streaming",
    category: "playback",
    type: "segmented",
    options: [
      {
        label: "Normal (Video + Audio)",
        value: "normal" as PlaybackMode,
        description: "Stream full video and audio tracks",
      },
      {
        label: "Audio Only",
        value: "audio-only" as PlaybackMode,
        description: "Stream only audio to save bandwidth and system resources",
      },
    ],
    onChange: async (val, current) => {
      console.log(`[Settings] Playback mode changed to: ${val}`, current);
    },
  }),

  // 4. Master Volume
  defineSetting({
    key: "volume",
    title: "Default Volume",
    description: "Default playback volume level when opening the application",
    category: "audio",
    type: "slider",
    min: 0,
    max: 1,
    step: 0.05,
    unit: "%",
    onChange: async (val, current) => {
      console.log(
        `[Settings] Master volume set to: ${Math.round((val || 0) * 100)}%`,
        current,
      );
    },
  }),
];

// ── Profile Accounts Management ──
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

onMounted(async () => {
  await Promise.all([loadSettings(), loadProfiles()]);
});
</script>

<template>
  <div class="max-w-4xl mx-auto p-6 space-y-8">
    <!-- Header -->
    <div
      class="flex items-center justify-between border-b border-border/40 pb-4"
    >
      <div class="flex items-center gap-3">
        <Button
          variant="ghost"
          size="icon"
          class="h-9 w-9 rounded-lg"
          @click="router.back()"
        >
          <ArrowLeft class="h-5 w-5" />
        </Button>
        <div
          class="flex h-10 w-10 items-center justify-center rounded-xl bg-primary/10 text-primary"
        >
          <SettingsIcon class="h-5 w-5" />
        </div>
        <div>
          <h1 class="text-xl font-bold text-foreground">Settings</h1>
          <p class="text-xs text-muted-foreground">
            Manage application preferences, playback, and accounts
          </p>
        </div>
      </div>
    </div>

    <!-- ── 1. PREFERENCES SECTION ── -->
    <section class="space-y-4">
      <div class="flex items-center gap-2">
        <Sliders class="h-4 w-4 text-primary" />
        <h2
          class="text-sm font-semibold uppercase tracking-wider text-muted-foreground"
        >
          Application Preferences
        </h2>
      </div>

      <div
        class="rounded-xl border border-border bg-card divide-y divide-border/40 overflow-hidden shadow-xs"
      >
        <div
          v-for="item in settingDefinitions"
          :key="item.key"
          class="p-4 sm:p-5 flex flex-col sm:flex-row sm:items-center justify-between gap-4 transition-colors hover:bg-muted/10"
        >
          <!-- Title & Description -->
          <div class="space-y-1 pr-4 max-w-md">
            <div class="flex items-center gap-2">
              <span class="text-sm font-medium text-foreground">{{
                item.title
              }}</span>
              <!-- Save Indicator badge -->
              <transition
                enter-active-class="transition duration-150 ease-out"
                enter-from-class="opacity-0 scale-90"
                enter-to-class="opacity-100 scale-100"
                leave-active-class="transition duration-150 ease-in"
                leave-from-class="opacity-100 scale-100"
                leave-to-class="opacity-0 scale-90"
              >
                <span
                  v-if="lastSavedKey === item.key"
                  class="inline-flex items-center gap-1 text-[11px] font-medium text-emerald-500 bg-emerald-500/10 px-2 py-0.5 rounded-full"
                >
                  <Check class="h-3 w-3" />
                  <span>Saved</span>
                </span>
              </transition>
            </div>
            <p class="text-xs text-muted-foreground leading-relaxed">
              {{ item.description }}
            </p>
          </div>

          <!-- Controls by Type -->
          <div class="sm:shrink-0 flex items-center">
            <!-- ── A. SELECT DROPDOWN ── -->
            <div v-if="item.type === 'select'" class="relative min-w-48">
              <select
                :value="settings[item.key]"
                class="w-full h-9 px-3 rounded-lg border border-border bg-background text-sm font-medium text-foreground focus:outline-hidden focus:ring-2 focus:ring-primary/40 cursor-pointer transition-shadow"
                @change="
                  handleValueChange(
                    item,
                    ($event.target as HTMLSelectElement).value as any,
                  )
                "
              >
                <option
                  v-for="opt in item.options"
                  :key="String(opt.value)"
                  :value="opt.value"
                >
                  {{ opt.label }}
                </option>
              </select>
            </div>

            <!-- ── B. SEGMENTED CONTROLS ── -->
            <div
              v-else-if="item.type === 'segmented'"
              class="inline-flex p-1 rounded-lg bg-muted/30 border border-border/50 gap-1"
            >
              <button
                v-for="opt in item.options"
                :key="String(opt.value)"
                type="button"
                class="px-3 py-1.5 text-xs font-medium rounded-md transition-all cursor-pointer select-none"
                :class="
                  settings[item.key] === opt.value
                    ? 'bg-background text-foreground shadow-xs font-semibold'
                    : 'text-muted-foreground hover:text-foreground'
                "
                @click="handleValueChange(item, opt.value)"
              >
                {{ opt.label }}
              </button>
            </div>

            <!-- ── C. VOLUME SLIDER ── -->
            <div
              v-else-if="item.type === 'slider'"
              class="flex items-center gap-3 w-full sm:w-56"
            >
              <button
                type="button"
                class="text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
                @click="
                  handleValueChange(item, (settings.volume || 0) > 0 ? 0 : 1)
                "
              >
                <VolumeX
                  v-if="(settings.volume || 0) === 0"
                  class="h-4 w-4 text-rose-500"
                />
                <Volume2 v-else class="h-4 w-4" />
              </button>

              <input
                type="range"
                :min="item.min ?? 0"
                :max="item.max ?? 1"
                :step="item.step ?? 0.05"
                :value="settings.volume ?? 1"
                class="w-full accent-primary h-1.5 bg-muted rounded-lg cursor-pointer"
                @input="
                  handleValueChange(
                    item,
                    Number(($event.target as HTMLInputElement).value),
                  )
                "
              />

              <span
                class="text-xs font-mono font-medium text-foreground w-10 text-right"
              >
                {{ Math.round((settings.volume ?? 1) * 100) }}%
              </span>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- ── 2. CONNECTED ACCOUNTS SECTION ── -->
    <section class="space-y-4">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <User class="h-4 w-4 text-primary" />
          <h2
            class="text-sm font-semibold uppercase tracking-wider text-muted-foreground"
          >
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
  </div>
</template>
