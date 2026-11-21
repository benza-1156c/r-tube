import type { AppSettings } from "@/composables/useSettings";

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

export type AnySettingDefinition =
  | SettingDefinition<"audioLanguage">
  | SettingDefinition<"defaultResolution">
  | SettingDefinition<"playbackMode">
  | SettingDefinition<"volume">;