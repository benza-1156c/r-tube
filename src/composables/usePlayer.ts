import {
  commands,
  type PlayerResponse_Deserialize,
  type VideoStreamFormat_Deserialize,
} from "@/bindings";

export interface CaptionOption {
  id: string;
  name: string;
  language: string;
  url: string;
}

export interface QualityOption {
  label: string;
  itag: number;
  url: string;
  height: number;
  isMuxed: boolean;
}

export interface CurrentVideoMeta {
  id: string;
  title: string;
  channelName: string;
  channelAvatarUrl?: string;
  thumbnailUrl?: string;
  duration: number;
  viewCount?: string;
  description?: string;
}

export type PlayerStatus =
  | "idle"
  | "loading"
  | "playing"
  | "paused"
  | "buffering"
  | "ended"
  | "error";

export type PlayableStreamFormat = VideoStreamFormat_Deserialize & {
  url: string;
};

function hasPlayableUrl(
  format: VideoStreamFormat_Deserialize,
): format is PlayableStreamFormat {
  return typeof format.url === "string" && format.url.trim().length > 0;
}

// Global reactive player state
const isOpen = ref<boolean>(false);
const isMiniPlayer = ref<boolean>(false);
const status = ref<PlayerStatus>("idle");
const errorMessage = ref<string | null>(null);

const currentVideo = ref<CurrentVideoMeta | null>(null);
const currentSrc = ref<string>("");
const currentAudioSrc = ref<string>(""); // Audio stream URL for adaptive 1080p+ formats
const bestAudioUrl = ref<string>(""); // Best available audio stream URL
const availableQualities = ref<QualityOption[]>([]);
const currentQuality = ref<string>("Auto");

const captionTracks = ref<CaptionOption[]>([]);
const selectedCaptionId = ref<string | null>(null);

const currentTime = ref<number>(0);
const duration = ref<number>(0);
const volume = ref<number>(1);
const isMuted = ref<boolean>(false);
const playbackRate = ref<number>(1);

const videoElement = shallowRef<HTMLVideoElement | null>(null);
const audioElement = shallowRef<HTMLAudioElement | null>(null);

// Watch Page Anchor Positioning
const watchAnchor = shallowRef<HTMLElement | null>(null);
const anchorRect = ref<{
  top: number;
  left: number;
  width: number;
  height: number;
} | null>(null);

// Active blob URLs for subtitle VTTs
const activeBlobUrls: string[] = [];

function revokeBlobUrls(): void {
  while (activeBlobUrls.length > 0) {
    const url = activeBlobUrls.pop();
    if (url) {
      try {
        URL.revokeObjectURL(url);
      } catch {
        // Ignore revocation error
      }
    }
  }
}

export function usePlayer() {
  const isPlaying = computed<boolean>(() => status.value === "playing");
  const isBuffering = computed<boolean>(
    () => status.value === "buffering" || status.value === "loading",
  );

  /**
   * Bind HTMLVideoElement ref
   */
  function attachVideo(el: HTMLVideoElement | null): void {
    videoElement.value = el;
    if (el) {
      volume.value = el.volume;
      isMuted.value = el.muted;
      playbackRate.value = el.playbackRate;
    }
  }

  /**
   * Bind HTMLAudioElement ref for adaptive audio playback
   */
  function attachAudio(el: HTMLAudioElement | null): void {
    audioElement.value = el;
    if (el) {
      el.volume = volume.value;
      el.muted = isMuted.value;
      el.playbackRate = playbackRate.value;
    }
  }

  /**
   * Update the bounding rectangle of the watch anchor placeholder.
   */
  function updateAnchorRect(): void {
    if (watchAnchor.value) {
      const rect = watchAnchor.value.getBoundingClientRect();
      anchorRect.value = {
        top: Math.round(rect.top),
        left: Math.round(rect.left),
        width: Math.round(rect.width),
        height: Math.round(rect.height),
      };
    } else {
      anchorRect.value = null;
    }
  }

  /**
   * Set or clear the watch anchor placeholder element (from [id].vue).
   */
  function setWatchAnchor(el: HTMLElement | null): void {
    watchAnchor.value = el;
    if (el) {
      isMiniPlayer.value = false;
      updateAnchorRect();
    } else {
      isMiniPlayer.value = true;
      anchorRect.value = null;
    }
  }

  /**
   * Load and play a YouTube video.
   */
  async function loadVideo(
    videoId: string,
    initialMeta?: Partial<CurrentVideoMeta>,
  ): Promise<void> {
    isOpen.value = true;
    status.value = "loading";
    errorMessage.value = null;
    currentTime.value = 0;
    availableQualities.value = [];
    currentQuality.value = "Auto";
    currentAudioSrc.value = "";
    bestAudioUrl.value = "";

    currentVideo.value = {
      id: videoId,
      title: initialMeta?.title || "Loading video...",
      channelName: initialMeta?.channelName || "",
      channelAvatarUrl: initialMeta?.channelAvatarUrl || "",
      thumbnailUrl: initialMeta?.thumbnailUrl || "",
      duration: initialMeta?.duration || 0,
      viewCount: initialMeta?.viewCount || "",
      description: initialMeta?.description || "",
    };

    try {
      const infoRes = await commands.getVideoInfo(videoId);
      if (infoRes.status !== "ok") {
        throw new Error(
          infoRes.error.message || "Failed to fetch video stream info",
        );
      }

      const info: PlayerResponse_Deserialize = infoRes.data;

      currentVideo.value = {
        id: info.id,
        title: info.title || currentVideo.value.title,
        channelName: info.channelName || currentVideo.value.channelName,
        channelAvatarUrl: currentVideo.value.channelAvatarUrl,
        thumbnailUrl: info.thumbnailUrl || currentVideo.value.thumbnailUrl,
        duration: Number(info.lengthSeconds) || currentVideo.value.duration,
        viewCount: info.viewCount,
        description: info.description,
      };

      // Populate Captions via Rust backend to bypass YouTube CORS restrictions
      revokeBlobUrls();
      if (info.captions && info.captions.length > 0) {
        const loadedTracks: CaptionOption[] = [];
        for (const c of info.captions) {
          try {
            const subRes = await commands.getSubtitleContent(c.vttUrl);
            if (subRes.status === "ok" && subRes.data) {
              const blob = new Blob([subRes.data], { type: "text/vtt" });
              const blobUrl = URL.createObjectURL(blob);
              activeBlobUrls.push(blobUrl);
              loadedTracks.push({
                id: c.languageCode,
                name: c.name,
                language: c.languageCode,
                url: blobUrl,
              });
            }
          } catch (e) {
            console.warn("Could not load caption track:", c.languageCode, e);
          }
        }
        captionTracks.value = loadedTracks;
      } else {
        captionTracks.value = [];
      }

      // 1. Extract best available audio track for adaptive streams (prioritizing m4a/mp4 before webm)
      const audioFormats = (info.adaptiveFormats || [])
        .filter(hasPlayableUrl)
        .filter((f) => f.isAudio && !f.isVideo)
        .sort((a, b) => {
          const aMp4 = a.mimeType.includes("mp4") ? 1 : 0;
          const bMp4 = b.mimeType.includes("mp4") ? 1 : 0;
          return bMp4 - aMp4 || b.bitrate - a.bitrate;
        });

      bestAudioUrl.value = audioFormats[0]?.url || "";

      // 2. Collect available video formats with type safety
      const qualities: QualityOption[] = [];
      const seenLabels = new Set<string>();

      // Progressive muxed formats (contain audio + video in a single MP4, like itag 18 360p or itag 22 720p)
      const muxedFormats = (info.formats || [])
        .filter(hasPlayableUrl)
        .filter((f) => f.isVideo && f.isAudio)
        .sort((a, b) => (b.height ?? 0) - (a.height ?? 0));

      for (const f of muxedFormats) {
        const label =
          f.qualityLabel || (f.height ? `${f.height}p` : `${f.itag}p`);
        if (!seenLabels.has(label)) {
          seenLabels.add(label);
          qualities.push({
            label,
            itag: f.itag,
            url: f.url,
            height: f.height ?? 360,
            isMuxed: true,
          });
        }
      }

      // Other adaptive video formats (1080p, 720p, 480p, etc.)
      // Prioritize MP4 / avc1 (H.264) codec over webm/vp9/av01 for maximum compatibility
      const otherFormats = (info.formats || [])
        .concat(info.adaptiveFormats || [])
        .filter(hasPlayableUrl)
        .filter((f) => f.isVideo)
        .sort((a, b) => {
          const aAvc = a.mimeType.includes("avc1") ? 1 : 0;
          const bAvc = b.mimeType.includes("avc1") ? 1 : 0;
          return (
            bAvc - aAvc ||
            (b.height ?? 0) - (a.height ?? 0) ||
            b.bitrate - a.bitrate
          );
        });

      for (const f of otherFormats) {
        const label =
          f.qualityLabel || (f.height ? `${f.height}p` : `${f.itag}p`);
        if (!seenLabels.has(label)) {
          seenLabels.add(label);
          qualities.push({
            label,
            itag: f.itag,
            url: f.url,
            height: f.height ?? 360,
            isMuxed: Boolean(f.isAudio),
          });
        }
      }

      // Sort qualities by height descending (e.g. 1080p, 720p, 480p, 360p, 240p, 144p)
      qualities.sort((a, b) => b.height - a.height);
      availableQualities.value = qualities;

      // Choose default stream (prefer 720p or 360p muxed MP4 for instant smooth playback)
      const preferred =
        qualities.find((q) => q.isMuxed && q.label.includes("720")) ||
        qualities.find((q) => q.isMuxed && q.label.includes("360")) ||
        qualities.find((q) => q.isMuxed) ||
        qualities[0];

      if (!preferred?.url) {
        throw new Error("No playable stream found for this video");
      }

      currentQuality.value = preferred.label;
      currentSrc.value = preferred.url;
      // If preferred format is not muxed (adaptive), activate separate audio stream
      currentAudioSrc.value = preferred.isMuxed ? "" : bestAudioUrl.value;
    } catch (err: unknown) {
      console.error("Error loading video:", err);
      status.value = "error";
      errorMessage.value =
        err instanceof Error ? err.message : "Failed to play video";
    }
  }

  /**
   * Switch video resolution/quality seamlessly while preserving playback time and syncing separate audio if needed.
   * (ท่าที่ 2: ท่าสลับลิงก์ MP4 สดๆ)
   */
  function setQuality(q: QualityOption): void {
    const v = videoElement.value;
    if (!v || currentSrc.value === q.url) return;

    const savedTime = v.currentTime;
    const wasPlaying = !v.paused;

    currentQuality.value = q.label;
    currentSrc.value = q.url;
    // Activate separate audio stream for adaptive video (1080p+), clear for muxed
    currentAudioSrc.value = q.isMuxed ? "" : bestAudioUrl.value;

    const onLoaded = () => {
      if (videoElement.value) {
        videoElement.value.currentTime = savedTime;
        if (audioElement.value && currentAudioSrc.value) {
          audioElement.value.currentTime = savedTime;
        }
        if (wasPlaying) {
          videoElement.value.play().catch(() => {});
          if (audioElement.value && currentAudioSrc.value) {
            audioElement.value.play().catch(() => {});
          }
        }
      }
    };
    v.addEventListener("loadedmetadata", onLoaded, { once: true });
  }

  function onPlay(): void {
    status.value = "playing";
    if (
      audioElement.value &&
      currentAudioSrc.value &&
      audioElement.value.paused
    ) {
      if (videoElement.value) {
        audioElement.value.currentTime = videoElement.value.currentTime;
      }
      audioElement.value.play().catch(() => {});
    }
  }

  function onPause(): void {
    if (status.value !== "ended") {
      status.value = "paused";
    }
    if (
      audioElement.value &&
      currentAudioSrc.value &&
      !audioElement.value.paused
    ) {
      audioElement.value.pause();
    }
  }

  function onTimeUpdate(e: Event): void {
    const el = e.target as HTMLVideoElement | null;
    if (el) {
      currentTime.value = el.currentTime;
      // Synchronize audio stream if drift exceeds 0.15s
      if (audioElement.value && currentAudioSrc.value) {
        const drift = Math.abs(audioElement.value.currentTime - el.currentTime);
        if (drift > 0.15) {
          audioElement.value.currentTime = el.currentTime;
        }
      }
    }
  }

  function onDurationChange(e: Event): void {
    const el = e.target as HTMLVideoElement | null;
    if (el && Number.isFinite(el.duration)) {
      duration.value = el.duration;
    }
  }

  function onVolumeChange(e: Event): void {
    const el = e.target as HTMLVideoElement | null;
    if (el) {
      volume.value = el.volume;
      isMuted.value = el.muted;
      if (audioElement.value) {
        audioElement.value.volume = el.volume;
        audioElement.value.muted = el.muted;
      }
    }
  }

  function onWaiting(): void {
    if (status.value !== "error") {
      status.value = "buffering";
    }
    // Pause audio while video is buffering to maintain sync
    if (
      audioElement.value &&
      currentAudioSrc.value &&
      !audioElement.value.paused
    ) {
      audioElement.value.pause();
    }
  }

  function onPlaying(): void {
    status.value = "playing";
    // Resume audio once buffering is complete
    if (
      audioElement.value &&
      currentAudioSrc.value &&
      audioElement.value.paused
    ) {
      if (videoElement.value) {
        audioElement.value.currentTime = videoElement.value.currentTime;
      }
      audioElement.value.play().catch(() => {});
    }
  }

  function onEnded(): void {
    status.value = "ended";
    if (audioElement.value && currentAudioSrc.value) {
      audioElement.value.pause();
    }
  }

  function onError(e: Event | unknown): void {
    if (!currentSrc.value) {
      return;
    }

    const target = (e as Event)?.target as HTMLElement | null;
    // Discard any errors from track elements to never break video playback
    if (target && target.tagName && target.tagName.toLowerCase() === "track") {
      console.warn("Subtitle track error ignored:", target);
      return;
    }

    const el = videoElement.value;
    const mediaErr = el?.error;

    // Ignore MEDIA_ERR_ABORTED (code 1) when unmounting or changing src
    if (mediaErr && mediaErr.code === 1) {
      return;
    }

    console.error("Video player error:", e, mediaErr);
    status.value = "error";
    errorMessage.value = "Failed to stream video. Please try again.";
  }

  function togglePlay(): void {
    const el = videoElement.value;
    if (!el) return;
    if (el.paused) {
      el.play().catch((err: unknown) => {
        console.warn("Play interrupted:", err);
      });
      if (audioElement.value && currentAudioSrc.value) {
        audioElement.value.currentTime = el.currentTime;
        audioElement.value.play().catch(() => {});
      }
    } else {
      el.pause();
      if (audioElement.value && currentAudioSrc.value) {
        audioElement.value.pause();
      }
    }
  }

  function seek(seconds: number): void {
    const el = videoElement.value;
    if (!el) return;
    const target = Math.max(0, Math.min(seconds, duration.value));
    el.currentTime = target;
    if (audioElement.value && currentAudioSrc.value) {
      audioElement.value.currentTime = target;
    }
  }

  function seekRelative(offsetSeconds: number): void {
    const el = videoElement.value;
    if (!el) return;
    seek(el.currentTime + offsetSeconds);
  }

  function setVolume(val: number): void {
    const clamped = Math.max(0, Math.min(val, 1));
    volume.value = clamped;
    if (videoElement.value) {
      videoElement.value.volume = clamped;
      videoElement.value.muted = clamped === 0;
    }
    if (audioElement.value) {
      audioElement.value.volume = clamped;
      audioElement.value.muted = clamped === 0;
    }
  }

  function toggleMute(): void {
    isMuted.value = !isMuted.value;
    if (videoElement.value) {
      videoElement.value.muted = isMuted.value;
    }
    if (audioElement.value) {
      audioElement.value.muted = isMuted.value;
    }
  }

  function setPlaybackRate(rate: number): void {
    playbackRate.value = rate;
    if (videoElement.value) {
      videoElement.value.playbackRate = rate;
    }
    if (audioElement.value) {
      audioElement.value.playbackRate = rate;
    }
  }

  function closePlayer(): void {
    isOpen.value = false;
    status.value = "idle";
    currentSrc.value = "";
    currentAudioSrc.value = "";
    bestAudioUrl.value = "";
    currentVideo.value = null;
    availableQualities.value = [];
    revokeBlobUrls();
    captionTracks.value = [];
    if (videoElement.value) {
      videoElement.value.pause();
      videoElement.value.removeAttribute("src");
      videoElement.value.load();
    }
    if (audioElement.value) {
      audioElement.value.pause();
      audioElement.value.removeAttribute("src");
      audioElement.value.load();
    }
  }

  function minimizeToMiniPlayer(): void {
    isMiniPlayer.value = true;
  }

  function restoreFromMiniPlayer(): void {
    isMiniPlayer.value = false;
  }

  return {
    isOpen,
    isMiniPlayer,
    status,
    errorMessage,
    currentVideo,
    currentSrc,
    currentAudioSrc,
    bestAudioUrl,
    availableQualities,
    currentQuality,
    captionTracks,
    selectedCaptionId,
    currentTime,
    duration,
    volume,
    isMuted,
    playbackRate,
    isPlaying,
    isBuffering,
    videoElement,
    audioElement,
    watchAnchor,
    anchorRect,
    attachVideo,
    attachAudio,
    setWatchAnchor,
    updateAnchorRect,
    loadVideo,
    setQuality,
    onPlay,
    onPause,
    onTimeUpdate,
    onDurationChange,
    onVolumeChange,
    onWaiting,
    onPlaying,
    onEnded,
    onError,
    togglePlay,
    seek,
    seekRelative,
    setVolume,
    toggleMute,
    setPlaybackRate,
    closePlayer,
    minimizeToMiniPlayer,
    restoreFromMiniPlayer,
  };
}
