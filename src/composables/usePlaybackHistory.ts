import { commands } from "@/bindings";

/**
 * Watch-history reporting, shared by the video and audio engines.
 *
 * It used to live inside `usePlayer`, which meant audio-only playback never
 * reported progress. Keeping one timer here also stops a mode hand-off from
 * producing two report loops for the same video.
 */

const REPORT_INTERVAL_MS = 30_000;

let timer: ReturnType<typeof setInterval> | null = null;
let historyVideoId: string | null = null;
let timeSource: (() => number) | null = null;

function report(): void {
  const videoId = historyVideoId;
  const read = timeSource;
  if (!videoId || !read) return;

  const seconds = read();
  if (!Number.isFinite(seconds) || seconds <= 0) return;

  void commands
    .updateWatchTime(videoId, Math.round(seconds * 100) / 100)
    .then((res) => {
      if (res.status !== "ok") {
        console.warn("Could not update watch time:", res.error.message);
      }
    })
    .catch((err) => {
      console.warn("Could not update watch time:", err);
    });
}

/** Report where playback currently is (before a pause, close or hand-off). */
export function reportPlaybackTime(): void {
  report();
}

/**
 * Start reporting for `videoId`.
 *
 * Re-entering with the same id (the video → audio hand-off) only swaps the time
 * source: the video is already in the history and must not be recorded twice.
 */
export function startPlaybackHistory(
  videoId: string,
  readTime: () => number,
): void {
  if (!videoId) return;

  if (historyVideoId === videoId) {
    timeSource = readTime;
    if (!timer) timer = setInterval(report, REPORT_INTERVAL_MS);
    return;
  }

  stopPlaybackHistory(true);

  historyVideoId = videoId;
  timeSource = readTime;

  void commands
    .recordWatchStart(videoId)
    .then((res) => {
      if (res.status !== "ok") {
        console.warn("Could not record watch history:", res.error.message);
      }
    })
    .catch((err) => {
      console.warn("Could not record watch history:", err);
    });

  timer = setInterval(report, REPORT_INTERVAL_MS);
}

/** Stop reporting, optionally flushing the last known position first. */
export function stopPlaybackHistory(reportLast = true): void {
  if (reportLast) report();

  if (timer) {
    clearInterval(timer);
    timer = null;
  }
  historyVideoId = null;
  timeSource = null;
}
