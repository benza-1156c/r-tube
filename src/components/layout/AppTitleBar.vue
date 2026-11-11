<script setup lang="ts">
import { computed } from "vue";
import { useRouter, useRoute } from "vue-router";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { ArrowLeft, AudioLines, Clapperboard } from "@lucide/vue";
import WindowControls from "@/components/layout/WindowControls.vue";
import { useSettings, type PlaybackMode } from "@/composables/useSettings";
import { usePlaybackModeNotice } from "@/composables/usePlaybackModeNotice";
import { usePlayer } from "@/composables/usePlayer";
import { withResumeAt } from "@/composables/useVideoPage";
import { goBackInMode } from "@/router";

const router = useRouter();
const route = useRoute();
const { settings, updateSettings } = useSettings();
const { notifyPlaybackMode } = usePlaybackModeNotice();
const player = usePlayer();

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
  // Confirm the switch on screen: the buttons themselves are too small and far
  // from where the user is looking.
  notifyPlaybackMode(mode);
  void updateSettings({ playbackMode: mode });

  const currentPath = route.path;
  const currentId = (route.params as Record<string, any>)?.id as
    | string
    | undefined;
  if (!currentId) return;

  // Hand the other engine the exact position this one is at, so the switch
  // continues from the same second.
  const query = withResumeAt(route.query, player.currentTime.value || 0);

  if (mode === "audio-only" && currentPath.startsWith("/watch/")) {
    void router.push({
      path: `/listen/${currentId}`,
      query,
    });
  } else if (mode === "normal" && currentPath.startsWith("/listen/")) {
    void router.push({
      path: `/watch/${currentId}`,
      query,
    });
  }
}

function goBack(): void {
  goBackInMode();
}

const isTauri = () => {
  return (
    typeof window !== "undefined" &&
    Boolean(
      (window as unknown as { __TAURI_INTERNALS__?: unknown })
        .__TAURI_INTERNALS__,
    )
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
    <!-- Left: back + playback mode (video / audio only) -->
    <div
      data-tauri-drag-region="false"
      class="flex h-full items-center gap-1 px-2"
      @mousedown.stop
    >
      <button
        type="button"
        title="Back"
        aria-label="Back"
        class="inline-flex h-6 w-6 cursor-pointer items-center justify-center rounded text-muted-foreground transition-colors hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
        @click="goBack"
      >
        <ArrowLeft class="h-3.5 w-3.5" />
      </button>

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
    <div data-tauri-drag-region class="flex-1 h-full cursor-default" />

    <!-- Right: Window Controls -->
    <WindowControls />
  </header>
</template>
