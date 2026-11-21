<script setup lang="ts">
import { Check, Volume2, VolumeX } from "@lucide/vue";
import type { AnySettingDefinition } from "./types";

defineOptions({ name: "SettingRow" });

defineProps<{
  item: AnySettingDefinition;
  value: string | number | null;
  saved?: boolean;
}>();

const emit = defineEmits<{
  (e: "change", value: string | number | null): void;
}>();
</script>

<template>
  <div
    class="p-4 sm:p-5 flex flex-col sm:flex-row sm:items-center justify-between gap-4 transition-colors hover:bg-muted/10"
  >
    <!-- Title & Description -->
    <div class="space-y-1 pr-4 max-w-md">
      <div class="flex items-center gap-2">
        <span class="text-sm font-medium text-foreground">{{ item.title }}</span>
        <!-- Save Indicator badge -->
        <transition
          enter-active-class="transition duration-150 ease-out"
          enter-from-class="opacity-0 scale-90"
          enter-to-class="opacity-100 scale-100"
          leave-active-class="transition duration-150 ease-in"
          leave-from-class="opacity-100 scale-100"
          leave-to-class="opacity-0 scale-90"
        >
          <span
            v-if="saved"
            class="inline-flex items-center gap-1 text-[11px] font-medium text-emerald-500 bg-emerald-500/10 px-2 py-0.5 rounded-full"
          >
            <Check class="h-3 w-3" />
            <span>Saved</span>
          </span>
        </transition>
      </div>
      <p class="text-xs text-muted-foreground leading-relaxed">
        {{ item.description }}
      </p>
    </div>

    <!-- Controls by Type -->
    <div class="sm:shrink-0 flex items-center">
      <!-- ── A. SELECT DROPDOWN ── -->
      <Select
        v-if="item.type === 'select'"
        :model-value="String(value ?? '')"
        @update:model-value="(value) => emit('change', value as string | number | null)"
      >
        <SelectTrigger class="w-full md:w-42 h-9">
          <SelectValue placeholder="Select an option" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem
            v-for="opt in item.options"
            :key="String(opt.value)"
            :value="String(opt.value)"
          >
            {{ opt.label }}
          </SelectItem>
        </SelectContent>
      </Select>

      <!-- ── B. SEGMENTED CONTROLS ── -->
      <div
        v-else-if="item.type === 'segmented'"
        class="inline-flex p-1 rounded-lg bg-muted/30 border border-border/50 gap-1"
      >
        <button
          v-for="opt in item.options"
          :key="String(opt.value)"
          type="button"
          class="px-3 py-1.5 text-xs font-medium rounded-md transition-all cursor-pointer select-none"
          :class="
            value === opt.value
              ? 'bg-background text-foreground shadow-xs font-semibold'
              : 'text-muted-foreground hover:text-foreground'
          "
          @click="emit('change', opt.value)"
        >
          {{ opt.label }}
        </button>
      </div>

      <!-- ── C. VOLUME SLIDER ── -->
      <div
        v-else-if="item.type === 'slider'"
        class="flex items-center gap-3 w-full sm:w-56"
      >
        <button
          type="button"
          class="text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
          @click="emit('change', Number(value) > 0 ? 0 : 1)"
        >
          <VolumeX v-if="Number(value) === 0" class="h-4 w-4 text-rose-500" />
          <Volume2 v-else class="h-4 w-4" />
        </button>

        <input
          type="range"
          :min="item.min ?? 0"
          :max="item.max ?? 1"
          :step="item.step ?? 0.05"
          :value="value ?? 1"
          class="w-full accent-primary h-1.5 bg-muted rounded-lg cursor-pointer"
          @input="
            emit('change', Number(($event.target as HTMLInputElement).value))
          "
        />

        <span
          class="text-xs font-mono font-medium text-foreground w-10 text-right"
        >
          {{ Math.round(Number(value ?? 1) * 100) }}%
        </span>
      </div>
    </div>
  </div>
</template>