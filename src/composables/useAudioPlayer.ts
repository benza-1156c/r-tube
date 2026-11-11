import { computed, ref, shallowRef } from "vue";
import { commands, type CaptionTrack } from "@/bindings";
import { router } from "@/router";
import { parseVtt, type CaptionCue } from "@/lib/vtt";
import { useSettings } from "./useSettings";
import { usePlayer } from "./usePlayer";
import { fetchUpNext } from "./useUpNext";
import * as state from "./playerState";
import type { CaptionOption } from "./playerState";
import {
  reportPlaybackTime,
  startPlaybackHistory,
  stopPlaybackHistory,
} from "./usePlaybackHistory";

/**
 * Audio-only playback engine.
 *
 * The `<audio>` element lives in the app layout, not in the listen route, so a
 * track keeps playing while the user browses elsewhere. It holds a single
 * progressive stream (Opus 160 kbps), which means no video decoder, no
 * SourceBuffers and no HLS worker: a few megabytes of buffer instead of a
 * hundred-plus megabytes of decoded frames.
 */

const audioElement = shallowRef<HTMLAudioElement | null>(null);

let loadToken = 0;
let pendingSeek: number | null = null;
let autoplayOnLoad = true;
let bound = false;
let nextTimer: ReturnType<typeof setTimeout> | null = null;

/** Caption cues per language, used by the listen page overlay. */
const captionCues = shallowRef<Record<string, CaptionCue[]>>({});
const captionLanguage = ref("");
const captionBlobUrls: string[] = [];
let captionToken = 0;

function revokeCaptionUrls(): void {
  while (captionBlobUrls.length > 0) {
    const url = captionBlobUrls.pop();
    if (url) URL.revokeObjectURL(url);
  }
}

async function loadCaptions(tracks: CaptionTrack[]): Promise<void> {
  const token = ++captionToken;
  revokeCaptionUrls();
  captionCues.value = {};
  captionLanguage.value = "";

  if (tracks.length === 0) return;

  const options: CaptionOption[] = [];
  const cues: Record<string, CaptionCue[]> = {};

  for (const track of tracks) {
    try {
      const res = await commands.getSubtitleContent(track.vttUrl);
      if (token !== captionToken) return;
      if (res.status !== "ok" || !res.data) continue;

      const blobUrl = URL.createObjectURL(
        new Blob([res.data], { type: "text/vtt" }),
      );
      captionBlobUrls.push(blobUrl);
      options.push({
        id: track.languageCode,
        name: track.name,
        language: track.languageCode,
        url: blobUrl,
      });
      cues[track.languageCode] = parseVtt(res.data);
    } catch (err) {
      console.warn("Could not load caption track:", track.languageCode, err);
    }
  }

  if (token !== captionToken) return;
  state.captionTracks.value = options;
  captionCues.value = cues;
  captionLanguage.value = options[0]?.language ?? "";
}

function clearCaptions(): void {
  captionToken++;
  revokeCaptionUrls();
  state.captionTracks.value = [];
  captionCues.value = {};
  captionLanguage.value = "";
}

export function useAudioPlayer() {
  const { settings, updateSettings } = useSettings();
  const player = usePlayer();

  /** Subtitle under the artwork while the audio engine is playing. */
  const activeCue = computed<CaptionCue | null>(() => {
    const language = captionLanguage.value;
    if (!language) return null;
    const cues = captionCues.value[language];
    if (!cues) return null;
    const time = state.currentTime.value;
    return cues.find((cue) => time >= cue.start && time <= cue.end) ?? null;
  });

  function setCaptionLanguage(language: string): void {
    captionLanguage.value = language;
  }

  function clearNextTimer(): void {
    if (nextTimer) {
      clearTimeout(nextTimer);
      nextTimer = null;
    }
  }

  function attachAudio(el: HTMLAudioElement | null): void {
    audioElement.value = el;

    if (!el) {
      if (state.activeEngine.value === "audio") {
        state.activeMedia.value = null;
      }
      return;
    }

    if (bound) return;
    bound = true;

    el.volume = settings.value?.volume ?? 0.75;

    el.addEventListener("loadedmetadata", () => {
      if (pendingSeek !== null) {
        try {
          el.currentTime = pendingSeek;
        } catch {}
        state.currentTime.value = pendingSeek;
        pendingSeek = null;
      }
      if (autoplayOnLoad) {
        el.play().catch((err: unknown) =>
          console.warn("Audio autoplay interrupted:", err),
        );
      }
    });

    el.addEventListener("play", () => {
      state.status.value = "playing";
      const videoId = state.currentVideo.value?.id;
      if (videoId) {
        startPlaybackHistory(videoId, () => el.currentTime);
      }
    });

    el.addEventListener("pause", () => {
      // Swapping the source pauses the element too; that is not a user pause.
      if (state.status.value === "ended" || state.status.value === "loading") {
        return;
      }
      state.status.value = "paused";
      reportPlaybackTime();
    });

    el.addEventListener("timeupdate", () => {
      state.currentTime.value = el.currentTime;

      // Safety net for a stream that stops just short of its reported length:
      // the native `ended` event never fires, so autoplay would stall.
      const total = state.currentVideo.value?.duration || 0;
      if (
        total > 0 &&
        !el.paused &&
        !el.seeking &&
        state.status.value !== "ended" &&
        el.currentTime >= total - 0.75
      ) {
        handleEnded();
      }
    });

    el.addEventListener("waiting", () => {
      if (state.status.value !== "error" && state.status.value !== "ended") {
        state.status.value = "buffering";
      }
    });

    el.addEventListener("playing", () => {
      state.status.value = "playing";
    });

    el.addEventListener("volumechange", () => {
      state.isMuted.value = el.muted || el.volume === 0;
      updateSettings({ volume: el.volume });
    });

    el.addEventListener("ended", handleEnded);

    el.addEventListener("error", () => {
      state.status.value = "error";
      state.errorMessage.value = "Could not play the audio stream";
      console.error("Audio player error:", el.error);
    });
  }

  /** End of track: remember the position and queue the next one. */
  async function handleEnded(): Promise<void> {
    if (state.status.value === "ended") return;

    state.status.value = "ended";
    reportPlaybackTime();

    if (!state.isAutoplayEnabled.value) return;

    // Re-resolve the queue when the page did not manage to (or lost) it.
    const videoId = state.currentVideo.value?.id;
    const list = router.currentRoute.value.query.list;
    const playlistId = typeof list === "string" && list ? list : null;
    const next =
      state.upNextVideo.value ??
      (videoId ? await fetchUpNext(videoId, playlistId) : null);
    if (!next) {
      console.warn("[autoplay] no next track resolved for", videoId);
      return;
    }
    state.upNextVideo.value = next;

    clearNextTimer();
    nextTimer = setTimeout(() => {
      nextTimer = null;
      // Keep the Mix/playlist id so the queue continues past this track.
      const list = router.currentRoute.value.query.list;
      const query: Record<string, string> =
        typeof list === "string" && list ? { list } : {};
      void router.push({ path: `/listen/${next.id}`, query });
    }, 3000);
  }

  interface LoadOptions {
    resumeAt?: number;
    autoplay?: boolean;
  }

  /**
   * Replace whatever is playing with the audio track of `videoId`.
   *
   * The video engine is released before the request starts, so the decoded
   * frames and the HLS instance are gone while the audio URL is being resolved.
   */
  async function loadAudio(
    videoId: string,
    options: LoadOptions = {},
  ): Promise<void> {
    const token = ++loadToken;
    const isStale = (): boolean => token !== loadToken;
    const resumeAt = options.resumeAt ?? 0;

    clearNextTimer();
    state.activeEngine.value = "audio";
    state.isOpen.value = true;
    state.isMiniPlayer.value = false;
    state.errorMessage.value = null;
    state.captionTracks.value = [];
    state.upNextVideo.value = null;
    state.currentTime.value = resumeAt;
    state.currentSrc.value = "";
    state.currentVideo.value = {
      id: videoId,
      title: "Loading audio...",
      channelName: "",
      channelAvatarUrl: "",
      thumbnailUrl: "",
      duration: 0,
      viewCount: "",
      description: "",
    };

    // Releasing the video engine pauses it, which publishes a `pause` status:
    // switch to `loading` afterwards so that event cannot win.
    await player.releaseVideoEngine();
    if (isStale()) return;
    state.status.value = "loading";

    const res = await commands.getAudioStream(videoId);
    if (isStale()) return;

    if (res.status !== "ok") {
      state.status.value = "error";
      state.errorMessage.value =
        res.error.message || "This video has no audio-only stream";
      return;
    }

    const info = res.data;
    state.currentVideo.value = {
      id: videoId,
      title: info.title || "Audio",
      channelName: info.author || "",
      channelId: info.channelId || undefined,
      channelAvatarUrl: "",
      thumbnailUrl: info.thumbnailUrl || "",
      duration:
        Number(info.lengthSeconds) ||
        (info.durationMs ? info.durationMs / 1000 : 0),
      viewCount: info.viewCount,
      description: info.description,
    };

    const el = audioElement.value;
    if (!el) {
      state.status.value = "error";
      state.errorMessage.value = "Audio output is unavailable";
      return;
    }

    state.activeMedia.value = el;
    state.currentSrc.value = info.url;

    pendingSeek = resumeAt > 0 ? resumeAt : null;
    autoplayOnLoad = options.autoplay ?? true;

    el.src = info.url;
    el.load();
    void loadCaptions(info.captions ?? []);

    if (pendingSeek === null && autoplayOnLoad) {
      el.play().catch((err: unknown) =>
        console.warn("Audio autoplay interrupted:", err),
      );
    }
  }

  /** Stop the audio engine and forget the track. */
  function stopAudio(): void {
    if (state.activeEngine.value !== "audio") return;

    clearNextTimer();
    loadToken++;
    pendingSeek = null;
    clearCaptions();


    const el = audioElement.value;
    if (el) {
      try {
        el.pause();
      } catch {}
      el.removeAttribute("src");
      try {
        el.load();
      } catch {}
    }

    reportPlaybackTime();
    stopPlaybackHistory(false);

    state.activeEngine.value = null;
    state.isOpen.value = false;
    state.status.value = "idle";
    state.currentVideo.value = null;
    state.errorMessage.value = null;
    state.currentSrc.value = "";
    state.currentTime.value = 0;
  }

  return {
    attachAudio,
    loadAudio,
    stopAudio,
    audioElement,
    captionTracks: state.captionTracks,
    captionCueList: computed(
      () => captionCues.value[captionLanguage.value] ?? [],
    ),
    activeCue,
    captionLanguage,
    setCaptionLanguage,
  };
}
