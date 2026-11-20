import { ref } from "vue";
import { load, type Store } from "@tauri-apps/plugin-store";
import { commands, type AppSettings_Serialize, type PlaybackMode } from "@/bindings";
import { getPrimaryDeviceLanguage } from "@/lib/utils";

export type AppSettings = AppSettings_Serialize;
export type { PlaybackMode };

export const defaultSettings: AppSettings = {
  volume: 1.0,
  audioLanguage: getPrimaryDeviceLanguage(),
  defaultResolution: "1080p",
  playbackMode: "normal",
};

export const SETTINGS_FILE = "settings.json";
export const SETTINGS_KEY = "settings";

let storeInstance: Store | null = null;

async function getStore(): Promise<Store> {
  if (!storeInstance) {
    storeInstance = await load(SETTINGS_FILE, { autoSave: false });
  }
  return storeInstance;
}

const settings = ref<AppSettings>({ ...defaultSettings });
const isLoaded = ref<boolean>(false);

/**
 * Mirror a settings snapshot into the Rust side, which resolves audio tracks.
 *
 * The store written by `plugin-store` stays the single source of truth: Rust
 * keeps its copy in memory only and never saves it back, so playback cannot
 * overwrite a choice the user made. Debounced because volume changes fire
 * repeatedly while the slider is dragged.
 */
let rustSyncTimer: ReturnType<typeof setTimeout> | null = null;

function syncSettingsWithRust(next: AppSettings, immediate = false): void {
  if (rustSyncTimer) {
    clearTimeout(rustSyncTimer);
    rustSyncTimer = null;
  }

  const send = (): void => {
    void commands
      .updateSettings(next)
      .then((res) => {
        if (res.status === "error") {
          console.warn("Could not sync settings with Rust:", res.error);
        }
      })
      .catch((err) => {
        console.warn("Could not sync settings with Rust:", err);
      });
  };

  if (immediate) {
    send();
  } else {
    rustSyncTimer = setTimeout(() => {
      rustSyncTimer = null;
      send();
    }, 150);
  }
}

export function useSettings() {
  /**
   * Load settings from plugin-store, falling back to defaults if not present.
   */
  async function loadSettings(): Promise<AppSettings> {
    try {
      const store = await getStore();
      const val = await store.get<Record<string, any>>(SETTINGS_KEY);
      if (val) {
        settings.value = {
          volume: typeof val.volume === "number" ? val.volume : defaultSettings.volume,
          audioLanguage: String(val.audioLanguage || val.language || defaultSettings.audioLanguage),
          defaultResolution: String(val.defaultResolution || defaultSettings.defaultResolution),
          playbackMode: (val.playbackMode === "audio-only" ? "audio-only" : "normal") as PlaybackMode,
        };
      } else {
        await store.set(SETTINGS_KEY, defaultSettings);
        await store.save();
        settings.value = { ...defaultSettings };
      }
      isLoaded.value = true;
      syncSettingsWithRust(settings.value, true);
    } catch (err) {
      console.error("Failed to load settings from store:", err);
    }
    return settings.value;
  }

  /**
   * Update settings in memory and persist immediately to store.
   */
  async function updateSettings(
    patch: Partial<AppSettings>,
  ): Promise<AppSettings> {
    try {
      const updated: AppSettings = {
        ...settings.value,
        ...patch,
      };
      settings.value = updated;

      const store = await getStore();
      await store.set(SETTINGS_KEY, updated);
      await store.save();
      // Audio/quality changes should affect the next track right away; volume
      // events are frequent while dragging, so only those are debounced.
      syncSettingsWithRust(
        updated,
        Object.keys(patch).some((key) => key !== "volume"),
      );
    } catch (err) {
      console.error("Failed to save settings to store:", err);
    }
    return settings.value;
  }

  /**
   * Reset settings to factory defaults.
   */
  async function resetSettings(): Promise<AppSettings> {
    try {
      settings.value = { ...defaultSettings };
      const store = await getStore();
      await store.set(SETTINGS_KEY, defaultSettings);
      await store.save();
      syncSettingsWithRust(settings.value, true);
    } catch (err) {
      console.error("Failed to reset settings in store:", err);
    }
    return settings.value;
  }

  return {
    settings,
    isLoaded,
    loadSettings,
    updateSettings,
    resetSettings,
    getStore,
  };
}
