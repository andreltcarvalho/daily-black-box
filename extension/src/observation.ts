export interface TabMetadata { id?: number; windowId?: number; url?: string; incognito?: boolean }
export interface WindowMetadata { id?: number; focused?: boolean; incognito?: boolean; tabs?: TabMetadata[] }
export interface Snapshot { focused: boolean; hostname: string | null; tab_id: number | null; window_id: number | null; system: boolean }
export function snapshot(window: WindowMetadata | null, tab: TabMetadata | null): Snapshot {
  const empty: Snapshot = { focused: false, hostname: null, tab_id: null, window_id: null, system: false };
  if (!window?.focused || window.incognito || !tab || tab.incognito || window.id !== tab.windowId) return empty;
  if (tab.id === undefined || window.id === undefined) return empty;
  const result: Snapshot = { ...empty, focused: true, tab_id: tab.id, window_id: window.id };
  if (!tab.url) return result;
  try {
    const url = new URL(tab.url);
    if (url.protocol === 'http:' || url.protocol === 'https:') result.hostname = url.hostname.toLowerCase().replace(/\.$/, '');
    else if (url.protocol === 'chrome:' || url.protocol === 'chrome-extension:') result.system = true;
  } catch { /* An unavailable or unsupported URL stays unknown. */ }
  return result;
}
export class ObservationGeneration {
  private value = 0;
  next() { return ++this.value; }
  isCurrent(value: number) { return this.value === value; }
}
