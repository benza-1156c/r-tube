import { ref, shallowRef } from "vue";
import type { VideoItem_Deserialize } from "@/bindings";

/**
 * Playback state shared by both engines.
 *
 * The video engine (`usePlayer`) and the audio engine (`useAudioPlayer`) never
 * own state of their own: they publish into this module so a single set of
 * refs drives the watch page, the listen page and the mini players. That is
 * what makes a mode hand-off keep the same title, position and play state.
 */

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
  channelId?: string;
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

/** Which engine currently owns playback (`null` = nothing loaded). */
export type PlaybackEngine = "video" | "audio" | null;

export const isOpen = ref(false);
export const isMiniPlayer = ref(false);
export const status = ref<PlayerStatus>("idle");
export const errorMessage = ref<string | null>(null);

export const currentVideo = ref<CurrentVideoMeta | null>(null);
/** Proxied playback source of the active engine (HLS manifest or audio URL). */
export const currentSrc = ref("");

export const captionTracks = ref<CaptionOption[]>([]);
export const currentTime = ref(0);

export const upNextVideo = ref<VideoItem_Deserialize | null>(null);
export const autoplayCountdown = ref<number | null>(null);
export const isAutoplayEnabled = ref(true);

export const activeEngine = ref<PlaybackEngine>(null);
/** The media element the controls act on: `<video>` or `<audio>`. */
export const activeMedia = shallowRef<HTMLMediaElement | null>(null);
export const isMuted = ref(false);

/** Set while the Video.js tree is torn down so its engine memory is released. */
export const videoReleased = ref(false);
