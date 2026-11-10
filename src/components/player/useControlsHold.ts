import { onBeforeUnmount, onMounted, watch, type Ref } from "vue";
import type { usePlayer } from "@/composables/usePlayer";

/**
 * Share of the player's height, measured from the bottom, that is treated as
 * "reaching for the controls". Clamped so it stays a sensible target on both
 * the mini player and a full-size watch-page player.
 */
const MIN_BAND = 88;
const MAX_BAND = 140;
const BAND_RATIO = 0.25;

/** The store is only reachable through the player element. */
interface PlayerElement extends HTMLElement {
  store?: { requestControlsLock?: () => () => void };
}

/**
 * Keep the player's control bar on screen while the pointer is near it.
 *
 * The packaged skin marks the user inactive two seconds after the last pointer
 * movement, then hides the bar and closes the popups owned by it — including
 * the hover volume slider. Reaching for a control produces no movement once
 * the pointer rests on it, so the bar could disappear from under the pointer
 * before it could be used. Holding the store's controls lock stops that idle
 * timer for as long as the pointer stays in the control band.
 */
export function useControlsHold(
  rootRef: Ref<HTMLElement | null>,
  player: ReturnType<typeof usePlayer>,
) {
  let release: (() => void) | null = null;
  let playerEl: PlayerElement | null = null;

  function getStore(): PlayerElement["store"] {
    if (!playerEl || !playerEl.isConnected) {
      playerEl =
        (rootRef.value?.querySelector(
          "video-player",
        ) as PlayerElement | null) ?? null;
    }
    return playerEl?.store;
  }

  function acquire(): void {
    if (release || !player.isOpen.value) return;
    try {
      const releaseLock = getStore()?.requestControlsLock?.();
      if (typeof releaseLock === "function") {
        release = releaseLock;
      }
    } catch {
      // The player is not attached to its media yet; the next move retries.
    }
  }

  function releaseLock(): void {
    if (!release) return;
    // Releasing can throw once the player has been destroyed.
    const releaseNow = release;
    release = null;
    try {
      releaseNow();
    } catch {
      // The player went away, taking its lock with it.
    }
  }

  function isInControlBand(event: PointerEvent): boolean {
    const rect = rootRef.value?.getBoundingClientRect();
    if (!rect || rect.height === 0) return false;
    const band = Math.min(
      MAX_BAND,
      Math.max(MIN_BAND, rect.height * BAND_RATIO),
    );
    return event.clientY >= rect.bottom - band;
  }

  function onPointerMove(event: PointerEvent): void {
    if (event.pointerType !== "mouse") return;
    if (isInControlBand(event)) {
      acquire();
      return;
    }
    releaseLock();
  }

  function onPointerLeave(): void {
    releaseLock();
  }

  onMounted(() => {
    const el = rootRef.value;
    if (!el) return;
    el.addEventListener("pointermove", onPointerMove, { passive: true });
    el.addEventListener("pointerleave", onPointerLeave);
    el.addEventListener("pointercancel", onPointerLeave);
  });

  onBeforeUnmount(() => {
    const el = rootRef.value;
    el?.removeEventListener("pointermove", onPointerMove);
    el?.removeEventListener("pointerleave", onPointerLeave);
    el?.removeEventListener("pointercancel", onPointerLeave);
    releaseLock();
  });

  // A closed player must not keep a lock on its own controls.
  watch(
    () => player.isOpen.value,
    (open) => {
      if (!open) releaseLock();
    },
  );
}
