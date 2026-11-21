<script setup lang="ts">
import { computed, ref } from "vue";
import { Sliders } from "@lucide/vue";
import {
  useSettings,
  type AppSettings,
  type PlaybackMode,
} from "@/composables/useSettings";
import { getDeviceLanguages } from "@/lib/utils";
import SettingRow from "./SettingRow.vue";
import type {
  AnySettingDefinition,
  SettingDefinition,
  SettingOption,
} from "./types";

defineOptions({ name: "SettingsPreferences" });

const { settings, updateSettings } = useSettings();

// Helper function to enforce type safety per item
function defineSetting<K extends keyof AppSettings>(
  def: SettingDefinition<K>,
): SettingDefinition<K> {
  return def;
}

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

// ── Audio Language Options (device languages first) ──
function displayLanguageName(code: string, locale: string): string | null {
  try {
    return (
      new Intl.DisplayNames([locale], { type: "language" }).of(code) ?? null
    );
  } catch {
    return null;
  }
}

function deviceLanguageOption(code: string): SettingOption<string> {
  const english = displayLanguageName(code, "en");
  const native = displayLanguageName(code, code);
  const label =
    english && native && english.toLowerCase() !== native.toLowerCase()
      ? `${english} (${native})`
      : english || native || code.toUpperCase();

  return {
    label,
    value: code,
    description: "Detected from your device language",
    badge: "Device",
  };
}

const knownAudioLanguages: SettingOption<string>[] = [
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
];

/** Device languages first, then the built-in list; never hide the saved value. */
const audioLanguageOptions = computed<SettingOption<string>[]>(() => {
  const options: SettingOption<string>[] = [
    {
      label: "Original / Default",
      value: "original",
      description: "Use the video's original audio track",
    },
  ];

  for (const code of getDeviceLanguages()) {
    if (!code || code === "original") continue;
    if (options.some((option) => option.value === code)) continue;

    options.push(
      knownAudioLanguages.find((option) => option.value === code) ??
        deviceLanguageOption(code),
    );
  }

  for (const option of knownAudioLanguages) {
    if (!options.some((existing) => existing.value === option.value)) {
      options.push(option);
    }
  }

  const current = settings.value.audioLanguage;
  if (current && !options.some((option) => option.value === current)) {
    options.push(deviceLanguageOption(current));
  }

  return options;
});

// ── Setting Items Definitions (All English & Type-Safe) ──
const settingDefinitions = computed<AnySettingDefinition[]>(() => [
  // 1. Preferred Audio Language (Video Audio / Dub track)
  defineSetting({
    key: "audioLanguage",
    title: "Preferred Audio Language",
    description:
      "Default audio track or dubbing language when multi-audio streams are available",
    category: "playback",
    type: "select",
    options: audioLanguageOptions.value,
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
        label: "240p",
        value: "240p",
        description: "Low bandwidth streaming",
      },
      {
        label: "144p",
        value: "144p",
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
        label: "Normal",
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
]);
</script>

<template>
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
      <SettingRow
        v-for="item in settingDefinitions"
        :key="item.key"
        :item="item"
        :value="settings[item.key]"
        :saved="lastSavedKey === item.key"
        @change="(value) => handleValueChange(item, value)"
      />
    </div>
  </section>
</template>