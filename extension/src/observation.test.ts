import { describe, it, expect } from 'vitest';
import { snapshot, ObservationGeneration } from './observation';
const win = { id: 1, focused: true };
describe('Privacy and focused tab observation', () => {
  it('removes paths, queries, fragments and titles before creating a message', () => {
    const result = snapshot(win, { id: 2, windowId: 1, url: 'https://YouTube.com/private/SECRET?q=TOKEN#FRAGMENT' });
    expect(result.hostname).toBe('youtube.com');
    expect(JSON.stringify(result)).not.toMatch(/SECRET|TOKEN|FRAGMENT|https|private/);
  });
  it('does not collect tabs in background, another window or incognito', () => {
    const tab = { id: 2, windowId: 1, url: 'https://example.org' };
    expect(snapshot({ ...win, focused: false }, tab).hostname).toBeNull();
    expect(snapshot(win, { ...tab, windowId: 3 }).hostname).toBeNull();
    expect(snapshot(win, { ...tab, incognito: true }).hostname).toBeNull();
    expect(snapshot({ ...win, incognito: true }, tab).hostname).toBeNull();
  });
  it('preserves exact subdomains and handles unavailable and internal addresses', () => {
    expect(snapshot(win, { id: 2, windowId: 1, url: 'https://www.docs.example.co.uk.' }).hostname).toBe('www.docs.example.co.uk');
    expect(snapshot(win, { id: 2, windowId: 1 }).hostname).toBeNull();
    expect(snapshot(win, { id: 2, windowId: 1, url: 'chrome://settings/passwords' }).system).toBe(true);
    expect(snapshot(win, { id: 2, windowId: 1, url: 'file:///C:/secret.txt' }).hostname).toBeNull();
  });
  it('rejects asynchronous snapshots from an earlier focus generation', () => {
    const guard = new ObservationGeneration(); const old = guard.next(); const fresh = guard.next();
    expect(guard.isCurrent(old)).toBe(false); expect(guard.isCurrent(fresh)).toBe(true);
  });
});
