<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import WindowControls from "@/components/layout/WindowControls.vue";

const isTauri = () => {
  return (
    typeof window !== "undefined" &&
    Boolean((window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__)
  );
};

async function handleTitleBarDblClick(e: MouseEvent) {
  const target = e.target as HTMLElement | null;
  if (target?.closest("button, [data-tauri-drag-region='false']")) {
    return;
  }
  if (!isTauri()) return;
  try {
    const win = getCurrentWindow();
    await win.toggleMaximize();
  } catch (err) {
    console.warn("Toggle maximize failed:", err);
  }
}
</script>

<template>
  <header
    data-tauri-drag-region
    class="flex h-8 w-full select-none items-center justify-between border-b border-sidebar-border bg-background/95 z-50 shrink-0"
    @dblclick="handleTitleBarDblClick"
  >
    <!-- Left: App Icon & Title -->
    <div
      data-tauri-drag-region
      class="flex items-center gap-2 px-3 h-full cursor-default select-none"
    >
      <img
        src="/app-icon.png"
        alt="r-tube"
        class="h-4 w-4 rounded-xs object-contain pointer-events-none"
      />
      <span class="text-xs font-medium text-foreground/80 tracking-wide font-sans">
        r-tube
      </span>
    </div>

    <!-- Center Drag Region -->
    <div
      data-tauri-drag-region
      class="flex-1 h-full cursor-default"
    />

    <!-- Right: Window Controls -->
    <WindowControls />
  </header>
</template>

