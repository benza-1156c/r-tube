<script setup lang="ts">
import "@videojs/html/ui/menu";
import "@videojs/html/ui/menu-content";
import "@videojs/html/ui/menu-radio-item";
import "@videojs/html/ui/menu-item-indicator";
import "@videojs/html/ui/quality-radio-group";
import { Settings } from "@lucide/vue";

defineOptions({ name: "PlayerQualityMenu" });
</script>

<template>
  <!--
    The quality list itself comes from Video.js: it reads the renditions the
    HLS engine reports. Only the placement and the theme live here. The menu
    is hidden when the engine exposes no renditions to switch between.
  -->
  <div class="quality-menu-bar flex items-center" @click.stop>
    <button
      type="button"
      class="rt-chrome-btn"
      commandfor="rt-quality-menu"
      title="Quality"
      aria-label="Change quality"
    >
      <Settings class="h-4 w-4" />
    </button>

    <media-menu
      id="rt-quality-menu"
      side="bottom"
      align="end"
      class="quality-menu"
    >
      <media-menu-content>
        <media-quality-radio-group class="quality-group" label="Quality" />
      </media-menu-content>
    </media-menu>
  </div>
</template>

<style>
/*
 * Quality menu styling, following the Video.js quality guide. The markup and
 * behaviour come from Video.js; only the theme lives here.
 */
.quality-menu-bar:has([data-availability="unavailable"]) {
  display: none;
}

.quality-menu {
  --media-menu-side-offset: 8px;
  box-sizing: border-box;
  min-width: 10rem;
  max-height: 16rem;
  padding: 6px;
  overflow: auto;
  overscroll-behavior: none;
  font-size: 12px;
  color: hsl(var(--popover-foreground));
  background: hsl(var(--popover) / 0.97);
  border: 1px solid hsl(var(--border));
  border-radius: 12px;
  box-shadow: 0 12px 32px rgb(0 0 0 / 0.45);
  backdrop-filter: blur(12px);
}

.quality-group {
  display: grid;
  gap: 2px;
}

.quality-menu media-menu-radio-item,
.quality-group media-menu-radio-item {
  display: flex;
  gap: 8px;
  align-items: center;
  justify-content: space-between;
  min-height: 30px;
  padding: 0 10px;
  font: inherit;
  color: inherit;
  cursor: pointer;
  background: none;
  border: 0;
  border-radius: 8px;
}

.quality-menu media-menu-radio-item[data-highlighted],
.quality-menu media-menu-radio-item[aria-checked="true"] {
  background: hsl(var(--accent) / 0.6);
}

.quality-menu media-menu-item-indicator {
  opacity: 0;
  margin-left: auto;
}

.quality-menu
  media-menu-radio-item[aria-checked="true"]
  media-menu-item-indicator {
  opacity: 1;
}
</style>
