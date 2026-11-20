import { computed, nextTick, shallowRef } from "vue";
import {
  commands,
  type CaptionTrack,
  type PlayerResponse_Deserialize,
  type VideoItem_Deserialize,
} from "@/bindings";
import { router } from "@/router";
import { useSettings } from "./useSettings";
import * as state from "./playerState";
import {
  reportPlaybackTime,
  startPlaybackHistory,
  stopPlaybackHistory,
} from "./usePlaybackHistory";
import type {
  CaptionOption,
  CurrentVideoMeta,
  PlayerStatus,
} from "./playerState";
import { fetchUpNext } from "./useUpNext";

export type { CaptionOption, CurrentVideoMeta, PlayerStatus };

/** A media element that can host an HLS playback engine. */
type MediaElement = HTMLVideoElement;

/**
 * Watch page slot the player is teleported into.
 */
const videoElement = shallowRef<MediaElement | null>(null);
const watchAnchor = shallowRef<HTMLElement | null>(null);

const activeBlobUrls: string[] = [];
const boundVideos = new WeakSet<HTMLVideoElement>();
/** Audio track lists already wired for language selection. */
const boundAudioTrackLists = new WeakSet<object>();

let autoplayTimer: ReturnType<typeof setInterval> | null = null;
let isRecovering = false;
let pendingResume: { videoId: string; at: number; autoplay: boolean } | null =
  null;

/**
 * Incremented by every load and by `closePlayer`. A load only publishes its
 * result while it still owns the latest token, so a slow response can never
 * overwrite a newer video or revive the source after the player was closed.
 */
let loadToken = 0;

/**
 * Safety net for a source that never produces data.
 *
 * hls.js reports a fatal error during the initial load while the player is
 * still in "loading". Treating those as stale teardown errors is necessary,
 * but leaving the watchdog out is what let a real failure leave the spinner
 * on screen forever.
 */
let loadWatchdog: ReturnType<typeof setTimeout> | null = null;
let lastLoadError: unknown = null;
let autoRecovered = false;

function clearLoadWatchdog(): void {
  if (loadWatchdog) {
    clearTimeout(loadWatchdog);
    loadWatchdog = null;
  }
}

function playbackErrorDetail(error: unknown): string {
  if (!error) return "";
  if (typeof error === "string") return error;
  const value = error as {
    error?: { message?: string; data?: { details?: string } };
    message?: string;
    data?: { details?: string };
  };
  const inner = value.error ?? value;
  return inner.data?.details ?? inner.message ?? "";
}

/**
 * Whether the app has received a user gesture.
 *
 * The media element is muted so autoplay is allowed, then unmuted as soon as
 * playback starts when the user has already interacted. Listening from module
 * load, rather than when the player mounts, also covers the click that
 * navigated to the watch page.
 */
let hasUserGesture = false;
/** The first play of a newly loaded source should restore sound. */
let restoreSoundOnPlay = false;
if (typeof document !== "undefined") {
  const markUserGesture = (): void => {
    hasUserGesture = true;
  };
  document.addEventListener("pointerdown", markUserGesture, {
    capture: true,
    passive: true,
  });
  document.addEventListener("keydown", markUserGesture, { capture: true });
  document.addEventListener("touchstart", markUserGesture, {
    capture: true,
    passive: true,
  });
}

function revokeBlobUrls(): void {
  while (activeBlobUrls.length > 0) {
    const url = activeBlobUrls.pop();
    if (url) {
      try {
        URL.revokeObjectURL(url);
      } catch {}
    }
  }
}

/**
 * Load every caption track in parallel, off the playback critical path.
 *
 * hls.js is not allowed to add the manifest's subtitle group (the proxy
 * removes it), so these tracks are the player's only captions.
 */
async function loadCaptionTracks(
  captions: CaptionTrack[],
  isStale: () => boolean,
): Promise<void> {
  const results = await Promise.all(
    captions.map(async (caption): Promise<CaptionOption | null> => {
      try {
        const subRes = await commands.getSubtitleContent(caption.vttUrl);
        if (isStale() || subRes.status !== "ok" || !subRes.data) return null;
        return {
          id: caption.languageCode,
          name: caption.name,
          language: caption.languageCode,
          url: URL.createObjectURL(
            new Blob([subRes.data], { type: "text/vtt" }),
          ),
        };
      } catch (err) {
        console.warn(
          "Could not load caption track:",
          caption.languageCode,
          err,
        );
        return null;
      }
    }),
  );

  const loaded = results.filter((track): track is CaptionOption => !!track);
  // A newer load owns the player now: release the blobs this one created
  // instead of leaving them for the next video.
  if (isStale()) {
    for (const track of loaded) URL.revokeObjectURL(track.url);
    return;
  }

  activeBlobUrls.push(...loaded.map((track) => track.url));
  state.captionTracks.value = loaded;
}

export function usePlayer() {
  const { settings, updateSettings } = useSettings();
  const isPlaying = computed<boolean>(() => state.status.value === "playing");

  /** Apply the resolved default quality once the element knows its renditions. */
  function applyResolution(el: HTMLVideoElement): void {
    const pref = settings.value?.defaultResolution;
    const renditions = (el as any).videoRenditions;
    if (!renditions || !pref || pref === "auto") return;
    const target = parseInt(pref, 10);
    const list = Array.from(renditions) as any[];
    const idx = list.findIndex((r) => r.height === target);
    const chosen = idx !== -1 ? idx : list.findIndex((r) => r.height <= target);
    if (chosen !== -1) renditions.selectedIndex = chosen;
  }

  /** Base part of a language tag: `"th-TH"` → `"th"`, lowercased. */
  function languageBase(tag: string): string {
    return tag.trim().toLowerCase().split(/[-_]/)[0] ?? "";
  }

  /**
   * The language the user picked in Settings. `null` means the original /
   * default track, which needs no switching.
   */
  function preferredAudioLanguage(): string | null {
    const value = (settings.value?.audioLanguage ?? "").trim();
    if (!value || value.toLowerCase() === "original") return null;
    return languageBase(value);
  }

  interface AudioTrackLike {
    id?: string;
    kind?: string;
    label?: string;
    language?: string;
    enabled: boolean;
  }

  function audioTrackMatches(track: AudioTrackLike, wanted: string): boolean {
    if (track.language && languageBase(track.language) === wanted) return true;
    if (track.label && languageBase(track.label) === wanted) return true;
    return track.id ? languageBase(track.id) === wanted : false;
  }

  /**
   * Switch the HLS video to the user's configured audio language.
   *
   * Read-only by design: playback never writes the chosen track back into the
   * settings. If the stream has no dub in that language, the original/default
   * track keeps playing and the preference is left untouched for the user to
   * change themselves.
   */
  function applyAudioLanguage(el: HTMLVideoElement): void {
    const wanted = preferredAudioLanguage();
    if (!wanted) return;

    const list = (el as any).audioTracks as
      | { length: number; [index: number]: AudioTrackLike }
      | undefined;
    if (!list || !list.length) return;

    // The list only exists once the HLS engine is attached, so wire its
    // `addtrack` here instead of when the element is mounted. New languages
    // reported by a manifest then get picked without another media event.
    if (!boundAudioTrackLists.has(list)) {
      boundAudioTrackLists.add(list);
      (list as any).addEventListener?.("addtrack", () =>
        applyAudioLanguage(el),
      );
    }

    const tracks = Array.from(list as ArrayLike<AudioTrackLike>);
    const matches = tracks.filter((track) => audioTrackMatches(track, wanted));
    // Prefer the manifest's main track over descriptive/alternative versions.
    const match =
      matches.find((track) => track.kind === "main") ?? matches[0];
    if (!match) return;

    const enabled = tracks.filter((track) => track.enabled);
    if (match.enabled && enabled.length === 1) return;

    for (const track of tracks) track.enabled = track === match;
  }

  /**
   * Seek to the position a hand-off handed over, then start playing.
   *
   * The video is mounted on demand, so `loadedmetadata` can fire before (or
   * long after) the load call: the position is kept here until the element that
   * actually holds the stream reports its metadata.
   */
  function applyPendingResume(el: HTMLVideoElement): void {
    if (!pendingResume) return;
    if (pendingResume.videoId !== state.currentVideo.value?.id) return;

    const { at, autoplay } = pendingResume;
    pendingResume = null;
    if (at > 0 && Number.isFinite(at)) {
      try {
        el.currentTime = at;
      } catch {}
    }
    if (autoplay) {
      el.play().catch((err: unknown) => console.warn("Play interrupted:", err));
    }
  }

  function attachVideo(el: MediaElement | null): void {
    videoElement.value = el;

    if (el && !boundVideos.has(el)) {
      boundVideos.add(el);

      el.volume = settings.value?.volume ?? 0.75;

      // Mobile browsers block autoplay unless the element starts muted.
      // Unmute on the first user interaction so audio plays normally after
      // the initial gesture. The `muted` attribute is set on the element in
      // the template; here we lift it once the user touches/clicks anything.
      // When a gesture already happened (for example, the navigation click),
      // `onPlay` restores the sound as soon as playback starts instead.
      if (!hasUserGesture) {
        const unmuteOnInteraction = (): void => {
          const volume = settings.value?.volume ?? 0.75;
          if (el.muted && volume > 0) {
            el.muted = false;
            el.volume = volume;
          }
          document.removeEventListener("click", unmuteOnInteraction);
          document.removeEventListener("touchstart", unmuteOnInteraction);
          document.removeEventListener("keydown", unmuteOnInteraction);
        };
        document.addEventListener("click", unmuteOnInteraction, { once: true });
        document.addEventListener("touchstart", unmuteOnInteraction, {
          once: true,
        });
        document.addEventListener("keydown", unmuteOnInteraction, {
          once: true,
        });
      }

      el.addEventListener("loadeddata", () => {
        clearLoadWatchdog();
        if (state.status.value === "loading") {
          state.status.value = "paused";
        }
      });

      el.addEventListener("volumechange", () => {
        state.isMuted.value = el.muted || el.volume === 0;
        updateSettings({ volume: el.volume });
      });

      el.addEventListener("loadedmetadata", () => {
        applyResolution(el);
        applyAudioLanguage(el);
        applyPendingResume(el);
      });
      el.addEventListener("loadeddata", () => {
        applyResolution(el);
        applyAudioLanguage(el);
      });
    }

    if (el) {
      state.activeMedia.value = el;
    } else if (state.activeEngine.value !== "audio") {
      state.activeMedia.value = null;
    }
  }

  /** Tell the player which slot hosts it (null = mini player). */
  function setWatchAnchor(el: HTMLElement | null): void {
    watchAnchor.value = el;
    state.isMiniPlayer.value = !el;
  }

  function cancelAutoplay(): void {
    if (autoplayTimer) {
      clearInterval(autoplayTimer);
      autoplayTimer = null;
    }
    state.autoplayCountdown.value = null;
  }

  function toggleAutoplay(): void {
    state.isAutoplayEnabled.value = !state.isAutoplayEnabled.value;
    if (!state.isAutoplayEnabled.value) {
      cancelAutoplay();
    }
  }

  function setUpNextVideo(video: VideoItem_Deserialize | null): void {
    state.upNextVideo.value = video;
  }

  function playNext(): void {
    cancelAutoplay();
    const next = state.upNextVideo.value;
    if (!next) return;

    // A Mix is a queue: dropping `list` would turn the next item into a
    // standalone video and end the Mix after one track.
    const list = router.currentRoute.value.query.list;
    const query: Record<string, string> =
      typeof list === "string" && list ? { list } : {};

    const base = state.activeEngine.value === "audio" ? "/listen" : "/watch";
    void router.push({ path: `${base}/${next.id}`, query });
  }

  interface LoadOptions {
    /** Position to resume from, in seconds (mode hand-off). */
    resumeAt?: number;
    autoplay?: boolean;
    /** Metadata to show immediately while the stream URL is being fetched. */
    meta?: Partial<CurrentVideoMeta>;
    preserveMeta?: boolean;
  }

  async function loadStream(
    videoId: string,
    options: LoadOptions = {},
  ): Promise<void> {
    const token = ++loadToken;
    const isStale = (): boolean => token !== loadToken;
    const reloadingSameVideo = state.currentVideo.value?.id === videoId;

    clearLoadWatchdog();
    lastLoadError = null;
    cancelAutoplay();
    // A recovery reloads the same video: the queue the page resolved is still
    // valid and must survive, or the next track would be lost.
    if (!reloadingSameVideo) {
      state.upNextVideo.value = null;
    }

    // Stop the previous video's audio right away, but do not tear the media
    // down here: detaching the engine and clearing the element's src while the
    // next video is still being fetched fires a media error, and that error
    // made `onError` call `recoverStream()` for the *old* video — which bumps
    // the load token and cancels the load that was just started. Assigning
    // `currentSrc` below replaces the stream and tears the old one down
    // cleanly.
    const previous = videoElement.value;
    if (previous && !previous.paused) {
      try {
        previous.pause();
      } catch {}
    }

    // Save where the replaced video stopped, then stop reporting for it.
    reportPlaybackTime();
    stopPlaybackHistory(false);

    state.status.value = "loading";
    state.errorMessage.value = null;
    if (!options.resumeAt) {
      state.currentTime.value = 0;
    }

    if (!options.preserveMeta) {
      state.currentVideo.value = {
        id: videoId,
        title: options.meta?.title || "Loading video...",
        channelName: options.meta?.channelName || "",
        channelAvatarUrl: options.meta?.channelAvatarUrl || "",
        thumbnailUrl: options.meta?.thumbnailUrl || "",
        duration: options.meta?.duration || 0,
        viewCount: "",
        description: "",
      };
    }

    try {
      const infoRes = await commands.getVideoInfo(videoId);
      if (isStale()) {
        return;
      }
      if (infoRes.status !== "ok") {
        throw new Error(
          infoRes.error.message || "Failed to fetch video stream info",
        );
      }

      const info: PlayerResponse_Deserialize = infoRes.data;

      state.currentVideo.value = {
        id: info.id || videoId,
        title: info.title || state.currentVideo.value?.title || "",
        channelName: info.channelName || state.currentVideo.value?.channelName || "",
        channelId: info.channelId || state.currentVideo.value?.channelId,
        channelAvatarUrl: state.currentVideo.value?.channelAvatarUrl || "",
        thumbnailUrl: info.thumbnailUrl || state.currentVideo.value?.thumbnailUrl || "",
        duration: Number(info.lengthSeconds) || state.currentVideo.value?.duration || 0,
        viewCount: info.viewCount,
        description: info.description,
      };

      if (!info.hlsManifestUrl) {
        throw new Error("This video has no HLS stream available");
      }

      pendingResume =
        options.resumeAt && options.resumeAt > 0
          ? {
              videoId,
              at: options.resumeAt,
              autoplay: options.autoplay ?? true,
            }
          : null;

      state.currentSrc.value = info.hlsManifestUrl;
      restoreSoundOnPlay = hasUserGesture;
      armLoadWatchdog(token);

      // Captions load after the source is assigned and in parallel, so
      // playback never waits on them. A recovery of the same video keeps the
      // tracks it already has instead of downloading them again.
      if (!reloadingSameVideo || state.captionTracks.value.length === 0) {
        revokeBlobUrls();
        state.captionTracks.value = [];
        if (info.captions && info.captions.length > 0) {
          void loadCaptionTracks(info.captions, isStale);
        }
      }

      // Resolve the audio-only URL while the video is still playing: the Rust
      // side reuses this player response and caches the result, so the switch
      // to audio mode only pays for the media fetch itself.
      void commands.getAudioStream(videoId).catch(() => {});
    } catch (err: unknown) {
      // A cancelled load is not a failure: it no longer owns this player.
      if (isStale()) {
        return;
      }
      console.error("Error loading video:", err);
      state.status.value = "error";
      state.errorMessage.value =
        err instanceof Error ? err.message : "Failed to play video";
    }
  }

  async function loadVideo(
    videoId: string,
    options: LoadOptions = {},
  ): Promise<void> {
    autoRecovered = false;
    state.videoReleased.value = false;
    state.activeEngine.value = "video";
    state.isOpen.value = true;
    state.isMiniPlayer.value = false;
    isRecovering = false;

    // The audio engine owns the shared state while it is active; taking the
    // engine back has to start from a clean slate.
    state.captionTracks.value = [];

    const resumeAt = options.resumeAt ?? 0;
    await loadStream(videoId, { ...options, resumeAt });
  }

  /**
   * Fail a load that has received no data, instead of leaving the spinner up.
   *
   * A stale error from the previous source is ignored while loading (that is
   * why `onError` does not act immediately), so this timer is the only place
   * that can distinguish "still starting" from "never starting".
   */
  function armLoadWatchdog(token: number): void {
    clearLoadWatchdog();
    loadWatchdog = setTimeout(() => {
      loadWatchdog = null;
      if (
        token !== loadToken ||
        state.status.value !== "loading" ||
        !state.isOpen.value
      ) {
        return;
      }

      const el = videoElement.value;
      // Data or playback arrived: only the status bookkeeping was missed.
      if (el && (el.readyState >= 2 || !el.paused || el.currentTime > 0)) {
        state.status.value = el.paused ? "paused" : "playing";
        return;
      }

      const detail = playbackErrorDetail(lastLoadError);
      console.error(
        "Playback did not start:",
        detail || "(no media error reported)",
        lastLoadError,
      );
      state.status.value = "error";
      state.errorMessage.value = detail
        ? `Failed to stream video: ${detail}`
        : "Failed to stream video. Please try again.";

      // One automatic retry; a deterministic failure should end on the
      // visible error instead of reloading forever.
      if (!autoRecovered) {
        autoRecovered = true;
        void recoverStream({ manual: true });
      }
    }, 8000);
  }

  /**
   * Reload the HLS session at the current position.
   *
   * A playlist expires after a few hours; a fresh one is minted by the proxy,
   * so the source has to be re-assigned for the engine to pick it up.
   */
  async function recoverStream(
    opts: { manual?: boolean } = {},
  ): Promise<boolean> {
    const video = state.currentVideo.value;
    const element = videoElement.value;
    if (!video || !element || isRecovering) {
      return false;
    }
    // Never fight a load that is already running: the error that asks for this
    // recovery usually comes from the previous video being switched out.
    if (state.status.value === "loading") {
      return false;
    }
    if (!opts.manual && state.status.value !== "error") {
      return false;
    }

    isRecovering = true;
    const resumeAt = element.currentTime;
    const wasPlaying = !element.paused;
    try {
      await loadStream(video.id, {
        resumeAt,
        autoplay: wasPlaying,
        preserveMeta: true,
      });
      return state.status.value !== "error";
    } finally {
      isRecovering = false;
    }
  }

  function onPlay(): void {
    cancelAutoplay();
    // Autoplay starts muted so the browser allows it; the first play of a new
    // source restores sound when a gesture already happened. A later play is
    // left alone so a mute chosen by the user survives pause/resume.
    if (restoreSoundOnPlay) {
      restoreSoundOnPlay = false;
      const el = videoElement.value;
      const volume = settings.value?.volume ?? 0.75;
      if (el && el.muted && volume > 0) {
        el.muted = false;
        el.volume = volume;
      }
    }
    state.status.value = "playing";
    clearLoadWatchdog();
    autoRecovered = false;
    lastLoadError = null;
    const videoId = state.currentVideo.value?.id;
    if (videoId) {
      startPlaybackHistory(videoId, () => videoElement.value?.currentTime ?? 0);
    }
  }

  function onPause(): void {
    if (state.status.value !== "ended") {
      state.status.value = "paused";
    }
    reportPlaybackTime();
  }

  function onTimeUpdate(e: Event): void {
    const el = e.target as HTMLMediaElement | null;
    if (!el) return;
    state.currentTime.value = el.currentTime;

    // Some HLS playlists are served as a live/DVR window and never emit
    // `ended`; treat "reached the known duration" as the end so a Mix keeps
    // playing. The status guard makes this fire once.
    const total = state.currentVideo.value?.duration || 0;
    if (
      total > 0 &&
      !el.paused &&
      !el.seeking &&
      state.status.value !== "ended" &&
      el.currentTime >= total - 0.75
    ) {
      onEnded();
    }
  }

  function onWaiting(): void {
    if (state.status.value !== "error" && state.status.value !== "ended") {
      state.status.value = "buffering";
    }
  }

  function onPlaying(): void {
    state.status.value = "playing";
    clearLoadWatchdog();
    autoRecovered = false;
    lastLoadError = null;
  }

  async function onEnded(): Promise<void> {
    if (state.status.value === "ended") return;

    state.status.value = "ended";
    reportPlaybackTime();

    if (!state.isAutoplayEnabled.value) return;

    // The page normally resolved the queue, but a Mix must keep playing even
    // when that result was lost (recovery reload, failed fetch, …).
    const videoId = state.currentVideo.value?.id;
    const list = router.currentRoute.value.query.list;
    const playlistId = typeof list === "string" && list ? list : null;
    const next =
      state.upNextVideo.value ??
      (videoId ? await fetchUpNext(videoId, playlistId) : null);
    if (!next) {
      console.warn("[autoplay] no next video resolved for", videoId);
      return;
    }
    state.upNextVideo.value = next;

    cancelAutoplay();
    state.autoplayCountdown.value = 4;
    autoplayTimer = setInterval(() => {
      if (state.autoplayCountdown.value === null) {
        cancelAutoplay();
        return;
      }
      state.autoplayCountdown.value -= 1;
      if (state.autoplayCountdown.value <= 0) {
        cancelAutoplay();
        playNext();
      }
    }, 1000);
  }

  function onError(e: Event | unknown): void {
    if (!state.currentSrc.value || !state.isOpen.value) {
      return;
    }
    // Switching videos tears the old stream down, and that teardown can emit a
    // media error. It is not a playback failure and must never cancel the load
    // that is already in flight. Keep the error for the watchdog, which knows
    // whether this load ever produced data.
    if (state.status.value === "loading") {
      lastLoadError = e;
      console.warn(
        "Media error while loading:",
        e,
        videoElement.value?.error,
      );
      return;
    }

    clearLoadWatchdog();
    const target = (e as Event)?.target as HTMLElement | null;
    if (target?.tagName?.toLowerCase() === "track") {
      return;
    }
    const mediaErr = videoElement.value?.error;
    if (mediaErr && mediaErr.code === 1) {
      return;
    }

    console.error("Video player error:", e, mediaErr);
    state.status.value = "error";
    state.errorMessage.value = "Failed to stream video. Please try again.";

    // A network/source failure usually means the playlist expired.
    if (!mediaErr || mediaErr.code === 2 || mediaErr.code === 4) {
      void recoverStream({ manual: true });
    }
  }

  /** Pause/play whichever engine currently holds the media element. */
  function togglePlay(): void {
    const el = state.activeMedia.value;
    if (!el) return;
    if (el.paused) {
      el.play().catch((err: unknown) => console.warn("Play interrupted:", err));
    } else {
      el.pause();
    }
  }

  function seek(seconds: number): void {
    const el = state.activeMedia.value;
    if (!el) return;
    try {
      el.currentTime = seconds;
    } catch {}
    state.currentTime.value = seconds;
  }

  function setVolume(volume: number): void {
    const el = state.activeMedia.value;
    if (el) {
      el.volume = volume;
      if (volume > 0 && el.muted) el.muted = false;
    }
    void updateSettings({ volume });
  }

  /** Picture-in-picture for the video element (no-op for audio). */
  async function togglePictureInPicture(): Promise<void> {
    // Video.js renders the real <video> inside its custom element; prefer it
    // when the wrapper does not expose the PiP API itself.
    const wrapper = videoElement.value as unknown as HTMLVideoElement | null;
    const inner = document.querySelector(".rt-player video") as
      | HTMLVideoElement
      | null;
    const el =
      wrapper && typeof wrapper.requestPictureInPicture === "function"
        ? wrapper
        : inner;
    if (!el) return;
    try {
      if (document.pictureInPictureElement) {
        await document.exitPictureInPicture();
      } else {
        await el.requestPictureInPicture();
      }
    } catch (err) {
      console.warn("Picture-in-picture failed:", err);
    }
  }

  function toggleMute(): void {    const el = state.activeMedia.value;
    if (!el) return;
    if (el.muted || el.volume === 0) {
      el.muted = false;
      el.volume = settings.value?.volume ?? 0.75;
    } else {
      el.muted = true;
    }
  }

  /**
   * Tear the Video.js tree down and release its engine.
   *
   * `closePlayer` and the audio engine both need this: the HLS instance, its
   * SourceBuffers and the decoder only go away when the element unmounts.
   */
  async function releaseVideoEngine(): Promise<void> {
    clearLoadWatchdog();
    const el = videoElement.value;
    if (el) {
      try {
        el.pause();
      } catch {}
      try {
        (el as any).engine?.stopLoad?.();
      } catch {}
    }

    state.videoReleased.value = true;
    // Unmount first, then clear the source: clearing it while the element is
    // still alive makes the HLS adapter treat the empty URL as a playlist.
    await nextTick();
    state.currentSrc.value = "";
    state.captionTracks.value = [];
  }

  /**
   * Close the player and release everything the stream holds.
   */
  function closePlayer(): void {
    cancelAutoplay();
    clearLoadWatchdog();
    autoRecovered = false;
    lastLoadError = null;
    state.upNextVideo.value = null;
    loadToken++;
    state.isOpen.value = false;
    state.status.value = "idle";
    state.currentVideo.value = null;
    state.errorMessage.value = null;
    isRecovering = false;
    restoreSoundOnPlay = false;
    revokeBlobUrls();
    state.captionTracks.value = [];

    // Remember the last position before dropping the session.
    reportPlaybackTime();
    stopPlaybackHistory(false);

    if (state.activeEngine.value === "audio") {
      // Closing the audio player must not clear the video source by accident.
      const media = state.activeMedia.value;
      if (media instanceof HTMLAudioElement) {
        try {
          media.pause();
          media.removeAttribute("src");
          media.load();
        } catch {}
      }
      state.activeEngine.value = null;
      return;
    }

    state.activeEngine.value = null;
    void releaseVideoEngine();
  }

  function minimizeToMiniPlayer(): void {
    state.isMiniPlayer.value = true;
  }

  function restoreFromMiniPlayer(): void {
    state.isMiniPlayer.value = false;
  }

  return {
    isOpen: state.isOpen,
    isMiniPlayer: state.isMiniPlayer,
    status: state.status,
    errorMessage: state.errorMessage,
    currentVideo: state.currentVideo,
    currentSrc: state.currentSrc,
    captionTracks: state.captionTracks,
    currentTime: state.currentTime,
    isPlaying,
    isMuted: state.isMuted,
    activeEngine: state.activeEngine,
    activeMedia: state.activeMedia,
    videoReleased: state.videoReleased,
    videoElement,
    watchAnchor,
    upNextVideo: state.upNextVideo,
    autoplayCountdown: state.autoplayCountdown,
    isAutoplayEnabled: state.isAutoplayEnabled,
    attachVideo,
    setWatchAnchor,
    loadVideo,
    recoverStream,
    onPlay,
    onPause,
    onTimeUpdate,
    onWaiting,
    onPlaying,
    onEnded,
    onError,
    togglePlay,
    seek,
    setVolume,
    toggleMute,
    togglePictureInPicture,
    closePlayer,
    releaseVideoEngine,
    minimizeToMiniPlayer,
    restoreFromMiniPlayer,
    cancelAutoplay,
    toggleAutoplay,
    playNext,
    setUpNextVideo,
  };
}
