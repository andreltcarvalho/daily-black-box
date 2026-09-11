import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import { Dashboard } from './Dashboard';
import { emptyDay } from '../test-fixtures';
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
