import { useState } from 'react';
import { clock, duration, sourceLabel } from '../api';
import type { AppTotal, Bucket, DayReport, Segment } from '../types';

const categoryColors: Record<string, string> = { vdi: '#2457d6', local: '#4f73bd', social: '#9062b1', video: '#b67638', entertainment: '#9d687d', research: '#337e83', communication: '#5e809d', system: '#6e798b', unknown: '#7b8598', idle: '#a8b2c3', paused: '#d6dce6', unobserved: '#e9edf3' };
function categoryFor(segment: Segment, report: DayReport) { return segment.hostname ? report.domains.find(d => d.hostname === segment.hostname)?.category ?? 'unknown' : segment.source === 'app' ? report.app_totals.find(a => a.label === segment.app_name)?.category ?? 'local' : segment.source; }
function Bars({ data, labels = {}, onSelect }: { data: (Bucket | AppTotal)[]; labels?: Record<string, string>; onSelect?: (label: string) => void }) {
  const largest = Math.max(1, ...data.map(b => b.duration_ms));
  if (!data.length) return <p className="chart-empty">Os períodos registrados aparecerão aqui.</p>;
  return <div className="bar-chart">{data.map((b) => <button key={b.label} type="button" className="bar-row" onClick={() => onSelect?.(b.label)} disabled={!onSelect} aria-label={`${labels[b.label] ?? b.label}: ${duration(b.duration_ms)}`}>
    <span className="bar-label">{labels[b.label] ?? b.label}</span><strong>{duration(b.duration_ms)}</strong>
    <span className="bar-track"><span style={{ width: `${b.duration_ms / largest * 100}%`, background: categoryColors[b.label] ?? '#7593cc' }}/></span>
  </button>)}</div>;
}
export function Dashboard({ report, onClassify, onClassifyApp, onExport }: { report: DayReport; onClassify: (host: string, category: string) => void; onClassifyApp: (appName: string, category: string) => void; onExport: () => void }) {
  const [filter, setFilter] = useState('');
  const [selected, setSelected] = useState<Segment | null>(null);
  const labels = Object.fromEntries(report.categories.map(c => [c.id, c.name]));
  const periods = report.sessions.filter(s => s.duration_ms > 0);
  const domains = report.domains.filter(d => !filter || d.hostname.includes(filter.toLowerCase()) || d.category === filter);
  const start = periods.length ? Math.min(...periods.map(s => s.start_utc)) : 0;
  const end = periods.length ? Math.max(...periods.map(s => s.start_utc + s.duration_ms)) : 1;
  const span = Math.max(end - start, 1);
  const hasClockAdjustment = periods.some(s => s.reason === 'clock_adjusted');
  const metrics = [
    ['VDI / trabalho focado', duration(report.vdi_ms), 'Janela remota em primeiro plano'],
    ['Distrações', report.distraction_ms === null ? 'Não definidas' : duration(report.distraction_ms), 'Conforme as categorias escolhidas'],
    ['Maior bloco de foco', duration(report.longest_focus_ms), 'Período contínuo na VDI'],
    ['Trocas de contexto', String(report.context_switches), 'Entre origens ativas diferentes'],
  ];
  return <>
    <div className="summary-grid">{metrics.map(([label, value, help]) => <section className="summary" key={label}><h2>{label}</h2><strong>{value}</strong><p>{help}</p></section>)}</div>
    <div className="secondary-metrics"><span>Inativo <strong>{duration(report.idle_ms)}</strong></span><span>Desconhecido <strong>{duration(report.unknown_ms)}</strong></span><span>Maior distração <strong>{report.largest_distraction ?? '—'}</strong></span><span>Cobertura medida <strong>{duration(report.coverage_ms)}</strong></span></div>
    <section className="panel timeline-panel">
      <div className="panel-heading"><div><h2>Linha do tempo</h2><p>O dia, na ordem em que aconteceu.</p></div><span className="quiet-label">{periods.length} períodos</span></div>
      {!periods.length ? <div className="empty-state"><h3>Sem períodos registrados neste dia</h3><p>Quando a coleta estiver ativa, a sequência de janelas e sites aparecerá aqui.</p></div> : <>
        <div className="timeline-labels"><span>{clock(start, periods[0].offset_seconds)}</span><span>{clock(start + span / 4, periods[0].offset_seconds)}</span><span>{clock(start + span / 2, periods[0].offset_seconds)}</span><span>{clock(start + span * 3 / 4, periods[0].offset_seconds)}</span><span>{clock(end, periods.at(-1)!.offset_seconds)}</span></div>
        {hasClockAdjustment && <p className="hint">O relógio foi ajustado. Consulte a lista abaixo para ver a ordem observada e os horários de cada período.</p>}
        <div className="timeline" role="group" aria-label="Períodos do dia">
          {periods.map(s => <button key={s.id} className={`timeline-block ${s.source}`} style={{ left: `${(s.start_utc - start) / span * 100}%`, width: `${s.duration_ms / span * 100}%`, background: categoryColors[categoryFor(s, report)] ?? categoryColors.unknown }} onClick={() => { setSelected(s); if (s.hostname) setFilter(s.hostname); }} title={`${clock(s.start_utc, s.offset_seconds)}–${clock(s.end_utc, s.offset_seconds)} · ${sourceLabel(s.source, s.hostname, s.app_name)} · ${duration(s.duration_ms)}`} aria-label={`${sourceLabel(s.source, s.hostname, s.app_name)}, ${clock(s.start_utc, s.offset_seconds)}, ${duration(s.duration_ms)}`} />)}
        </div>
        <div className="timeline-legend">{[...new Set(periods.map(s => categoryFor(s, report)))].map(c => <span key={c}><i style={{ background: categoryColors[c] ?? categoryColors.unknown }}/>{labels[c] ?? sourceLabel(c)}</span>)}</div>
        <p className="hint">Faixas sem observação não contam como foco. Tempo em foco não é uma avaliação de produtividade.</p>
        {selected && <div className="selection" role="status"><strong>{sourceLabel(selected.source, selected.hostname, selected.app_name)}</strong><span>{clock(selected.start_utc, selected.offset_seconds)}–{clock(selected.end_utc, selected.offset_seconds)} · {duration(selected.duration_ms)}</span><button className="text-button" onClick={() => setSelected(null)}>Fechar detalhe</button></div>}
        <details className="period-list"><summary>Consultar todos os períodos em lista</summary><div className="table-scroll"><table><thead><tr><th>Horário</th><th>Origem</th><th>Duração</th></tr></thead><tbody>{periods.map(s => <tr key={s.id}><td>{clock(s.start_utc, s.offset_seconds)}–{clock(s.end_utc, s.offset_seconds)}</td><td>{sourceLabel(s.source, s.hostname, s.app_name)}{s.reason === 'clock_adjusted' && ' · relógio ajustado'}</td><td>{duration(s.duration_ms)}</td></tr>)}</tbody></table></div></details>
      </>}
    </section>
    <div className="charts-grid"><section className="panel"><div className="panel-heading"><div><h2>Tempo por categoria</h2><p>Tempo ativo observado. Inatividade à parte.</p></div></div><Bars data={report.category_totals} labels={labels} onSelect={setFilter}/></section><section className="panel"><div className="panel-heading"><div><h2>Tempo por aplicativo</h2><p>Os cinco aplicativos locais com mais tempo em foco.</p></div></div><Bars data={report.app_totals.slice(0, 5)}/></section><section className="panel"><div className="panel-heading"><div><h2>Tempo por domínio</h2><p>Os cinco sites com mais tempo em foco.</p></div></div><Bars data={report.domains.slice(0, 5).map(d => ({ label: d.hostname, duration_ms: d.duration_ms }))} onSelect={setFilter}/></section></div>
    <div className="charts-grid"><section className="panel"><div className="panel-heading"><div><h2>Distribuição por horário</h2><p>Cobertura medida, incluindo inatividade.</p></div></div><div className="hour-chart">{report.hourly.length ? report.hourly.map(b => <div key={b.label} title={`${b.label}: ${duration(b.duration_ms)}`}><strong>{duration(b.duration_ms)}</strong><span style={{ height: `${Math.max(2, b.duration_ms / Math.max(...report.hourly.map(h => h.duration_ms)) * 70)}px` }}/><small>{b.label}</small></div>) : <p className="chart-empty">Ainda não há distribuição neste dia.</p>}</div></section><section className="panel"><div className="panel-heading"><div><h2>VDI e distrações</h2><p>Valores absolutos do dia selecionado.</p></div></div>{report.distraction_ms === null ? <p className="chart-empty">Escolha em Configurações quais categorias entram como distrações. Nenhuma é marcada automaticamente.</p> : <Bars data={[{ label: 'vdi', duration_ms: report.vdi_ms }, { label: 'distractions', duration_ms: report.distraction_ms }]} labels={{ vdi: 'VDI / trabalho focado', distractions: 'Distrações selecionadas' }}/>}<p className="hint">Outros períodos ativos: {duration(report.coverage_ms - report.idle_ms - report.vdi_ms - (report.distraction_ms ?? 0))}.</p></section></div>
    <section className="panel domains-panel"><div className="panel-heading"><div><h2>Sites e categorias</h2><p>A categoria escolhida também será aplicada aos dias anteriores.</p></div><button className="secondary-button" onClick={onExport}>Exportar dia</button></div>
      <div className="table-tools"><label>Filtrar domínios<input type="search" value={filter} onChange={e => setFilter(e.target.value)} placeholder="Domínio ou categoria"/></label>{filter && <button className="text-button" onClick={() => setFilter('')}>Limpar filtro</button>}<span>{domains.length} domínios</span></div>
      <div className="table-scroll"><table><thead><tr><th>Domínio</th><th>Categoria</th><th className="numeric">Tempo ativo</th><th className="numeric">Acessos</th></tr></thead><tbody>{domains.map(d => <tr key={d.hostname}><td className="domain-name">{d.hostname}</td><td><select aria-label={`Categoria de ${d.hostname}`} value={d.category} onChange={e => onClassify(d.hostname, e.target.value)}>{report.categories.filter(c => c.id !== 'vdi').map(c => <option key={c.id} value={c.id}>{c.name}</option>)}</select></td><td className="numeric">{duration(d.duration_ms)}</td><td className="numeric">{d.accesses}</td></tr>)}</tbody></table>{!domains.length && <p className="table-empty">{filter ? 'Nenhum domínio corresponde ao filtro.' : 'Nenhum domínio registrado neste dia.'}</p>}</div>
      <p className="hint">Desconhecido: {duration(report.unclassified_ms)} em sites sem categoria e {duration(report.unidentified_ms)} sem origem identificada. Acessos contam entradas em abas em foco, não carregamentos de página.</p>
    </section>
    <section className="panel domains-panel"><div className="panel-heading"><div><h2>Aplicativos locais e categorias</h2><p>A categoria escolhida também será aplicada aos dias anteriores.</p></div></div>
      <div className="table-scroll"><table><thead><tr><th>Aplicativo</th><th>Categoria</th><th className="numeric">Tempo em foco</th></tr></thead><tbody>{report.app_totals.map(a => <tr key={a.label}><td className="domain-name">{a.label}</td><td><select aria-label={`Categoria de ${a.label}`} value={a.category} onChange={e => onClassifyApp(a.label, e.target.value)}>{report.categories.filter(c => c.id !== 'vdi').map(c => <option key={c.id} value={c.id}>{c.name}</option>)}</select></td><td className="numeric">{duration(a.duration_ms)}</td></tr>)}</tbody></table>{!report.app_totals.length && <p className="table-empty">Nenhum aplicativo local registrado neste dia.</p>}</div>
    </section>
  </>;
}
