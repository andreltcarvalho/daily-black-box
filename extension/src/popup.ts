async function refresh(kind: string) {
  const label = document.getElementById('status')!;
  try {
    const status = await chrome.runtime.sendMessage({ kind });
    label.textContent = status.connected ? status.paused ? 'Conectado · rastreamento pausado' : 'Conectado · rastreamento ativo' : 'Desconectado. Abra o aplicativo e confira a instalação da ponte local.';
  } catch { label.textContent = 'Extensão indisponível. Recarregue a extensão no Chrome.'; }
}
document.getElementById('reconnect')!.addEventListener('click', () => { void refresh('reconnect'); });
void refresh('status');
