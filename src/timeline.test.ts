import { expect, it } from 'vitest';
import { groupTimelinePeriods, summarizePeriods, timelineState } from './timeline';
import type { Segment } from './types';

function segment(id: number, source: string, start: number, duration_ms: number): Segment {
  return { id, run_id: 'run', start_utc: start, end_utc: start + duration_ms, duration_ms, local_date: '2026-09-11', offset_seconds: 0, source, hostname: null, app_name: null, reason: 'test', is_open: false };
}

it('groups a short system interruption between periods in the same category', () => {
  const periods = [segment(1, 'vdi', 0, 60_000), segment(2, 'system', 60_000, 4_000), segment(3, 'vdi', 64_000, 60_000)];
  const result = groupTimelinePeriods(periods, item => item.source);
  expect(result).toHaveLength(1);
  expect(result[0]).toMatchObject({ start_utc: 0, end_utc: 124_000, duration_ms: 124_000 });
  expect(result[0].merged_interruptions.map(item => item.id)).toEqual([2]);
});

it('preserves short interruptions when the surrounding categories differ', () => {
  const periods = [segment(1, 'vdi', 0, 60_000), segment(2, 'system', 60_000, 4_000), segment(3, 'app', 64_000, 60_000)];
  expect(groupTimelinePeriods(periods, item => item.source)).toHaveLength(3);
});

it('uses first and last raw records and finds the largest internal gap', () => {
  const periods = [segment(1, 'vdi', 3_600_000, 60_000), segment(2, 'app', 3_720_000, 60_000), segment(3, 'vdi', 3_960_000, 60_000)];
  const result = summarizePeriods(periods);
  expect(result.first).toBe(periods[0]);
  expect(result.last).toBe(periods[2]);
  expect(result.gaps.map(gap => gap.duration_ms)).toEqual([60_000, 180_000]);
  expect(result.largestGap).toEqual({ before: periods[1], after: periods[2], duration_ms: 180_000 });
});

it('has no boundaries or gaps for an empty day', () => {
  expect(summarizePeriods([])).toEqual({ first: null, last: null, gaps: [], largestGap: null });
});

it('does not infer unobserved time before or after a single period', () => {
  const period = segment(1, 'vdi', 3_600_000, 60_000);
  expect(summarizePeriods([period])).toEqual({ first: period, last: period, gaps: [], largestGap: null });
});

it('keeps consecutive periods gap-free, including pauses and recorded unobserved states', () => {
  const periods = ['vdi', 'paused', 'unobserved', 'idle'].map((source, index) => segment(index, source, index * 60_000, 60_000));
  expect(summarizePeriods(periods).gaps).toEqual([]);
});

it('retains the last confirmed end of an open session, including zero duration', () => {
  for (const ms of [0, 60_000]) {
    const period = { ...segment(1, 'vdi', 3_600_000, ms), is_open: true };
    expect(summarizePeriods([period]).last).toEqual(period);
  }
});

it('does not hide a raw gap accepted by visual grouping', () => {
  const periods = [segment(1, 'vdi', 0, 60_000), segment(2, 'system', 60_500, 4_000), segment(3, 'vdi', 64_500, 60_000)];
  expect(groupTimelinePeriods(periods, item => item.source)).toHaveLength(1);
  expect(summarizePeriods(periods).largestGap?.duration_ms).toBe(500);
});

it('preserves observation order after clock rollback and excludes known clock jumps from gaps', () => {
  const first = segment(1, 'vdi', 3_600_000, 60_000);
  const last = { ...segment(2, 'app', 0, 60_000), reason: 'clock_adjusted', offset_seconds: -10_800 };
  expect(summarizePeriods([first, last])).toMatchObject({ first, last, largestGap: null });
  expect(summarizePeriods([last, { ...first, reason: 'clock_adjusted' }]).largestGap).toBeNull();
});

it('maps raw states independently of site or application category', () => {
  for (const source of ['vdi', 'browser', 'app', 'system', 'idle', 'paused', 'unknown', 'unobserved']) {
    expect(timelineState(segment(1, source, 0, 60_000))).toBe(source);
  }
  expect(timelineState(segment(1, 'unrecognized', 0, 60_000))).toBe('unknown');
});
