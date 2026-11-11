import { ref } from "vue";
import { load, type Store } from "@tauri-apps/plugin-store";
import type { AppSettings_Serialize, PlaybackMode } from "@/bindings";

export type AppSettings = AppSettings_Serialize;
export type { PlaybackMode };

export const defaultSettings: AppSettings = {
  volume: 1.0,
  audioLanguage: "th",
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
