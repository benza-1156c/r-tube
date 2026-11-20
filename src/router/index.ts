import { ref } from "vue";
import { createRouter, createWebHistory } from "vue-router";
import { routes } from "vue-router/auto-routes";
import { useSettings } from "@/composables/useSettings";

export const router = createRouter({
  history: createWebHistory(),
  routes,
  scrollBehavior() {
    return { top: 0 };
  },
});

export { routes };

export const previousPath = ref("");

router.beforeEach((_to, from) => {
  previousPath.value = from.path;
  return true;
});

interface HistoryEntry {
  path: string;
  position: number;
}

/** Visited routes, keyed by vue-router's history position. */
const entries: HistoryEntry[] = [];

function historyPosition(): number {
  const state = window.history.state as { position?: number } | null;
  return typeof state?.position === "number" ? state.position : 0;
}

router.afterEach((to) => {
  const position = historyPosition();
  // A back/forward or a `replace` reuses or rewinds positions: drop everything
  // at or after this one before recording the route that actually rendered.
  for (let i = entries.length - 1; i >= 0; i -= 1) {
    if (entries[i].position >= position) entries.splice(i, 1);
  }
  entries.push({ path: to.fullPath, position });
});

/** Whether a history entry is playable in the current playback mode. */
function suitsMode(path: string, audioOnly: boolean): boolean {
  if (path.startsWith("/watch/")) return !audioOnly;
  if (path.startsWith("/listen/")) return audioOnly;
  return true;
}

/**
 * Back, but skipping entries that belong to the other playback mode.
 *
 * Switching modes pushes both routes for the same video (watch → listen), so a
 * plain `router.back()` lands on the other mode, which immediately redirects
 * forward again. Only entries the current mode can actually show are used.
 */
export function goBackInMode(): void {
  const audioOnly = useSettings().settings.value.playbackMode !== "normal";
  const position = historyPosition();
  const earlier = entries.filter((entry) => entry.position < position);

  for (let i = earlier.length - 1; i >= 0; i -= 1) {
    if (suitsMode(earlier[i].path, audioOnly)) {
      router.go(earlier[i].position - position);
      return;
    }
  }

  void router.push("/");
}
