<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { Captions, ChevronDown } from "@lucide/vue";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuTrigger,
} from "@/lib/components/ui/dropdown-menu";
import { useAudioPlayer } from "@/composables/useAudioPlayer";
import { usePlayer } from "@/composables/usePlayer";
import type { CaptionCue } from "@/lib/vtt";

defineOptions({ name: "ListenCaptions" });

const audio = useAudioPlayer();
const player = usePlayer();

const tracks = computed(() => audio.captionTracks.value);
const language = computed(() => audio.captionLanguage.value);
const cues = computed(() => audio.captionCueList.value);

/**
 * The audio engine only publishes `currentTime` on `timeupdate` (~4 Hz), which
 * makes the fill step visibly. Sampling the element at ~12 Hz keeps the
 * karaoke edge moving smoothly without re-rendering every frame.
 */
const now = ref(0);
let sampler: ReturnType<typeof setInterval> | null = null;

onMounted(() => {
  sampler = setInterval(() => {
    now.value =
      player.activeMedia.value?.currentTime ?? player.currentTime.value;
  }, 80);
});

onBeforeUnmount(() => {
  if (sampler) {
    clearInterval(sampler);
    sampler = null;
  }
});

function cueIndexAt(time: number): number {
  return cues.value.findIndex((cue) => time >= cue.start && time < cue.end);
}

const activeIndex = computed(() => cueIndexAt(now.value));
const activeCue = computed<CaptionCue | null>(() =>
  activeIndex.value >= 0 ? cues.value[activeIndex.value] : null,
);
const previousCue = computed<CaptionCue | null>(() =>
  activeIndex.value > 0 ? cues.value[activeIndex.value - 1] : null,
);
const nextCue = computed<CaptionCue | null>(() =>
  activeIndex.value >= 0 && activeIndex.value + 1 < cues.value.length
    ? cues.value[activeIndex.value + 1]
    : null,
);

interface GraphemeSegmenter {
  segment(input: string): Iterable<{ segment: string }>;
}

type GraphemeSegmenterCtor = new (
  locales?: string | string[],
  options?: { granularity: "grapheme" },
) => GraphemeSegmenter;

// `Intl.Segmenter` is not in this project's TS lib target, but WebView2 has it.
const Segmenter = (Intl as unknown as { Segmenter?: GraphemeSegmenterCtor })
  .Segmenter;

let graphemeSegmenter: GraphemeSegmenter | null = null;

function splitGraphemes(text: string): string[] {
  if (!text) return [];
  if (!Segmenter) return Array.from(text);
  try {
    graphemeSegmenter ??= new Segmenter(undefined, { granularity: "grapheme" });
    return Array.from(graphemeSegmenter.segment(text), (part) => part.segment);
  } catch {
    return Array.from(text);
  }
}

const graphemes = computed(() => splitGraphemes(activeCue.value?.text ?? ""));

/** How many graphemes are already "sung" in the active cue. */
const filled = computed(() => {
  const cue = activeCue.value;
  if (!cue || graphemes.value.length === 0) return 0;
  const span = Math.max(0.001, cue.end - cue.start);
  const progress = Math.min(1, Math.max(0, (now.value - cue.start) / span));
  return Math.floor(progress * graphemes.value.length);
});

const isOn = computed(() => Boolean(language.value));

function selectLanguage(next: unknown): void {
  audio.setCaptionLanguage(String(next ?? ""));
}
</script>

<template>
  <div v-if="tracks.length > 0" class="mx-auto mt-6 w-full max-w-2xl px-2">
    <!-- Language picker (shadcn dropdown) -->
    <div class="flex justify-center">
      <DropdownMenu>
        <DropdownMenuTrigger as-child>
          <button
            type="button"
            class="inline-flex cursor-pointer items-center gap-1.5 rounded-full border border-border/60 bg-background/60 px-3 py-1 text-[11px] font-medium text-muted-foreground transition-colors hover:bg-muted/60 hover:text-foreground"
          >
            <Captions class="h-3.5 w-3.5" />
            {{
              language
                ? tracks.find((track) => track.language === language)?.name ||
                  "Captions"
                : "Captions off"
            }}
            <ChevronDown class="h-3 w-3 opacity-70" />
          </button>
        </DropdownMenuTrigger>

        <DropdownMenuContent align="center" class="min-w-40">
          <DropdownMenuRadioGroup
            :model-value="language"
            @update:model-value="selectLanguage"
          >
            <DropdownMenuRadioItem value="">
              Captions off
            </DropdownMenuRadioItem>
            <DropdownMenuRadioItem
              v-for="track in tracks"
              :key="track.id"
              :value="track.language"
            >
              {{ track.name }}
            </DropdownMenuRadioItem>
          </DropdownMenuRadioGroup>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>

    <!-- Karaoke line: previous / current / next -->
    <div
      v-if="isOn"
      class="mt-3 flex min-h-38 flex-col items-center justify-center gap-2 text-center"
    >
      <p class="line-clamp-1 text-xs text-muted-foreground/45">
        {{ previousCue?.text || "\u00a0" }}
      </p>

      <p
        v-if="activeCue"
        class="text-balance text-2xl font-semibold leading-snug tracking-tight sm:text-3xl"
        aria-live="off"
      >
        <span
          v-for="(part, index) in graphemes"
          :key="index"
          class="transition-colors duration-150 ease-linear"
          :class="index < filled ? 'text-foreground' : 'text-foreground/30'"
          >{{ part }}</span
        >
      </p>
      <p
        v-else
        class="text-2xl font-semibold leading-snug text-foreground/20 sm:text-3xl"
      >
        ♪
      </p>

      <p class="line-clamp-1 text-xs text-muted-foreground/45">
        {{ nextCue?.text || "\u00a0" }}
      </p>
    </div>
  </div>
</template>
