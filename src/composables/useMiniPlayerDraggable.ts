import { ref, computed } from "vue";

const position = ref<{ x: number; y: number } | null>(null);
const isDragging = ref(false);

export function useMiniPlayerDraggable() {
  let startX = 0;
  let startY = 0;
  let startPosX = 0;
  let startPosY = 0;
  let hasMoved = false;
  let suppressClick = false;

  function onPointerDown(e: PointerEvent): void {
    if (e.button !== 0) return;

    const target = e.target as HTMLElement | null;
    if (target?.closest("button[data-action]")) {
      return;
    }

    const host = document.getElementById("mini-player-host");
    if (!host) return;

    const rect = host.getBoundingClientRect();
    startX = e.clientX;
    startY = e.clientY;
    startPosX = rect.left;
    startPosY = rect.top;
    hasMoved = false;

    const currentTarget = e.currentTarget as HTMLElement | null;
    if (currentTarget) {
      currentTarget.setPointerCapture(e.pointerId);
    }

    const onPointerMove = (moveEvent: PointerEvent) => {
      const dx = moveEvent.clientX - startX;
      const dy = moveEvent.clientY - startY;

      if (!hasMoved && Math.hypot(dx, dy) > 4) {
        hasMoved = true;
        isDragging.value = true;
      }

      if (hasMoved) {
        const hostEl = document.getElementById("mini-player-host");
        const w = hostEl ? hostEl.offsetWidth : 384;
        const h = hostEl ? hostEl.offsetHeight : 260;
        const pad = 12;

        const maxX = Math.max(pad, window.innerWidth - w - pad);
        const maxY = Math.max(pad, window.innerHeight - h - pad);

        position.value = {
          x: Math.max(pad, Math.min(maxX, startPosX + dx)),
          y: Math.max(pad, Math.min(maxY, startPosY + dy)),
        };
      }
    };

    const onPointerUp = (upEvent: PointerEvent) => {
      if (currentTarget) {
        try {
          currentTarget.releasePointerCapture(upEvent.pointerId);
        } catch {}
      }

      window.removeEventListener("pointermove", onPointerMove);
      window.removeEventListener("pointerup", onPointerUp);
      window.removeEventListener("pointercancel", onPointerUp);

      if (hasMoved) {
        suppressClick = true;
        setTimeout(() => {
          suppressClick = false;
        }, 80);
      }
      isDragging.value = false;
    };

    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("pointerup", onPointerUp);
    window.addEventListener("pointercancel", onPointerUp);
  }

  function shouldSuppressClick(): boolean {
    return suppressClick;
  }

  function clampToViewport(): void {
    if (!position.value) return;
    const hostEl = document.getElementById("mini-player-host");
    const w = hostEl ? hostEl.offsetWidth : 384;
    const h = hostEl ? hostEl.offsetHeight : 260;
    const pad = 12;

    const maxX = Math.max(pad, window.innerWidth - w - pad);
    const maxY = Math.max(pad, window.innerHeight - h - pad);

    position.value = {
      x: Math.max(pad, Math.min(maxX, position.value.x)),
      y: Math.max(pad, Math.min(maxY, position.value.y)),
    };
  }

  const hostStyle = computed(() => {
    if (!position.value) {
      return undefined;
    }
    return {
      left: `${position.value.x}px`,
      top: `${position.value.y}px`,
      bottom: "auto",
      right: "auto",
    };
  });

  return {
    position,
    isDragging,
    hostStyle,
    onPointerDown,
    shouldSuppressClick,
    clampToViewport,
  };
}
