import { watch } from "vue";
import * as state from "./playerState";
import { usePlayer } from "./usePlayer";

let started = false;

/**
 * Publish the current track to the OS (Windows media controls / hardware media
 * keys) through the Media Session API. Chromium in WebView2 implements it, so
 * audio-only playback can be controlled while the window is in the background.
 */
export function useMediaSession(): void {
  if (started) return;
  if (typeof navigator === "undefined") return;
  if (!("mediaSession" in navigator) || typeof MediaMetadata === "undefined") {
    return;
  }
  started = true;

  const player = usePlayer();
  const media = (): HTMLMediaElement | null => state.activeMedia.value;

  function setHandler(action: MediaSessionAction, handler: MediaSessionActionHandler): void {
    try {
      navigator.mediaSession.setActionHandler(action, handler);
    } catch {
      // Action not supported by this WebView: ignore it.
    }
  }

  watch(
    () => state.currentVideo.value,
    (video) => {
      if (!video || !state.isOpen.value) {
        navigator.mediaSession.metadata = null;
        return;
      }

      navigator.mediaSession.metadata = new MediaMetadata({
        title: video.title,
        artist: video.channelName,
        album: "r-tube",
        artwork: video.thumbnailUrl
          ? [{ src: video.thumbnailUrl, sizes: "480x360", type: "image/jpeg" }]
          : [],
      });
    },
    { immediate: true },
  );

  watch(
    () => state.status.value,
    (status) => {
      navigator.mediaSession.playbackState =
        status === "playing" ? "playing" : "paused";
    },
    { immediate: true },
  );

  watch(
    [() => state.currentTime.value, () => state.currentVideo.value?.duration],
    ([position, duration]) => {
      if (!duration || duration <= 0 || !Number.isFinite(position)) return;
      try {
        navigator.mediaSession.setPositionState({
          duration,
          position: Math.max(0, Math.min(position, duration)),
          playbackRate: 1,
        });
      } catch {
        // Ignore incorrect position states while the stream settles.
      }
    },
  );

  setHandler("play", () => {
    void media()?.play().catch(() => {});
  });
  setHandler("pause", () => media()?.pause());
  setHandler("stop", () => media()?.pause());
  setHandler("previoustrack", () => {
    if (media()) player.seek(0);
  });
  setHandler("nexttrack", () => player.playNext());
  setHandler("seekbackward", (details) => {
    const element = media();
    if (element) player.seek(Math.max(0, element.currentTime - (details.seekOffset ?? 10)));
  });
  setHandler("seekforward", (details) => {
    const element = media();
    if (element) player.seek(element.currentTime + (details.seekOffset ?? 10));
  });
  setHandler("seekto", (details) => {
    if (typeof details.seekTime === "number") player.seek(details.seekTime);
  });
}
