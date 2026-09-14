import type { Segment } from './types';

export function summarizePeriods(periods: Segment[]) {
  const gaps: { before: Segment; after: Segment; duration_ms: number }[] = [];
  // The report is in observation order (SQLite id), even after a clock change.
  for (let index = 1; index < periods.length; index += 1) {
    const before = periods[index - 1];
    const after = periods[index];
    const duration_ms = after.start_utc - before.end_utc;
    if (duration_ms > 0 && after.reason !== 'clock_adjusted') gaps.push({ before, after, duration_ms });
  }
  const largestGap = gaps.reduce<typeof gaps[number] | null>((largest, gap) => !largest || gap.duration_ms > largest.duration_ms ? gap : largest, null);
  return { first: periods[0] ?? null, last: periods.at(-1) ?? null, gaps, largestGap };
}

export const timelineStates = {
  vdi: 'VDI', browser: 'Navegador', app: 'Aplicativo local', system: 'Sistema',
  idle: 'Inatividade', paused: 'Pausa', unknown: 'Desconhecido', unobserved: 'Não observado',
};

export function timelineState(segment: Segment): keyof typeof timelineStates {
  return Object.hasOwn(timelineStates, segment.source) ? segment.source as keyof typeof timelineStates : 'unknown';
}

export interface TimelineBlock {
  id: string;
  segment: Segment;
  start_utc: number;
  end_utc: number;
  duration_ms: number;
  merged_interruptions: Segment[];
}

const SHORT_INTERRUPTION_MS = 5_000;
const INTERRUPTION_CATEGORIES = new Set(['system', 'unknown']);

function touches(left: TimelineBlock, right: TimelineBlock) {
  return Math.abs(left.end_utc - right.start_utc) <= 1_000;
}

export function groupTimelinePeriods(periods: Segment[], categoryFor: (segment: Segment) => string): TimelineBlock[] {
  const blocks = periods.map(segment => ({ id: String(segment.id), segment, start_utc: segment.start_utc, end_utc: segment.end_utc, duration_ms: segment.duration_ms, merged_interruptions: [] as Segment[] }));
  for (let index = 1; index < blocks.length - 1;) {
    const left = blocks[index - 1];
    const interruption = blocks[index];
    const right = blocks[index + 1];
    const isShortInterruption = interruption.duration_ms <= SHORT_INTERRUPTION_MS && INTERRUPTION_CATEGORIES.has(categoryFor(interruption.segment));
    if (isShortInterruption && categoryFor(left.segment) === categoryFor(right.segment) && touches(left, interruption) && touches(interruption, right)) {
      left.end_utc = right.end_utc;
      left.duration_ms += interruption.duration_ms + right.duration_ms;
      left.merged_interruptions.push(...interruption.merged_interruptions, interruption.segment, ...right.merged_interruptions);
      blocks.splice(index, 2);
      index = Math.max(1, index - 1);
    } else index += 1;
  }
  return blocks;
}
