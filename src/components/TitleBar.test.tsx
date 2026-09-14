import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { TitleBar } from './TitleBar';

const native = vi.hoisted(() => ({
  isTauri: vi.fn(), minimize: vi.fn(), toggleMaximize: vi.fn(), close: vi.fn(),
  isMaximized: vi.fn(), onResized: vi.fn(), unlisten: vi.fn(),
}));
vi.mock('@tauri-apps/api/core', () => ({ isTauri: native.isTauri }));
vi.mock('@tauri-apps/api/window', () => ({ getCurrentWindow: () => native }));
beforeEach(() => {
  native.isTauri.mockReturnValue(true);
  native.isMaximized.mockResolvedValue(false);
  native.minimize.mockResolvedValue(undefined);
  native.toggleMaximize.mockResolvedValue(undefined);
  native.close.mockResolvedValue(undefined);
  native.onResized.mockResolvedValue(native.unlisten);
});
afterEach(() => { cleanup(); vi.resetAllMocks(); });

it('minimizes and requests normal close, preserving the native tray handler', async () => {
  render(<TitleBar/>);
  fireEvent.click(screen.getByRole('button', { name: 'Minimizar janela' }));
  fireEvent.click(screen.getByRole('button', { name: 'Fechar janela' }));
  await waitFor(() => expect(native.minimize).toHaveBeenCalledOnce());
  expect(native.close).toHaveBeenCalledOnce();
  expect(screen.getByRole('button', { name: 'Fechar janela' })).toHaveAttribute('title', 'Fechar janela · continuar na bandeja');
});

it('updates maximize and restore controls using the actual window state', async () => {
  render(<TitleBar/>);
  await waitFor(() => expect(native.isMaximized).toHaveBeenCalled());
  native.isMaximized.mockResolvedValue(true);
  fireEvent.click(screen.getByRole('button', { name: 'Maximizar janela' }));
  expect(await screen.findByRole('button', { name: 'Restaurar janela' })).toBeInTheDocument();
  expect(native.toggleMaximize).toHaveBeenCalledOnce();
  native.isMaximized.mockResolvedValue(false);
  fireEvent.click(screen.getByRole('button', { name: 'Restaurar janela' }));
  expect(await screen.findByRole('button', { name: 'Maximizar janela' })).toBeInTheDocument();
});

it('tracks resizing outside the controls and removes its listener on unmount', async () => {
  const { unmount } = render(<TitleBar/>);
  await waitFor(() => expect(native.onResized).toHaveBeenCalledOnce());
  native.isMaximized.mockResolvedValue(true);
  await act(async () => { native.onResized.mock.calls[0][0](); });
  expect(screen.getByRole('button', { name: 'Restaurar janela' })).toBeInTheDocument();
  unmount();
  expect(native.unlisten).toHaveBeenCalledOnce();
});

it('cleans up a listener that resolves after the titlebar unmounts', async () => {
  let resolve!: (stop: () => void) => void;
  native.onResized.mockReturnValue(new Promise<() => void>(done => { resolve = done; }));
  const { unmount } = render(<TitleBar/>);
  unmount();
  await act(async () => { resolve(native.unlisten); });
  expect(native.unlisten).toHaveBeenCalledOnce();
});

it('reports a rejected window action and permits retry', async () => {
  native.minimize.mockRejectedValueOnce(new Error('denied'));
  render(<TitleBar/>);
  fireEvent.click(screen.getByRole('button', { name: 'Minimizar janela' }));
  expect(await screen.findByRole('alert')).toHaveTextContent('Não foi possível controlar a janela. Tente novamente.');
  fireEvent.click(screen.getByRole('button', { name: 'Minimizar janela' }));
  await waitFor(() => expect(screen.queryByRole('alert')).not.toBeInTheDocument());
});

it('disables native actions in a browser-only preview', () => {
  native.isTauri.mockReturnValue(false);
  render(<TitleBar/>);
  for (const button of screen.getAllByRole('button')) expect(button).toBeDisabled();
  expect(native.onResized).not.toHaveBeenCalled();
});
