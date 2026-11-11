export interface CaptionCue {
  start: number;
  end: number;
  text: string;
}

const NAMED_ENTITIES: Record<string, string> = {
  amp: "&",
  lt: "<",
  gt: ">",
  quot: '"',
  apos: "'",
  nbsp: " ",
};

/**
 * YouTube's caption files are HTML-ish: tags are stripped, but the text also
 * carries entities such as `&gt;&gt;` or `&#39;`. Without this the lyrics show
 * the entity literally instead of the character.
 */
function decodeEntities(input: string): string {
  return input.replace(/&(#x[0-9a-fA-F]+|#[0-9]+|[a-zA-Z]+);/g, (match, entity: string) => {
    if (entity.startsWith("#x") || entity.startsWith("#X")) {
      const code = Number.parseInt(entity.slice(2), 16);
      return decodeCodePoint(code, match);
    }
    if (entity.startsWith("#")) {
      const code = Number.parseInt(entity.slice(1), 10);
      return decodeCodePoint(code, match);
    }
    return NAMED_ENTITIES[entity.toLowerCase()] ?? match;
  });
}

function decodeCodePoint(code: number, fallback: string): string {
  if (!Number.isFinite(code) || code < 0 || code > 0x10ffff) return fallback;
  try {
    return String.fromCodePoint(code);
  } catch {
    return fallback;
  }
}

function cleanCueText(raw: string): string {
  return decodeEntities(
    raw
      // Inline styling/timing tags: <c>, <00:00:01.500>, <b>, <br>…
      .replace(/<[^>]*>/g, "")
      // WebVTT line breaks
      .replace(/\\N|\\n/g, "\n"),
  )
    .replace(/\u00a0/g, " ")
    .replace(/[ \t]+/g, " ")
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean)
    .join("\n")
    .trim();
}

function parseTimestamp(value: string): number {
  const parts = value.trim().split(":");
  if (parts.length === 3) {
    const [hours, minutes, seconds] = parts;
    return (
      Number(hours) * 3600 +
      Number(minutes) * 60 +
      Number(seconds.replace(",", "."))
    );
  }
  if (parts.length === 2) {
    const [minutes, seconds] = parts;
    return Number(minutes) * 60 + Number(seconds.replace(",", "."));
  }
  return Number(value.replace(",", "."));
}

/** Minimal WebVTT parser: enough for YouTube's caption files. */
export function parseVtt(raw: string): CaptionCue[] {
  const cues: CaptionCue[] = [];
  const blocks = raw.replace(/\r/g, "").split(/\n{2,}/);

  for (const block of blocks) {
    const lines = block.split("\n").filter((line) => line.trim().length > 0);
    const timingIndex = lines.findIndex((line) => line.includes("-->"));
    if (timingIndex < 0) continue;

    const [startRaw, endRaw] = lines[timingIndex].split("-->");
    const start = parseTimestamp(startRaw.trim().split(/\s+/)[0]);
    const end = parseTimestamp(endRaw.trim().split(/\s+/)[0]);
    const text = cleanCueText(lines.slice(timingIndex + 1).join("\n"));

    if (Number.isFinite(start) && Number.isFinite(end) && text) {
      cues.push({ start, end, text });
    }
  }

  return cues;
}
