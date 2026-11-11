import { ref, type ComputedRef } from "vue";
import { useRouter, type LocationQuery, type LocationQueryRaw } from "vue-router";
import {
  commands,
  type VideoItem_Deserialize,
  type WatchNextResults_Deserialize,
} from "@/bindings";
import { usePlayer } from "./usePlayer";
import { useSettings } from "./useSettings";
import { buildUpNext } from "./useUpNext";

/**
 * The watch and listen routes show the same video: same metadata, same queue,
 * same related list. They only differ in which engine plays it, so the data
 * loading lives here instead of being copied into both pages.
 */
export function useVideoPage(
  videoId: ComputedRef<string>,
  playlistId: ComputedRef<string | null>,
  playlistIndex: ComputedRef<number | null>,
) {
  const router = useRouter();
  const player = usePlayer();
  const { settings } = useSettings();

  const watchNext = ref<WatchNextResults_Deserialize | null>(null);
  const loading = ref(false);

  async function load(): Promise<void> {
    const id = videoId.value;
    if (!id) return;

    loading.value = true;
    try {
      const res = await commands.getWatchNext(
        id,
        playlistId.value,
        playlistIndex.value,
      );
      if (res.status !== "ok") return;

      watchNext.value = res.data;

      const next = buildUpNext(res.data, id, playlistId.value);
      if (next) player.setUpNextVideo(next);
    } catch (err) {
      console.warn("Failed to load watch-next details:", err);
    } finally {
      loading.value = false;
    }
  }

  /** The route the current playback mode plays this video on. */
  function playbackBasePath(): string {
    return settings.value.playbackMode === "normal" ? "/watch" : "/listen";
  }

  function selectRelated(item: VideoItem_Deserialize): void {
    const base = playbackBasePath();
    if (item.playlistId) {
      void router.push({
        path: `${base}/${item.id}`,
        query: { list: item.playlistId },
      });
    } else {
      void router.push(`${base}/${item.id}`);
    }
  }

  return { watchNext, loading, load, playbackBasePath, selectRelated };
}

/**
 * Read the hand-off position from `?t=`.
 *
 * Both routes carry it, so a mode switch can hand the other engine an exact
 * second to resume from.
 */
export function resumeFromQuery(query: LocationQuery): number {
  const raw = Number(query.t);
  return Number.isFinite(raw) && raw > 0 ? raw : 0;
}

/** URL query with the hand-off position set (or dropped when it is zero). */
export function withResumeAt(
  query: LocationQuery,
  seconds: number,
): LocationQueryRaw {
  const next: LocationQueryRaw = { ...query };
  if (seconds > 0) {
    next.t = String(Math.floor(seconds));
  } else {
    delete next.t;
  }
  return next;
}
