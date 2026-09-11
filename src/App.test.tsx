import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import App from './App';
import { emptyDay, statusFixture } from './test-fixtures';
import { today } from './api';

const api = vi.hoisted(() => ({ call: vi.fn() }));
vi.mock('./api', async importOriginal => ({ ...(await importOriginal<typeof import('./api')>()), call: api.call }));
vi.mock('@tauri-apps/plugin-autostart', () => ({ enable: vi.fn(), disable: vi.fn(), isEnabled: vi.fn().mockResolvedValue(false) }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });
beforeEach(() => { api.call.mockImplementation(async (command, args) => command === 'get_status' ? structuredClone(statusFixture) : command === 'get_day' ? emptyDay(args.day) : undefined); });
describe('daily dashboard', () => {
  it('opens today and presents no records rather than fabricated activity', async () => {
    render(<App/>);
    expect(await screen.findByText('Sem períodos registrados neste dia')).toBeInTheDocument();
    expect(api.call).toHaveBeenCalledWith('get_day', { day: today() });
    expect(screen.getByText('Não definidas')).toBeInTheDocument();
    expect(screen.queryByText('youtube.com')).not.toBeInTheDocument();
  });
  it('switches the complete report when selecting a historical day', async () => {
    render(<App/>); await screen.findByText('Sem períodos registrados neste dia');
    fireEvent.click(screen.getByRole('button', { name: 'Histórico' }));
    fireEvent.change(screen.getByLabelText('Consultar dia'), { target: { value: '2026-08-01' } });
    await waitFor(() => expect(api.call).toHaveBeenCalledWith('get_day', { day: '2026-08-01' }));
  });
  it('sends the explicit pause transition and exposes a failed command', async () => {
    api.call.mockImplementation(async (command, args) => {
      if (command === 'get_status') return { ...statusFixture, settings: { ...statusFixture.settings, paused: false } };
      if (command === 'get_day') return emptyDay(args.day);
      throw new Error('disco indisponível');
    });
    render(<App/>); await screen.findByText('Sem períodos registrados neste dia');
    fireEvent.click(screen.getByRole('button', { name: 'Pausar' }));
    expect(await screen.findByText(/disco indisponível/)).toBeInTheDocument();
    expect(api.call).toHaveBeenCalledWith('set_paused', { paused: true });
  });
  it('leaves distraction categories unchecked until chosen by the user', async () => {
    render(<App/>); await screen.findByText('Sem períodos registrados neste dia');
    fireEvent.click(screen.getByRole('button', { name: 'Configurações' }));
    const checkbox = screen.getByRole('checkbox', { name: 'Vídeo' }); expect(checkbox).not.toBeChecked();
    fireEvent.click(checkbox);
    await waitFor(() => expect(api.call).toHaveBeenCalledWith('set_distraction', { category: 'video', enabled: true }));
  });
});
