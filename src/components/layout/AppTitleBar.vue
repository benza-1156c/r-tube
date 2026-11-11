<script setup lang="ts">
import { computed } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { AudioLines, Clapperboard } from "@lucide/vue";
import WindowControls from "@/components/layout/WindowControls.vue";
import { useSettings, type PlaybackMode } from "@/composables/useSettings";

const { settings, updateSettings } = useSettings();

const playbackMode = computed<PlaybackMode>(() => settings.value.playbackMode);

const modes: {
  value: PlaybackMode;
  label: string;
  icon: typeof Clapperboard;
  title: string;
}[] = [
  {
    value: "normal",
    label: "Video",
    icon: Clapperboard,
    title: "Normal playback (video + audio)",
  },
  {
    value: "audio-only",
    label: "Audio",
    icon: AudioLines,
    title: "Audio only",
  },
];

function setPlaybackMode(mode: PlaybackMode): void {
  if (settings.value.playbackMode === mode) return;
  void updateSettings({ playbackMode: mode });
}

const isTauri = () => {
  return (
    typeof window !== "undefined" &&
    Boolean((window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__)
  );
};

async function handleTitleBarDblClick(e: MouseEvent) {
  const target = e.target as HTMLElement | null;
  if (target?.closest("button, [data-tauri-drag-region='false']")) {
    return;
  }
  if (!isTauri()) return;
  try {
    const win = getCurrentWindow();
    await win.toggleMaximize();
  } catch (err) {
    console.warn("Toggle maximize failed:", err);
  }
}
</script>

<template>
  <header
    data-tauri-drag-region
    class="flex h-8 w-full select-none items-center justify-between border-b border-sidebar-border bg-background/95 z-50 shrink-0"
    @dblclick="handleTitleBarDblClick"
  >
    <!-- Left: playback mode (video / audio only) -->
    <div
      data-tauri-drag-region="false"
      class="flex h-full items-center px-2"
      @mousedown.stop
    >
      <div
        class="inline-flex items-center gap-0.5 rounded-md p-0.5"
        role="group"
        aria-label="Playback mode"
      >
        <button
          v-for="mode in modes"
          :key="mode.value"
          type="button"
          :title="mode.title"
          :aria-label="mode.title"
          :aria-pressed="playbackMode === mode.value"
          class="inline-flex h-6 cursor-pointer items-center gap-1.5 rounded px-2 text-[11px] font-medium transition-colors"
          :class="
            playbackMode === mode.value
              ? 'bg-sidebar-accent text-sidebar-accent-foreground'
              : 'text-muted-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground'
          "
          @click="setPlaybackMode(mode.value)"
        >
          <component :is="mode.icon" class="h-3.5 w-3.5" />
          <span>{{ mode.label }}</span>
        </button>
      </div>
    </div>

    <!-- Center Drag Region -->
    <div
      data-tauri-drag-region
      class="flex-1 h-full cursor-default"
    />

    <!-- Right: Window Controls -->
    <WindowControls />
  </header>
</template>
