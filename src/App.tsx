import { useEffect, useState } from 'react';
import { save } from '@tauri-apps/plugin-dialog';
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';
import { call, today, sourceLabel } from './api';
import { Dashboard } from './components/Dashboard';
import { Icon } from './components/Icon';
import type { DayReport, Status } from './types';
import './styles.css';

export default function App() {
  const [view, setView] = useState<'day' | 'history' | 'settings'>('day');
  const [date, setDate] = useState(today);
  const [status, setStatus] = useState<Status | null>(null);
  const [report, setReport] = useState<DayReport | null>(null);
  const [error, setError] = useState('');
  const [notice, setNotice] = useState('');
  const [busy, setBusy] = useState(false);
  const [revision, setRevision] = useState(0);
  const [capture, setCapture] = useState(false);
  const [countdown, setCountdown] = useState(0);
  const [deleteScope, setDeleteScope] = useState<'day' | 'all' | null>(null);
  const [autostart, setAutostart] = useState<boolean | null>(null);

  useEffect(() => {
    let cancelled = false;
    let pending = false;
    async function refresh() {
      if (pending) return;
      pending = true;
      try {
        const nextStatus = await call<Status>('get_status');
        if (!cancelled) setStatus(nextStatus);
        const nextReport = await call<DayReport>('get_day', { day: date });
        if (!cancelled) { setReport(nextReport); setError(''); }
      } catch (e) { if (!cancelled) setError(String(e instanceof Error ? e.message : e)); }
      finally { pending = false; }
    }
    void refresh();
    const timer = setInterval(() => { void refresh(); }, 2000);
    return () => { cancelled = true; clearInterval(timer); };
  }, [date, revision]);
  useEffect(() => {
    if (view !== 'day') return;
    const timer = setInterval(() => setDate(today()), 1000);
    return () => clearInterval(timer);
  }, [view]);
  useEffect(() => { if ('__TAURI_INTERNALS__' in window) void isEnabled().then(setAutostart).catch(() => setAutostart(null)); }, []);
  useEffect(() => { if (!countdown) return; const timer = setTimeout(() => setCountdown(v => v - 1), 1000); return () => clearTimeout(timer); }, [countdown]);

  async function act(command: string, args?: Record<string, unknown>, success?: string) {
    setBusy(true); setNotice('');
    try { await call(command, args); if (success) setNotice(success); setRevision(v => v + 1); return true; }
    catch (e) { setNotice(`Não foi possível concluir: ${String(e)}`); return false; }
    finally { setBusy(false); }
  }
  async function identify() {
    setCapture(true); setCountdown(5);
    await act('capture_vdi', undefined, 'Janela da sessão remota identificada. Você já pode iniciar a coleta.');
    setCapture(false); setCountdown(0);
  }
  async function exportDay() {
    try {
      const path = await save({ defaultPath: `caixa-preta-${date}.json`, filters: [{ name: 'JSON', extensions: ['json'] }] });
      if (path) await act('export_day', { day: date, path }, 'Dia exportado para o arquivo selecionado.');
    } catch (e) { setNotice(`Exportação não concluída: ${String(e)}`); }
  }
  function go(next: typeof view) { setView(next); if (next === 'day') setDate(today()); }
  const stateLabel = !status || status.error ? 'Indisponível' : status.settings.paused ? 'Pausado' : 'Ativo';
  const humanDate = new Date(`${date}T12:00:00`).toLocaleDateString('pt-BR', { weekday: 'long', day: 'numeric', month: 'long' });
  const chooseVdi = <div className="setting-row"><div><h3>Janela da VDI</h3><p>{status?.settings.vdi ? 'Windows App · sessão remota identificada' : 'Abra sua sessão remota no Windows App.'}</p><p>Ao clicar, você terá 5 segundos para colocar a janela remota em foco.</p>{status?.settings.vdi && <span className="connection-state">{status.vdi_in_focus ? 'VDI em foco agora' : 'VDI fora de foco agora'}</span>}</div><button className="secondary-button" disabled={!status || busy} onClick={() => { void identify(); }}><Icon name="monitor"/>{capture ? `Identificando em ${countdown}s…` : status?.settings.vdi ? 'Identificar novamente' : 'Identificar minha VDI'}</button></div>;

  return <div className="app-shell">
    <aside className="sidebar"><a href="#main" className="skip-link">Ir para conteúdo</a><div className="brand"><div className="brand-mark"><span/><span/><span/></div><span>Caixa Preta<strong>do Dia</strong></span></div><nav aria-label="Navegação principal">{([['day', 'Hoje', 'day'], ['history', 'Histórico', 'history'], ['settings', 'Configurações', 'settings']] as const).map(([id, label, icon]) => <button key={id} className={view === id ? 'nav-button selected' : 'nav-button'} aria-current={view === id ? 'page' : undefined} onClick={() => go(id)}><Icon name={icon}/>{label}</button>)}</nav><div className="sidebar-bottom"><span className="local-dot"/>Dados neste computador<p>Sem nuvem. Sem avaliação<br/>de produtividade.</p></div></aside>
    <main id="main" className="main-content"><header className="page-header"><div><h1>{view === 'settings' ? 'Configurações' : view === 'history' ? 'Histórico do dia' : 'Seu dia, em perspectiva'}</h1><p>{view === 'settings' ? 'Controle o que é observado e como aparece para você.' : humanDate}</p></div><div className="header-actions"><span className={`tracking-status ${stateLabel === 'Ativo' ? 'active' : ''}`} role="status"><i/>{stateLabel}</span><button className="primary-button" disabled={!status || busy || !!status.error || !status.settings.vdi} onClick={() => { void act('set_paused', { paused: !status?.settings.paused }); }}><Icon name={status?.settings.paused ? 'play' : 'pause'} size={17}/>{status?.settings.paused ? 'Iniciar / retomar' : 'Pausar'}</button></div></header>
      {error && <div className="message error" role="alert">{error}</div>}
      {status?.error && <div className="message error" role="alert">{status.error}</div>}
      {status?.warning && <div className="message warning" role="status">{status.warning}</div>}
      {notice && <div className="message" role="status">{notice}<button className="text-button" onClick={() => setNotice('')}>Dispensar</button></div>}
      {status && !status.error && view !== 'settings' && !status.settings.configured && <section className="panel onboarding"><h2>Prepare a primeira observação</h2><p>O rastreamento começa somente quando você iniciar. Serão registrados foco da VDI, domínio da aba ativa e inatividade. Conteúdo das páginas, teclas e telas não são capturados.</p>{chooseVdi}<div className="setting-row"><div><h3>Extensão do Google Chrome</h3><p>{status.extension_connected ? 'Extensão conectada ao aplicativo local.' : 'Instale a extensão local e registre a ponte conforme o README do projeto.'}</p><p>Você pode começar pela VDI. Sem a extensão, o navegador fica desconhecido.</p></div><span className="connection-state">{status.extension_connected ? 'Conectada' : 'Não conectada'}</span></div><p className="hint">Ao fechar esta janela, a coleta continua na bandeja. Use “Sair e encerrar coleta” para encerrar o aplicativo.</p></section>}
      {view === 'history' && <div className="date-toolbar"><label>Consultar dia<input aria-label="Consultar dia" type="date" value={date} max={today()} onChange={e => { if (e.target.value) { setReport(null); setDate(e.target.value); } }}/></label><button className="secondary-button" onClick={() => go('day')}>Voltar para hoje</button></div>}
      {view !== 'settings' ? report?.date === date ? <Dashboard key={date} report={report} onClassify={(hostname, category) => { void act('classify_domain', { hostname, category }); }} onClassifyApp={(appName, category) => { void act('classify_app', { appName, category }); }} onExport={() => { void exportDay(); }}/> : !error && <div className="loading-state" role="status">Carregando os registros locais…</div> : <>
        <section className="panel settings-panel"><h2>Rastreamento</h2>{chooseVdi}<div className="setting-row"><div><h3>Inatividade</h3><p>Depois do limite sem teclado ou mouse, o tempo passa a ser inatividade. Leitura ou vídeo sem interação também pode atingir esse limite.</p></div><label className="inline-label">Após<select aria-label="Minutos para inatividade" disabled={!status || busy} value={status?.settings.idle_minutes ?? 5} onChange={e => { void act('set_idle_minutes', { minutes: Number(e.target.value) }); }}>{[1, 2, 5, 10, 15, 30, 60].map(m => <option key={m} value={m}>{m} minutos</option>)}</select></label></div><div className="setting-row"><div><h3>Iniciar com o Windows</h3><p>Abre no login do usuário, respeitando a pausa salva.</p></div><label className="checkbox-label"><input type="checkbox" disabled={autostart === null || busy} checked={autostart ?? false} onChange={async e => { const next = e.target.checked; try { if (next) await enable(); else await disable(); setAutostart(next); } catch (e) { setNotice(`Não foi possível alterar a inicialização: ${String(e)}`); } }}/>{autostart === null ? 'Indisponível nesta execução' : 'Iniciar automaticamente'}</label></div></section>
        <section className="panel settings-panel"><h2>O que entra como distração</h2><p>Você decide. Nenhuma categoria é marcada automaticamente. A escolha também atualiza os resumos anteriores.</p><div className="category-options">{report?.categories.filter(c => c.id !== 'vdi').map(c => <label key={c.id} className="checkbox-label"><input type="checkbox" checked={c.distraction} disabled={busy} onChange={e => { void act('set_distraction', { category: c.id, enabled: e.target.checked }); }}/>{c.name}</label>)}</div></section>
        <section className="panel settings-panel"><h2>Seus dados</h2><p>Armazenados localmente. Exportações são cópias independentes e não são apagadas pelo aplicativo.</p><div className="data-path">{status?.data_path ?? 'Banco local indisponível'}</div><div className="data-actions"><button className="secondary-button" disabled={!report || busy} onClick={() => { void exportDay(); }}><Icon name="download" size={17}/>Exportar {date}</button><button className="secondary-button danger-text" disabled={!status || busy} onClick={() => setDeleteScope('day')}>Apagar {date}</button><button className="secondary-button danger-text" disabled={!status || busy} onClick={() => setDeleteScope('all')}>Apagar toda atividade</button></div></section>
      </>}
      <footer className="page-footer"><span>{status && !status.settings.paused && !status.error ? `Agora: ${sourceLabel(status.activity.source, status.activity.hostname, status.activity.app_name)}` : 'Você controla quando observar.'}</span><span>{status?.last_saved_utc ? `Última gravação ${new Date(status.last_saved_utc).toLocaleTimeString('pt-BR')}` : 'Nenhuma atividade gravada'}</span></footer>
    </main>
    {deleteScope && (
      <DeleteConfirmation scope={deleteScope} date={date} busy={busy} onCancel={() => setDeleteScope(null)} onConfirm={async () => { if (await act('delete_data', { day: deleteScope === 'day' ? date : null }, 'Atividade apagada. O rastreamento ficou pausado; categorias e configuração foram preservadas.')) { setDeleteScope(null); setReport(null); } }}/>
    )}
  </div>;
}

function DeleteConfirmation({ scope, date, busy, onCancel, onConfirm }: { scope: 'day' | 'all'; date: string; busy: boolean; onCancel: () => void; onConfirm: () => Promise<void> }) {
  useEffect(() => {
    const dialog = document.getElementById('delete-confirmation') as HTMLDialogElement;
    dialog.showModal();
    return () => dialog.close();
  }, []);
  return <dialog id="delete-confirmation" aria-labelledby="delete-title" onCancel={e => { e.preventDefault(); if (!busy) onCancel(); }}><h2 id="delete-title">{scope === 'all' ? 'Apagar toda a atividade?' : `Apagar os registros de ${date}?`}</h2><p>Essa ação não pode ser desfeita. O rastreamento será pausado. Categorias, configuração e arquivos exportados serão preservados.</p><div className="dialog-actions"><button autoFocus className="secondary-button" disabled={busy} onClick={onCancel}>Cancelar</button><button className="danger-button" disabled={busy} onClick={() => { void onConfirm(); }}>Apagar registros</button></div></dialog>;
}
