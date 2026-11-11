import { computed, ref, watch } from "vue";

/**
 * Local library: Watch Later, liked videos and resume positions.
 *
 * YouTube keeps these server-side, but the app only exposes a read-only history
 * feed, so the desktop library is persisted locally (localStorage survives
 * restarts in WebView2).
 */

export interface SavedVideo {
  id: string;
  title: string;
  thumbnailUrl: string;
  channelName: string;
  duration: string;
  savedAt: number;
}

export interface LibraryVideoInput {
  id: string;
  title: string;
  thumbnailUrl?: string;
  channelName?: string;
  duration?: number | string;
}

interface LibraryState {
  watchLater: SavedVideo[];
  liked: SavedVideo[];
  resume: Record<string, number>;
  channels: string[];
}

const STORAGE_KEY = "rtube.library.v1";

function emptyState(): LibraryState {
  return { watchLater: [], liked: [], resume: {}, channels: [] };
}

function load(): LibraryState {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return emptyState();
    const parsed = JSON.parse(raw) as Partial<LibraryState>;
    return {
      watchLater: Array.isArray(parsed.watchLater) ? parsed.watchLater : [],
      liked: Array.isArray(parsed.liked) ? parsed.liked : [],
      resume:
        parsed.resume && typeof parsed.resume === "object" ? parsed.resume : {},
      channels: Array.isArray(parsed.channels) ? parsed.channels : [],
    };
  } catch {
    return emptyState();
  }
}

const state = ref<LibraryState>(load());

let saveTimer: ReturnType<typeof setTimeout> | null = null;
watch(
  state,
  () => {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(state.value));
      } catch (err) {
        console.warn("Could not persist the local library:", err);
      }
    }, 200);
  },
  { deep: true },
);

function toSaved(video: LibraryVideoInput): SavedVideo {
  const duration =
    typeof video.duration === "number"
      ? formatDuration(video.duration)
      : (video.duration ?? "");

  return {
    id: video.id,
    title: video.title,
    thumbnailUrl: video.thumbnailUrl ?? "",
    channelName: video.channelName ?? "",
    duration,
    savedAt: Date.now(),
  };
}

function formatDuration(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds <= 0) return "";
  const total = Math.floor(seconds);
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const secs = total % 60;
  const mm = String(minutes).padStart(hours > 0 ? 2 : 1, "0");
  return hours > 0
    ? `${hours}:${mm}:${String(secs).padStart(2, "0")}`
    : `${minutes}:${String(secs).padStart(2, "0")}`;
}

export function useLibrary() {
  function isSaved(id: string): boolean {
    return state.value.watchLater.some((video) => video.id === id);
  }

  function toggleSave(video: LibraryVideoInput): boolean {
    const index = state.value.watchLater.findIndex(
      (entry) => entry.id === video.id,
    );
    if (index >= 0) {
      state.value.watchLater.splice(index, 1);
      return false;
    }
    state.value = {
      ...state.value,
      watchLater: [toSaved(video), ...state.value.watchLater],
    };
    return true;
  }

  function isLiked(id: string): boolean {
    return state.value.liked.some((video) => video.id === id);
  }

  function toggleLike(video: LibraryVideoInput): boolean {
    const index = state.value.liked.findIndex((entry) => entry.id === video.id);
    if (index >= 0) {
      state.value.liked.splice(index, 1);
      return false;
    }
    state.value = {
      ...state.value,
      liked: [toSaved(video), ...state.value.liked],
    };
    return true;
  }

  /** Remember where a video stopped, unless it is at the very start or end. */
  function setResume(id: string, seconds: number, duration: number): void {
    if (!id || !Number.isFinite(seconds)) return;
    if (seconds < 8) {
      delete state.value.resume[id];
      return;
    }
    if (duration > 0 && seconds > duration - 10) {
      delete state.value.resume[id];
      return;
    }
    state.value.resume[id] = Math.floor(seconds);
  }

  function getResume(id: string): number {
    return state.value.resume[id] ?? 0;
  }

  function clearResume(id: string): void {
    delete state.value.resume[id];
  }

  function isSubscribedChannel(id: string): boolean {
    return state.value.channels.includes(id);
  }

  function setSubscribedChannel(id: string, subscribed: boolean): void {
    const channels = state.value.channels.filter((entry) => entry !== id);
    if (subscribed) channels.push(id);
    state.value = { ...state.value, channels };
  }

  return {
    watchLater: computed(() => state.value.watchLater),
    liked: computed(() => state.value.liked),
    isSaved,
    toggleSave,
    isLiked,
    toggleLike,
    setResume,
    getResume,
    clearResume,
    isSubscribedChannel,
    setSubscribedChannel,
  };
}
