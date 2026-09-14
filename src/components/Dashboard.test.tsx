import { cleanup, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import { Dashboard } from './Dashboard';
import { emptyDay } from '../test-fixtures';
import type { Segment } from '../types';
afterEach(cleanup);
it('edits a domain category and exposes text equivalents for charts', () => {
  const report = emptyDay('2026-09-10');
  report.domains = [{ hostname: 'example.com', category: 'unknown', duration_ms: 60000, accesses: 2 }];
  report.category_totals = [{ label: 'unknown', duration_ms: 60000 }];
  const edit = vi.fn(); render(<Dashboard report={report} onClassify={edit} onClassifyApp={vi.fn()} onExport={vi.fn()}/>);
  fireEvent.change(screen.getByRole('combobox', { name: 'Categoria de example.com' }), { target: { value: 'video' } });
  expect(edit).toHaveBeenCalledWith('example.com', 'video');
  expect(screen.getByRole('button', { name: 'example.com: 1min' })).toBeInTheDocument();
  fireEvent.change(screen.getByRole('searchbox'), { target: { value: 'no-match' } });
  expect(screen.getByText('Nenhum domínio corresponde ao filtro.')).toBeInTheDocument();
});

it('shows focused application names without exposing window content', () => {
  const report = emptyDay('2026-09-10');
  report.categories.push({ id: 'entertainment', name: 'Entretenimento', distraction: false });
  report.app_totals = [{ label: 'League of Legends', category: 'local', duration_ms: 120000 }];
  report.sessions = [{ id: 1, run_id: 'run', start_utc: 0, end_utc: 120000, duration_ms: 120000, local_date: '2026-09-10', offset_seconds: 0, source: 'app', hostname: null, app_name: 'League of Legends', reason: 'foreground_app', is_open: false }];
  const edit = vi.fn(); render(<Dashboard report={report} onClassify={vi.fn()} onClassifyApp={edit} onExport={vi.fn()}/>);
  expect(screen.getByRole('button', { name: 'League of Legends: 2min' })).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'League of Legends, 00:00, 2min' })).toBeInTheDocument();
  fireEvent.change(screen.getByRole('combobox', { name: 'Categoria de League of Legends' }), { target: { value: 'entertainment' } });
  expect(edit).toHaveBeenCalledWith('League of Legends', 'entertainment');
});

it('filters the timeline by category without removing raw records', () => {
  const report = emptyDay('2026-09-10');
  report.categories.push({ id: 'system', name: 'Sistema', distraction: false });
  report.sessions = [
    { id: 1, run_id: 'run', start_utc: 0, end_utc: 60000, duration_ms: 60000, local_date: '2026-09-10', offset_seconds: 0, source: 'vdi', hostname: null, app_name: null, reason: 'foreground', is_open: false },
    { id: 2, run_id: 'run', start_utc: 60000, end_utc: 120000, duration_ms: 60000, local_date: '2026-09-10', offset_seconds: 0, source: 'system', hostname: null, app_name: null, reason: 'foreground', is_open: false },
  ];
  render(<Dashboard report={report} onClassify={vi.fn()} onClassifyApp={vi.fn()} onExport={vi.fn()}/>);
  fireEvent.click(screen.getByRole('button', { name: 'Sistema' }));
  expect(screen.queryByRole('button', { name: 'VDI / trabalho focado, 00:00, 1min' })).not.toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Sistema, 00:01, 1min' })).toBeInTheDocument();
  expect(screen.getByText('Consultar 1 registro(s) bruto(s) em lista')).toBeInTheDocument();
});

function period(id: number, source: string, start: number, minutes = 1): Segment {
  return { id, source, start_utc: start, end_utc: start + minutes * 60_000, duration_ms: minutes * 60_000, local_date: '2026-09-11', offset_seconds: -10_800, run_id: 'run', hostname: null, app_name: null, reason: 'test', is_open: false };
}
const morning = Date.parse('2026-09-11T11:00:00Z');

it('shows unavailable boundaries on an empty day', () => {
  render(<Dashboard report={emptyDay('2026-09-11')} onClassify={vi.fn()} onClassifyApp={vi.fn()} onExport={vi.fn()}/>);
  expect(screen.getAllByText('Não disponível')).toHaveLength(3);
  expect(screen.getByText('Sem períodos registrados neste dia')).toBeInTheDocument();
  expect(screen.queryByRole('group', { name: 'Estados dos registros brutos' })).not.toBeInTheDocument();
});

it('shows first, last and largest gap with the offset recorded at each boundary', () => {
  const report = emptyDay('2026-09-11');
  report.sessions = [period(1, 'vdi', morning), period(2, 'app', morning + 120_000), { ...period(3, 'idle', morning + 360_000), offset_seconds: -7200 }];
  render(<Dashboard report={report} onClassify={vi.fn()} onClassifyApp={vi.fn()} onExport={vi.fn()}/>);
  const summary = screen.getByLabelText('Limites dos registros do dia');
  expect(within(summary).getByText('08:00')).toBeInTheDocument();
  expect(within(summary).getByText('09:07')).toBeInTheDocument();
  expect(within(summary).getByText('08:03–09:06 · 3min')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Estados' }));
  expect(screen.getByRole('img', { name: 'Lacuna não observada · 08:03–09:06 · 3min' })).toBeInTheDocument();
});

it.each([false, true])('shows one period without an internal gap (open: %s)', (is_open) => {
  const report = emptyDay('2026-09-11');
  report.sessions = [{ ...period(1, 'vdi', morning), is_open }];
  render(<Dashboard report={report} onClassify={vi.fn()} onClassifyApp={vi.fn()} onExport={vi.fn()}/>);
  const summary = screen.getByLabelText('Limites dos registros do dia');
  expect(within(summary).getByText('08:00')).toBeInTheDocument();
  expect(within(summary).getByText('08:01')).toBeInTheDocument();
  expect(within(summary).getByText('Sem lacuna interna')).toBeInTheDocument();
  expect(within(summary).queryByText('Em andamento · último instante confirmado') !== null).toBe(is_open);
});

it('shows confirmed boundaries even when the newly opened period has zero duration', () => {
  const report = emptyDay('2026-09-11');
  report.sessions = [{ ...period(1, 'vdi', morning, 0), is_open: true }];
  render(<Dashboard report={report} onClassify={vi.fn()} onClassifyApp={vi.fn()} onExport={vi.fn()}/>);
  const summary = screen.getByLabelText('Limites dos registros do dia');
  expect(within(summary).getAllByText('08:00')).toHaveLength(2);
  expect(within(summary).getByText('Em andamento · último instante confirmado')).toBeInTheDocument();
  expect(screen.getByText('Ainda não há duração confirmada neste dia')).toBeInTheDocument();
});

it('keeps a gap visible up to a newly opened zero-duration period', () => {
  const report = emptyDay('2026-09-11');
  report.sessions = [period(1, 'vdi', morning), { ...period(2, 'app', morning + 120_000, 0), is_open: true }];
  render(<Dashboard report={report} onClassify={vi.fn()} onClassifyApp={vi.fn()} onExport={vi.fn()}/>);
  fireEvent.click(screen.getByRole('button', { name: 'Estados' }));
  const gap = screen.getByRole('img', { name: 'Lacuna não observada · 08:01–08:02 · 1min' });
  expect(gap).toHaveStyle({ left: '50%', width: '50%' });
  expect(within(screen.getByLabelText('Limites dos registros do dia')).getByText('08:02')).toBeInTheDocument();
});

it('exposes all states, legend and full raw text for a long domain and a local application', () => {
  const report = emptyDay('2026-09-11');
  const sources = ['vdi', 'browser', 'app', 'system', 'idle', 'paused', 'unknown', 'unobserved'];
  const labels = ['VDI', 'Navegador', 'Aplicativo local', 'Sistema', 'Inatividade', 'Pausa', 'Desconhecido', 'Não observado'];
  report.sessions = sources.map((source, index) => period(index, source, morning + index * 60_000));
  const hostname = `${'dominio-longo.'.repeat(16)}example.com`;
  report.sessions[1].hostname = hostname;
  report.sessions[2].app_name = 'League of Legends';
  render(<Dashboard report={report} onClassify={vi.fn()} onClassifyApp={vi.fn()} onExport={vi.fn()}/>);
  fireEvent.click(screen.getByRole('button', { name: 'Estados' }));
  const legend = screen.getByRole('group', { name: 'Legenda dos estados' });
  const track = screen.getByRole('group', { name: 'Estados dos registros brutos' });
  expect(within(track).getAllByRole('button')).toHaveLength(8);
  for (const label of labels) {
    expect(within(legend).getByText(label)).toBeInTheDocument();
    expect(within(track).getByRole('button', { name: new RegExp(`^${label} ·`) })).toBeInTheDocument();
  }
  expect(within(track).getByRole('button', { name: new RegExp(hostname.replaceAll('.', '\\.')) })).toHaveAccessibleName(expect.stringContaining('Navegador'));
  fireEvent.click(screen.getByText('Consultar 8 registro(s) bruto(s) em lista'));
  expect(screen.getByRole('cell', { name: hostname })).toBeInTheDocument();
  expect(screen.getByRole('cell', { name: 'League of Legends' })).toBeInTheDocument();
  expect(screen.getByRole('cell', { name: 'Aplicativo local' })).toBeInTheDocument();
  expect(screen.getByText('Sem lacuna interna')).toBeInTheDocument();
});

it('preserves grouping, filters, totals and export while showing raw states once', () => {
  const report = emptyDay('2026-09-11');
  report.sessions = [period(1, 'vdi', morning), period(2, 'system', morning + 60_000, 4 / 60), period(3, 'vdi', morning + 64_000)];
  report.vdi_ms = 120_000;
  report.coverage_ms = 124_000;
  report.category_totals = [{ label: 'vdi', duration_ms: 120_000 }, { label: 'system', duration_ms: 4_000 }];
  const original = JSON.stringify(report);
  const onExport = vi.fn();
  render(<Dashboard report={report} onClassify={vi.fn()} onClassifyApp={vi.fn()} onExport={onExport}/>);
  const blocks = screen.getByRole('group', { name: 'Períodos do dia' });
  expect(within(blocks).getAllByRole('button')).toHaveLength(1);
  expect(screen.queryByRole('group', { name: 'Estados dos registros brutos' })).not.toBeInTheDocument();
  fireEvent.click(within(blocks).getByRole('button'));
  expect(screen.getByText('Inclui 1 interrupção(ões) curta(s).')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Estados' }));
  expect(screen.queryByRole('group', { name: 'Períodos do dia' })).not.toBeInTheDocument();
  const track = screen.getByRole('group', { name: 'Estados dos registros brutos' });
  expect(within(track).getAllByRole('button')).toHaveLength(3);
  expect(within(track).getAllByRole('button').reduce((sum, mark) => sum + parseFloat(mark.style.width), 0)).toBeCloseTo(100);
  fireEvent.click(within(track).getByRole('button', { name: /^Sistema ·/ }));
  expect(screen.getByRole('status')).toHaveTextContent('Sistema');
  fireEvent.click(screen.getByRole('button', { name: 'VDI / trabalho' }));
  expect(within(track).getAllByRole('button')).toHaveLength(2);
  expect(screen.getByText('Consultar 2 registro(s) bruto(s) em lista')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'VDI / trabalho: 2min' })).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'system: 4s' })).toBeInTheDocument();
  expect(screen.getByText('Cobertura medida')).toHaveTextContent('2min');
  fireEvent.click(screen.getByRole('button', { name: 'Tudo' }));
  expect(within(track).getAllByRole('button')).toHaveLength(3);
  fireEvent.click(screen.getByRole('button', { name: 'Categorias' }));
  expect(within(screen.getByRole('group', { name: 'Períodos do dia' })).getAllByRole('button')).toHaveLength(1);
  expect(screen.queryByRole('group', { name: 'Estados dos registros brutos' })).not.toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Exportar dia' }));
  expect(onExport).toHaveBeenCalledOnce();
  expect(JSON.stringify(report)).toBe(original);
});

it('does not turn periods hidden by category into gaps or change the day boundaries', () => {
  const report = emptyDay('2026-09-11');
  report.sessions = [period(1, 'vdi', morning), period(2, 'idle', morning + 60_000), period(3, 'vdi', morning + 180_000)];
  render(<Dashboard report={report} onClassify={vi.fn()} onClassifyApp={vi.fn()} onExport={vi.fn()}/>);
  fireEvent.click(screen.getByRole('button', { name: 'Estados' }));
  const summary = screen.getByLabelText('Limites dos registros do dia').textContent;
  expect(screen.getByRole('img', { name: /^Lacuna não observada/ })).toHaveAccessibleName('Lacuna não observada · 08:02–08:03 · 1min');
  fireEvent.click(screen.getByRole('button', { name: 'VDI / trabalho' }));
  expect(screen.queryByRole('img', { name: /^Lacuna não observada/ })).not.toBeInTheDocument();
  expect(screen.getByLabelText('Limites dos registros do dia')).toHaveTextContent(summary!);
});

it('renders a dense day without losing or duplicating raw states', () => {
  const report = emptyDay('2026-09-11');
  report.sessions = Array.from({ length: 500 }, (_, index) => period(index, index % 2 ? 'idle' : 'vdi', morning + index * 60_000));
  render(<Dashboard report={report} onClassify={vi.fn()} onClassifyApp={vi.fn()} onExport={vi.fn()}/>);
  fireEvent.click(screen.getByRole('button', { name: 'Estados' }));
  expect(within(screen.getByRole('group', { name: 'Estados dos registros brutos' })).getAllByRole('button')).toHaveLength(500);
  expect(screen.getByText('Consultar 500 registro(s) bruto(s) em lista')).toBeInTheDocument();
});
