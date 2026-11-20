import { onBeforeUnmount, onMounted, watch, type Ref } from "vue";

/**
 * `moveBefore` is only declared in newer DOM libs (TypeScript 6 puts it on
 * `ParentNode`), and editors on an older TypeScript do not know it yet. Typing
 * the one call locally keeps the file compiling everywhere; the runtime check
 * below covers browsers without the API.
 */
type MoveBeforeParent = {
  moveBefore?: (node: Node, child: Node | null) => void;
};

/**
 * Move `node` into `parent` without disconnecting it when the browser can.
 *
 * `moveBefore` keeps element state (video.js elements are not written for
 * being reconnected — see `playerElementMove.ts`). Older browsers still get
 * the player, just through a reconnect.
 */
function moveNode(node: HTMLElement, parent: Element): void {
  if (node.parentElement === parent) return;
  const mover = parent as unknown as MoveBeforeParent;
  try {
    if (typeof mover.moveBefore === "function") {
      mover.moveBefore(node, null);
    }
  } catch {
    // Not every node pair is movable (detached targets, for example).
  }
  if (node.parentElement !== parent) {
    parent.appendChild(node);
  }
}

/**
 * Keep the single player element inside whichever host currently owns it.
 *
 * The player is created once and then moved — into the watch page's slot while
 * that page owns it, back into its own stage for the mini player. It is moved
 * rather than re-created because playback, volume and the HLS engine live in
 * it, and it is moved with `moveBefore()` rather than through Vue's Teleport
 * because Teleport disconnects the subtree, which video.js turns into a pile
 * of duplicated controllers.
 *
 * `host` is the element that should hold the player right now, or `null` for
 * "keep it in the stage".
 */
export function usePlayerStage(
  rootRef: Ref<HTMLElement | null>,
  stageRef: Ref<HTMLElement | null>,
  host: Ref<HTMLElement | null>,
) {
  function sync(): void {
    const root = rootRef.value;
    const stage = stageRef.value;
    if (!root || !stage) return;
    const target = host.value;
    moveNode(root, target?.isConnected ? target : stage);
  }

  // Synchronous so the player leaves the page slot in the same tick the slot
  // is released, before the route swap removes it and disconnects the player.
  watch(host, sync, { flush: "sync" });

  onMounted(sync);

  onBeforeUnmount(() => {
    const root = rootRef.value;
    const stage = stageRef.value;
    if (root && stage) moveNode(root, stage);
  });

  return { sync };
}
