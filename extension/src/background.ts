import { ObservationGeneration, snapshot } from './observation';
const HOST = 'local.caixapreta.dodia';
let port: chrome.runtime.Port | null = null;
let generation = 0;
let sequence = 0;
let paused = true;
let connected = false;
let browserInFocus = false;
let lastSnapshot = 0;
let lastResponse = 0;
let statusError: string | null = null;
const queries = new ObservationGeneration();

async function collect() {
  // Do not even request tab metadata during manual pause or when Chrome is not in focus.
  if (!port || !connected || paused || !browserInFocus) return;
  const query = queries.next();
  const requestGeneration = generation;
  try {
    const windows = await chrome.windows.getAll({ windowTypes: ['normal', 'popup'] });
    const focused = windows.find(w => w.focused && !w.incognito);
    const tabs = focused?.id === undefined ? [] : await chrome.tabs.query({ windowId: focused.id, active: true });
    if (!queries.isCurrent(query) || paused || generation !== requestGeneration || !port) return;
    const state = snapshot(focused ?? null, tabs[0] ?? null);
    port.postMessage({ kind: 'browser_state', generation, sequence: ++sequence, ...state });
    lastSnapshot = Date.now();
  } catch { /* Reconcile on the next poll, never fall back to an old domain. */ }
}
function invalidate() { queries.next(); lastSnapshot = 0; void collect(); }
function connect() {
  if (port) return;
  paused = true; connected = false; lastResponse = 0; queries.next();
  try {
    const current = chrome.runtime.connectNative(HOST); port = current;
    current.onMessage.addListener(message => {
      if (port !== current || message?.kind !== 'tracking_state' || message.version !== 1) return;
      lastResponse = Date.now(); connected = true; statusError = message.error ?? null;
      const changed = generation !== message.generation || paused !== message.paused;
      generation = message.generation; paused = message.paused || Boolean(message.error);
      browserInFocus = Boolean(message.browser_in_focus);
      if (changed) { queries.next(); lastSnapshot = 0; }
      void chrome.action.setBadgeText({ text: paused ? 'Ⅱ' : '' });
      if (!paused && browserInFocus && (changed || Date.now() - lastSnapshot >= 5000)) void collect();
    });
    current.onDisconnect.addListener(() => {
      statusError = chrome.runtime.lastError?.message ?? 'Aplicativo desconectado';
      if (port !== current) return;
      port = null; connected = false; paused = true; queries.next();
      void chrome.action.setBadgeText({ text: '!' });
    });
    current.postMessage({ kind: 'hello', version: 1 });
  } catch { port = null; connected = false; statusError = 'Não foi possível conectar ao aplicativo.'; }
}
chrome.tabs.onActivated.addListener(invalidate);
chrome.tabs.onUpdated.addListener((_id, change) => { if (change.url !== undefined) invalidate(); });
chrome.tabs.onRemoved.addListener(invalidate);
chrome.tabs.onReplaced.addListener(invalidate);
chrome.windows.onFocusChanged.addListener(invalidate);
chrome.windows.onRemoved.addListener(invalidate);
chrome.runtime.onInstalled.addListener(connect);
chrome.runtime.onStartup.addListener(connect);
chrome.alarms.onAlarm.addListener(alarm => { if (alarm.name === 'reconnect') connect(); });
void chrome.alarms.create('reconnect', { periodInMinutes: 0.5 });
chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (message?.kind === 'reconnect') { port?.disconnect(); port = null; connect(); }
  if (message?.kind === 'status' || message?.kind === 'reconnect') sendResponse({ connected, paused, error: statusError });
});
// The connected native port keeps the worker alive; timers are never relied on across restarts.
setInterval(() => {
  if (!port) return;
  if (lastResponse > 0 && Date.now() - lastResponse > 10_000) {
    paused = true; connected = false; queries.next(); port.disconnect(); port = null; return;
  }
  try { port.postMessage({ kind: 'poll' }); } catch { port = null; paused = true; connected = false; }
}, 1000);
connect();
