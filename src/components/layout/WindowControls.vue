<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from "vue";
import { Minus, Square, Copy, X } from "@lucide/vue";
import { getCurrentWindow } from "@tauri-apps/api/window";

const isMaximized = ref(false);
let unlistenResize: (() => void) | null = null;

const isTauri = () => {
  return (
    typeof window !== "undefined" &&
    Boolean((window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__)
  );
};

onMounted(async () => {
  if (!isTauri()) return;
  try {
    const appWindow = getCurrentWindow();
    isMaximized.value = await appWindow.isMaximized();
    unlistenResize = await appWindow.onResized(async () => {
      isMaximized.value = await appWindow.isMaximized();
    });
  } catch (err) {
    console.warn("Failed to initialize window controls listener:", err);
  }
});

onBeforeUnmount(() => {
  if (unlistenResize) {
    unlistenResize();
    unlistenResize = null;
  }
});

async function handleMinimize() {
  if (!isTauri()) return;
  try {
    await getCurrentWindow().minimize();
  } catch (err) {
    console.error("Failed to minimize window:", err);
  }
}

async function handleToggleMaximize() {
  if (!isTauri()) return;
  try {
    const win = getCurrentWindow();
    await win.toggleMaximize();
    isMaximized.value = await win.isMaximized();
  } catch (err) {
    console.error("Failed to toggle maximize window:", err);
  }
}

async function handleClose() {
  if (!isTauri()) return;
  try {
    await getCurrentWindow().close();
  } catch (err) {
    console.error("Failed to close window:", err);
  }
}
</script>

<template>
  <div
    class="flex items-center h-full select-none shrink-0"
    data-tauri-drag-region="false"
    @mousedown.stop
  >
    <button
      type="button"
      tabindex="-1"
      title="Minimize"
      aria-label="Minimize"
      class="inline-flex h-full w-11 items-center justify-center text-foreground/75 transition-colors hover:bg-accent/80 hover:text-foreground focus:outline-hidden cursor-pointer"
      @click="handleMinimize"
    >
      <Minus class="h-3.5 w-3.5" />
    </button>

    <button
      type="button"
      tabindex="-1"
      :title="isMaximized ? 'Restore Down' : 'Maximize'"
      :aria-label="isMaximized ? 'Restore Down' : 'Maximize'"
      class="inline-flex h-full w-11 items-center justify-center text-foreground/75 transition-colors hover:bg-accent/80 hover:text-foreground focus:outline-hidden cursor-pointer"
      @click="handleToggleMaximize"
    >
      <Copy v-if="isMaximized" class="h-3 w-3 rotate-90" />
      <Square v-else class="h-3 w-3" />
    </button>

    <button
      type="button"
      tabindex="-1"
      title="Close"
      aria-label="Close"
      class="inline-flex h-full w-12 items-center justify-center text-foreground/75 transition-colors hover:bg-destructive hover:text-destructive-foreground focus:outline-hidden cursor-pointer"
      @click="handleClose"
    >
      <X class="h-4 w-4" />
    </button>
  </div>
</template>
