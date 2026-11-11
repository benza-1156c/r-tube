import { readonly, ref } from "vue";
import type { PlaybackMode } from "@/bindings";

/**
 * App-wide "playback mode changed" notice.
 *
 * Only user-initiated switches should call `notifyPlaybackMode` — never the
 * initial settings hydration — otherwise the OSD flashes on app start.
 */
const HOLD_MS = 1200;

const visible = ref(false);
const mode = ref<PlaybackMode>("normal");

let hideTimer: ReturnType<typeof setTimeout> | null = null;
let currentRun = 0;

export function notifyPlaybackMode(next: PlaybackMode): void {
  mode.value = next;
  visible.value = true;

  // Rapid toggling restarts the hold instead of queueing notices.
  currentRun += 1;
  const run = currentRun;
  if (hideTimer) clearTimeout(hideTimer);

  hideTimer = setTimeout(() => {
    if (run !== currentRun) return;
    visible.value = false;
  }, HOLD_MS);
}

export function usePlaybackModeNotice() {
  return {
    visible: readonly(visible),
    mode: readonly(mode),
    notifyPlaybackMode,
  };
}
