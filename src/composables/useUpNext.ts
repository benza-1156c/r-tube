import {
  commands,
  type VideoItem_Deserialize,
  type WatchNextResults_Deserialize,
} from "@/bindings";

/**
 * Turning a watch-next response into "the video that should play after this
 * one". Shared by the page loader and by the two engines, so an autoplay that
 * somehow lost its queue can resolve it again at the moment the track ends.
 */
export function buildUpNext(
  data: WatchNextResults_Deserialize,
  videoId: string,
  playlistId: string | null,
): VideoItem_Deserialize | null {
  const items = data.playlistItems ?? [];
  const index = items.findIndex(
    (item) => item.videoId === videoId || item.isSelected,
  );

  // A Mix keeps its order; a video that is missing from the panel (mixes
  // reorder themselves) falls back to the first related video.
  if (index >= 0 && index + 1 < items.length) {
    const next = items[index + 1];
    return {
      id: next.videoId,
      title: next.title,
      thumbnailUrl: next.thumbnailUrl,
      duration: "",
      channelName: next.author,
      channelAvatarUrl: "",
      isVerified: false,
      isLive: false,
      viewCount: "",
      publishedAt: "",
      playlistId: playlistId ?? undefined,
    };
  }

  return data.relatedVideos?.[0] ?? null;
}

/** Ask YouTube what follows `videoId`, current in `playlistId` when known. */
export async function fetchUpNext(
  videoId: string,
  playlistId: string | null,
): Promise<VideoItem_Deserialize | null> {
  try {
    const res = await commands.getWatchNext(videoId, playlistId, null);
    if (res.status !== "ok") return null;
    return buildUpNext(res.data, videoId, playlistId);
  } catch (err) {
    console.warn("Could not resolve the next video:", err);
    return null;
  }
}
