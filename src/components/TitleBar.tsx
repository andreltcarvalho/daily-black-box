import { useEffect, useState } from 'react';
import { isTauri } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { Icon } from './Icon';

export function TitleBar() {
  const native = isTauri();
  const [maximized, setMaximized] = useState(false);
  const [error, setError] = useState('');

  useEffect(() => {
    if (!native) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    const appWindow = getCurrentWindow();
    const update = async () => {
      try { const value = await appWindow.isMaximized(); if (!disposed) setMaximized(value); }
      catch { if (!disposed) setError('Não foi possível consultar o estado da janela.'); }
    };
    void update();
    const listener = appWindow.onResized(() => { void update(); });
    void listener.then(stop => { if (disposed) stop(); else unlisten = stop; }).catch(() => {
      if (!disposed) setError('Não foi possível acompanhar o tamanho da janela.');
    });
    return () => { disposed = true; unlisten?.(); };
  }, [native]);

  async function control(action: 'minimize' | 'toggleMaximize' | 'close') {
    if (!native) return;
    try {
      setError('');
      const appWindow = getCurrentWindow();
      await appWindow[action]();
      if (action === 'toggleMaximize') setMaximized(await appWindow.isMaximized());
    } catch { setError('Não foi possível controlar a janela. Tente novamente.'); }
  }

  return <header className="titlebar">
    <div className="titlebar-drag" data-tauri-drag-region>
      <div className="app-emblem" aria-hidden="true"><span/><span/><span/></div>
      <span className="app-title">Caixa Preta <span>do Dia</span></span>
    </div>
    {error && <span className="window-error" role="alert">{error}</span>}
    <div className="window-controls" role="group" aria-label="Controles da janela">
      <button type="button" disabled={!native} aria-label="Minimizar janela" title="Minimizar" onClick={() => { void control('minimize'); }}><Icon name="minimize" size={16}/></button>
      <button type="button" disabled={!native} aria-label={maximized ? 'Restaurar janela' : 'Maximizar janela'} title={maximized ? 'Restaurar' : 'Maximizar'} onClick={() => { void control('toggleMaximize'); }}><Icon name={maximized ? 'restore' : 'maximize'} size={16}/></button>
      <button type="button" className="window-close" disabled={!native} aria-label="Fechar janela" title="Fechar janela · continuar na bandeja" onClick={() => { void control('close'); }}><Icon name="close" size={17}/></button>
    </div>
  </header>;
}
