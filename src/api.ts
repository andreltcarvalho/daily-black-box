import { invoke } from '@tauri-apps/api/core';
export function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!('__TAURI_INTERNALS__' in window)) return Promise.reject(new Error('Abra o aplicativo Windows para conectar a coleta local. Esta prévia de interface não registra atividade.'));
  return invoke<T>(command, args);
}
export function today() { const d = new Date(); return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`; }
export function duration(ms: number) {
  const seconds = Math.floor(ms / 1000);
  if (seconds < 60) return `${seconds}s`;
  const minutes = Math.floor(seconds / 60);
  return minutes < 60 ? `${minutes}min` : `${Math.floor(minutes / 60)}h ${String(minutes % 60).padStart(2, '0')}min`;
}
export function clock(utc: number, offset: number) { return new Date(utc + offset * 1000).toISOString().slice(11, 16); }
export function sourceLabel(source: string, hostname?: string | null, appName?: string | null) {
  return hostname ?? appName ?? ({ vdi: 'VDI / trabalho focado', idle: 'Inatividade', paused: 'Rastreamento pausado', unobserved: 'Sem coleta', system: 'Sistema', unknown: 'Desconhecido' }[source] ?? 'Desconhecido');
}
