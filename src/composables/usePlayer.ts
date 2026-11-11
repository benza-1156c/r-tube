import { commands, type PlayerResponse_Deserialize } from "@/bindings";
import { useSettings } from "./useSettings";

export interface CaptionOption {
  id: string;
  name: string;
  language: string;
  url: string;
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

/** A media element that can host an HLS playback engine. */
type MediaElement = HTMLVideoElement;

const isOpen = ref<boolean>(false);
const isMiniPlayer = ref<boolean>(false);
const status = ref<PlayerStatus>("idle");
const errorMessage = ref<string | null>(null);

const currentVideo = ref<CurrentVideoMeta | null>(null);
/** Proxied HLS master playlist — the only source the player ever loads. */
const currentSrc = ref<string>("");

const captionTracks = ref<CaptionOption[]>([]);
const currentTime = ref<number>(0);

const videoElement = shallowRef<MediaElement | null>(null);

/** Watch page slot the player is teleported into. */
const watchAnchor = shallowRef<HTMLElement | null>(null);

const activeBlobUrls: string[] = [];
const boundVideos = new WeakSet<HTMLVideoElement>();

let isRecovering = false;

/**
 * Incremented by every load and by `closePlayer`. A load only publishes its
 * result while it still owns the latest token, so a slow response can never
 * overwrite a newer video or revive the source after the player was closed.
 */
let loadToken = 0;

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

export function usePlayer() {
  const { settings, updateSettings } = useSettings();
  const isPlaying = computed<boolean>(() => status.value === "playing");

  function attachVideo(el: MediaElement | null): void {
    videoElement.value = el;
    if (el && !boundVideos.has(el)) {
      boundVideos.add(el);

      if (settings.value) {
        el.volume = settings.value.volume || 0.75;
      }

      el.addEventListener("loadeddata", () => {
        if (status.value === "loading") {
          status.value = "paused";
        }
      });

      el.addEventListener("volumechange", () => {
        updateSettings({ volume: el.volume });
      });
    }
  }

  /** Tell the player which slot hosts it (null = mini player). */
  function setWatchAnchor(el: HTMLElement | null): void {
    watchAnchor.value = el;
    isMiniPlayer.value = !el;
  }

  interface LoadOptions {
    resumeAt?: number;
    autoplay?: boolean;
    preserveMeta?: boolean;
  }

  async function loadStream(
    videoId: string,
    options: LoadOptions = {},
  ): Promise<void> {
    const token = ++loadToken;
    const isStale = (): boolean => token !== loadToken;

    status.value = "loading";
    errorMessage.value = null;
    if (!options.resumeAt) {
      currentTime.value = 0;
    }

    if (!options.preserveMeta) {
      currentVideo.value = {
        id: videoId,
        title: "Loading video...",
        channelName: "",
        channelAvatarUrl: "",
        thumbnailUrl: "",
        duration: 0,
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

      currentVideo.value = {
        id: info.id || videoId,
        title: info.title || currentVideo.value?.title || "",
        channelName: info.channelName || currentVideo.value?.channelName || "",
        channelAvatarUrl: currentVideo.value?.channelAvatarUrl || "",
        thumbnailUrl:
          info.thumbnailUrl || currentVideo.value?.thumbnailUrl || "",
        duration:
          Number(info.lengthSeconds) || currentVideo.value?.duration || 0,
        viewCount: info.viewCount,
        description: info.description,
      };

      // Captions come through Rust so YouTube's CORS rules never apply.
      revokeBlobUrls();
      if (info.captions && info.captions.length > 0) {
        const loaded: CaptionOption[] = [];
        // Kept local until the whole load succeeds: a cancelled load has to
        // release its own object URLs instead of parking them in the shared
        // list for a player that is no longer showing anything.
        const createdUrls: string[] = [];
        for (const caption of info.captions) {
          try {
            const subRes = await commands.getSubtitleContent(caption.vttUrl);
            if (isStale()) {
              for (const url of createdUrls) URL.revokeObjectURL(url);
              return;
            }
            if (subRes.status === "ok" && subRes.data) {
              const blobUrl = URL.createObjectURL(
                new Blob([subRes.data], { type: "text/vtt" }),
              );
              createdUrls.push(blobUrl);
              loaded.push({
                id: caption.languageCode,
                name: caption.name,
                language: caption.languageCode,
                url: blobUrl,
              });
            }
          } catch (e) {
            console.warn(
              "Could not load caption track:",
              caption.languageCode,
              e,
            );
          }
        }
        if (isStale()) {
          for (const url of createdUrls) URL.revokeObjectURL(url);
          return;
        }
        activeBlobUrls.push(...createdUrls);
        captionTracks.value = loaded;
      } else {
        captionTracks.value = [];
      }

      if (!info.hlsManifestUrl) {
        throw new Error("This video has no HLS stream available");
      }

      const video = videoElement.value;
      const resumeAt = options.resumeAt ?? 0;
      if (video && resumeAt > 0) {
        video.addEventListener(
          "loadedmetadata",
          () => {
            // The element may already be loading a newer video by now.
            if (isStale()) return;
            video.currentTime = resumeAt;
            if (options.autoplay ?? true) {
              video.play().catch(() => {});
            }
          },
          { once: true },
        );
      }

      currentSrc.value = info.hlsManifestUrl;
    } catch (err: unknown) {
      // A cancelled load is not a failure: it no longer owns this player.
      if (isStale()) {
        return;
      }
      console.error("Error loading video:", err);
      status.value = "error";
      errorMessage.value =
        err instanceof Error ? err.message : "Failed to play video";
    }
  }

  async function loadVideo(
    videoId: string,
    initialMeta?: Partial<CurrentVideoMeta>,
  ): Promise<void> {
    isOpen.value = true;
    isRecovering = false;

    if (initialMeta) {
      currentVideo.value = {
        id: videoId,
        title: initialMeta.title || "Loading video...",
        channelName: initialMeta.channelName || "",
        channelAvatarUrl: initialMeta.channelAvatarUrl || "",
        thumbnailUrl: initialMeta.thumbnailUrl || "",
        duration: initialMeta.duration || 0,
        viewCount: initialMeta.viewCount || "",
        description: initialMeta.description || "",
      };
    }

    await loadStream(videoId, { preserveMeta: Boolean(initialMeta) });
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
    const video = currentVideo.value;
    const element = videoElement.value;
    if (!video || !element || isRecovering) {
      return false;
    }
    if (!opts.manual && status.value !== "error") {
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
      return status.value !== "error";
    } finally {
      isRecovering = false;
    }
  }

  function onPlay(): void {
    status.value = "playing";
  }

  function onPause(): void {
    if (status.value !== "ended") {
      status.value = "paused";
    }
  }

  function onTimeUpdate(e: Event): void {
    const el = e.target as HTMLVideoElement | null;
    if (el) {
      currentTime.value = el.currentTime;
    }
  }

  function onWaiting(): void {
    if (status.value !== "error") {
      status.value = "buffering";
    }
  }

  function onPlaying(): void {
    status.value = "playing";
  }

  function onEnded(): void {
    status.value = "ended";
  }

  function onError(e: Event | unknown): void {
    if (!currentSrc.value) {
      return;
    }
    const target = (e as Event)?.target as HTMLElement | null;
    if (target?.tagName?.toLowerCase() === "track") {
      return;
    }
    const mediaErr = videoElement.value?.error;
    if (mediaErr && mediaErr.code === 1) {
      return;
    }

    console.error("Video player error:", e, mediaErr);
    status.value = "error";
    errorMessage.value = "Failed to stream video. Please try again.";

    // A network/source failure usually means the playlist expired.
    if (!mediaErr || mediaErr.code === 2 || mediaErr.code === 4) {
      void recoverStream({ manual: true });
    }
  }

  function togglePlay(): void {
    const el = videoElement.value;
    if (!el) return;
    if (el.paused) {
      el.play().catch((err: unknown) => console.warn("Play interrupted:", err));
    } else {
      el.pause();
    }
  }

  /**
   * Close the player and release everything the stream holds.
   *
   * The Video.js elements themselves stay mounted (they are reused by the next
   * video, and a destroyed element cannot be reconnected), so this only tears
   * down the stream: the pending load is cancelled, caption object URLs are
   * revoked, and the empty source makes the HLS adapter drop its media source
   * and buffers.
   */
  function closePlayer(): void {
    loadToken++;
    isOpen.value = false;
    status.value = "idle";
    currentSrc.value = "";
    currentVideo.value = null;
    isRecovering = false;
    revokeBlobUrls();
    captionTracks.value = [];
    if (videoElement.value) {
      videoElement.value.pause();
      videoElement.value.removeAttribute("src");
      videoElement.value.load();
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
    captionTracks,
    currentTime,
    isPlaying,
    videoElement,
    watchAnchor,
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
    closePlayer,
    minimizeToMiniPlayer,
    restoreFromMiniPlayer,
  };
}
