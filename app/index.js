// Memory Sentry Custom App. Source and default locale are English; Portuguese is selected automatically.
// The installer replaces __MSS_TOKEN__ with a random per-install local API token.
const MSSReact = Spicetify.React;
const h = MSSReact.createElement;
const MSS_API = 'http://127.0.0.1:38472';
const MSS_TOKEN = '__MSS_TOKEN__';
const MSS_GITHUB_URL = 'https://github.com/yanlvl99/spicetify-memory-sentry';
const MSS_RELEASES_URL = `${MSS_GITHUB_URL}/releases`;
const MSS_INSTALL_GUIDE_URL = `${MSS_GITHUB_URL}/blob/main/docs/INSTALLATION.md`;
const MSS_LOCALE = (typeof navigator !== 'undefined' && /^pt(?:-|$)/i.test(navigator.language || '')) ? 'pt-BR' : 'en-US';
const MSS_PT = MSS_LOCALE === 'pt-BR';

const MSS_TEXT = {
  en: {
    kicker: 'SPICETIFY · WINDOWS MEMORY TOOL · RELEASE CANDIDATE v1.0.0-rc.1', title: 'Memory Sentry',
    intro: 'Adaptive memory management for Spotify Desktop. Supported operating systems: Windows 10 and Windows 11 only. When Spotify is minimized or hidden, the Windows companion adjusts per-process working-set limits based on sustained CPU activity and available memory headroom. Original limits are restored when the window returns.',
    connected: 'Companion connected', unavailable: 'Connection unavailable', waiting: 'Waiting for companion',
    helperMissingTitle: 'Companion is not connected',
    helperMissingBody: 'The Marketplace listing is for discovery only. Use the official Setup from GitHub Releases to install or repair the native Windows companion. There is no separate Marketplace installer.',
    helperButton: 'Download official setup', installGuideButton: 'Installation guide', safety: 'On supported Windows 10/11 x64 PCs, optimization is intended to substantially reduce Spotify resident RAM while minimized or hidden; actual savings vary by PC and workload. Audio-service processes are excluded. Very low-end PCs may briefly show a black window on restore; that alone does not prove a reload if the previous UI state returns. Input may take a few seconds to respond.',
    spotifyRam: 'Spotify resident memory', ramNote: '{count} Spotify processes detected. Summing working sets can double-count shared pages.',
    optimizedRam: 'Managed-process memory', managedNote: '{count} non-audio processes are currently eligible for optimization.',
    activeLimits: 'Adaptive limits active', activeNote: 'Per-process working-set ceilings managed automatically.',
    pageFaults: 'Page faults since last sample', pageFaultNote: 'Includes soft and hard faults; not equivalent to disk reads.',
    pageFaultRate: 'Page faults per second', pageFaultRateNote: 'A local diagnostic signal, not proof of disk I/O by itself.',
    restoreDuration: 'Limit restoration duration', restoreNote: 'Measures the helper operation, not full UI recovery time.',
    adaptiveChanges: 'Adaptive adjustments', adaptiveNote: 'Changes made by the automatic per-process policy.',
    processes: 'Monitored processes', updated: 'Updated {time}', cpu: 'CPU {value}%', privateMemory: 'Private {value} MB',
    audioProtected: 'Audio protected · no limit applied', adaptiveAt: 'Adaptive ceiling: {value} MB',
    willOptimize: 'Ready for automatic optimization', notIncluded: 'Outside the current process tree',
    reasonHigh: 'raised for sustained CPU activity', reasonLow: 'lowered because CPU and memory usage were low', reasonBaseline: 'using role baseline', reasonHolding: 'holding current ceiling',
    noneDetected: 'No eligible processes detected. Open Spotify and wait for process discovery. Memory Sentry will not guess at unknown process identities.',
    restoreNow: 'Restore original limits now', restoring: 'Restoring…', refresh: 'Refresh data', lastAdjustment: 'Last adjustment: {time}',
    lastAction: 'Last automatic action: adjusted {count} process(es). Audio-service processes were excluded.',
    behavior: 'Automatic policy', active: 'Enable automatic optimization', autoExplanation: 'Optimize only after Spotify has remained minimized or hidden for the selected delay. Original native limits are restored when it becomes visible.',
    delay: 'Delay after minimize/hide', delayExplanation: 'Avoids optimizing during brief minimizations. Process limits adapt automatically; there is no fixed user-set RAM cap.',
    seconds: '{value} seconds', save: 'Save settings', saving: 'Saving…', savedLocal: 'Settings are stored locally on this PC.',
    events: 'Recent activity', noEvents: 'No activity recorded yet.', statusError: 'Could not connect to the local Rust companion. The diagnostic appears in brackets; check startup.log in %LOCALAPPDATA%\\SpicetifyMemorySentry\\logs.',
    footer: 'Supported OS: Windows 10/11, Linux, macOS (x64 & ARM). The native Rust companion is open source under GPL-3.0. No telemetry is uploaded. CPU and memory diagnostics stay on this PC.',
    confirmRestore: 'Restore the original Windows working-set limits for Spotify processes now? If Spotify is still hidden, automatic optimization may apply adaptive limits again after the configured delay.',
    restoreFailed: 'Could not restore the original limits.', saveFailed: 'Could not save settings.', helperApiError: 'The local companion returned an error.',
    window: { foreground: 'Spotify in foreground', 'background-visible': 'Spotify visible in background', minimized: 'Spotify minimized', hidden: 'Spotify window hidden', 'not-running': 'Spotify is not running', 'window-not-found': 'Spotify found; window not located', 'window-check-error': 'Window state unavailable', 'not-detected': 'Waiting for detection' }
  },
  pt: {
    kicker: 'SPICETIFY · FERRAMENTA DE RAM PARA WINDOWS · RELEASE CANDIDATE v1.0.0-rc.1', title: 'Memory Sentry',
    intro: 'Gerenciamento adaptativo de memória para o Spotify Desktop. Sistemas compatíveis: somente Windows 10 e Windows 11. Quando o Spotify fica minimizado ou oculto, o auxiliar do Windows ajusta os limites de working set por processo conforme a atividade sustentada de CPU e a folga de memória. Os limites originais são restaurados quando a janela volta.',
    connected: 'Auxiliar conectado', unavailable: 'Conexão indisponível', waiting: 'Aguardando auxiliar',
    helperMissingTitle: 'O auxiliar nativo não está conectado',
    helperMissingBody: 'A listagem do Marketplace serve apenas para descoberta. Use o instalador oficial em GitHub Releases para instalar ou reparar o auxiliar nativo do Windows. Não existe um instalador separado do Marketplace.',
    helperButton: 'Baixar instalador oficial', installGuideButton: 'Guia de instalação', safety: 'Em PCs compatíveis com Windows 10/11 x64, a otimização busca reduzir bastante a RAM residente do Spotify enquanto minimizado ou oculto; a economia varia conforme o PC e a carga. Os processos de áudio ficam excluídos. Em PCs muito fracos, a janela pode ficar preta por alguns instantes ao restaurar; isso não prova uma recarga se a interface volta ao estado anterior. A entrada pode demorar alguns segundos para responder.',
    spotifyRam: 'Memória residente do Spotify', ramNote: '{count} processos do Spotify detectados. A soma dos working sets pode contar páginas compartilhadas mais de uma vez.',
    optimizedRam: 'Memória dos processos gerenciados', managedNote: '{count} processos sem relação com áudio estão elegíveis para otimização.',
    activeLimits: 'Limites adaptativos ativos', activeNote: 'Limites de working set por processo, administrados automaticamente.',
    pageFaults: 'Falhas de página desde a amostra', pageFaultNote: 'Inclui falhas suaves e duras; não equivale a leituras de disco.',
    pageFaultRate: 'Falhas de página por segundo', pageFaultRateNote: 'Sinal de diagnóstico local; sozinho não comprova acesso ao disco.',
    restoreDuration: 'Duração da restauração dos limites', restoreNote: 'Mede a operação do auxiliar, não a recuperação completa da interface.',
    adaptiveChanges: 'Ajustes adaptativos', adaptiveNote: 'Mudanças feitas pela política automática por processo.',
    processes: 'Processos monitorados', updated: 'Atualizado {time}', cpu: 'CPU {value}%', privateMemory: 'Privada {value} MB',
    audioProtected: 'Áudio protegido · sem limite', adaptiveAt: 'Limite adaptativo: {value} MB',
    willOptimize: 'Pronto para otimização automática', notIncluded: 'Fora da árvore de processos atual',
    reasonHigh: 'elevado por atividade sustentada de CPU', reasonLow: 'reduzido porque CPU e memória estavam baixas', reasonBaseline: 'usando limite inicial da função', reasonHolding: 'mantendo o limite atual',
    noneDetected: 'Nenhum processo elegível detectado. Abra o Spotify e aguarde a descoberta dos processos. O Memory Sentry não tenta adivinhar identidades de processos desconhecidos.',
    restoreNow: 'Restaurar limites originais agora', restoring: 'Restaurando…', refresh: 'Atualizar dados', lastAdjustment: 'Último ajuste: {time}',
    lastAction: 'Última ação automática: ajustou {count} processo(s). Os processos de serviço de áudio foram excluídos.',
    behavior: 'Política automática', active: 'Ativar otimização automática', autoExplanation: 'Otimiza somente depois que o Spotify permanece minimizado ou oculto pelo tempo selecionado. Os limites nativos originais são restaurados quando a janela fica visível.',
    delay: 'Atraso após minimizar/ocultar', delayExplanation: 'Evita otimizar em minimizações rápidas. Os limites por processo se adaptam automaticamente; não existe um teto fixo de RAM configurado manualmente.',
    seconds: '{value} segundos', save: 'Salvar configurações', saving: 'Salvando…', savedLocal: 'As configurações ficam salvas localmente neste PC.',
    events: 'Atividade recente', noEvents: 'Ainda não há atividade registrada.', statusError: 'Não foi possível conectar ao auxiliar Rust local. O diagnóstico aparece entre colchetes; consulte startup.log em %LOCALAPPDATA%\\SpicetifyMemorySentry\\logs.',
    footer: 'Compatível com Windows 10/11, Linux e macOS (x64 e ARM). O auxiliar nativo em Rust é código aberto sob GPL-3.0. Nenhuma telemetria é enviada. Os diagnósticos de CPU e memória ficam neste PC.',
    confirmRestore: 'Restaurar agora os limites originais de working set dos processos do Spotify? Se o Spotify continuar oculto, a otimização automática poderá aplicar limites adaptativos novamente depois do atraso configurado.',
    restoreFailed: 'Não foi possível restaurar os limites originais.', saveFailed: 'Não foi possível salvar as configurações.', helperApiError: 'O auxiliar local retornou um erro.',
    window: { foreground: 'Spotify em primeiro plano', 'background-visible': 'Spotify visível em segundo plano', minimized: 'Spotify minimizado', hidden: 'Janela do Spotify oculta', 'not-running': 'Spotify não está aberto', 'window-not-found': 'Spotify detectado; janela não localizada', 'window-check-error': 'Estado da janela indisponível', 'not-detected': 'Aguardando detecção' }
  }
};
const T = MSS_PT ? MSS_TEXT.pt : MSS_TEXT.en;
function tx(key, vars = {}) {
  let value = T[key] || MSS_TEXT.en[key] || key;
  for (const [name, replace] of Object.entries(vars)) value = value.replaceAll(`{${name}}`, String(replace));
  return value;
}
function MSSLocalizeEvent(message) {
  if (!MSS_PT || typeof message !== 'string') return message;
  const mappings = [
    [/^Não foi possível persistir o backup dos limites originais:/, 'Não foi possível salvar o arquivo local de recuperação:'],
    [/^Existem (\d+) backups de limites; a detecção falhou, então não vou descartá-los\.$/, 'Há $1 registros de recuperação, mas a detecção falhou; os registros foram preservados.'],
    [/^Recuperação de backups: (\d+) restaurados, (\d+) processos encerrados, (\d+) pendentes\.$/, 'Recuperação: $1 restaurados, $2 processos encerrados, $3 pendentes.'],
    [/^Spotify ficou (.*); o teto rígido por processo será ativado após (\d+)s\.$/, 'O Spotify ficou $1; os limites adaptativos serão ativados após $2 s.'],
    [/^Erro inesperado no monitor:/, 'Erro inesperado no monitor:'],
    [/^Settings updated:/, 'Configurações atualizadas:'],
    [/^Adaptive working-set cap applied to/, 'Limite adaptativo de working set aplicado a'],
    [/^Restored original limits for/, 'Limites originais restaurados para'],
    [/^Adaptive optimization started for/, 'Otimização adaptativa iniciada para'],
    [/^Adaptive policy raised/, 'Política adaptativa elevou'],
    [/^Adaptive policy lowered/, 'Política adaptativa reduziu'],
    [/^Adaptive cap update failed for/, 'Falha ao atualizar o limite adaptativo de'],
    [/^Adaptive cap update skipped for/, 'Atualização do limite adaptativo ignorada para'],
    [/^Could not query Spotify processes:/, 'Não foi possível consultar os processos do Spotify:'],
    [/^Could not inspect the Spotify window:/, 'Não foi possível verificar a janela do Spotify:'],
    [/^Adaptive working-set cap applied to (.*) PID (\d+): (\d+) MB; (.*)$/, 'Limite adaptativo aplicado a $1 PID $2: $3 MB; $4'],
    [/^Adaptive optimization started for (\d+) process\(es\); dedicated audio processes remain excluded\.$/, 'Otimização adaptativa iniciada para $1 processo(s); serviços de áudio continuam excluídos.'],
    [/^Adaptive policy raised (.*) PID (\d+) to (\d+) MB (.*)$/, 'A política adaptativa elevou $1 PID $2 para $3 MB $4'],
    [/^Adaptive policy lowered (.*) PID (\d+) to (\d+) MB (.*)$/, 'A política adaptativa reduziu $1 PID $2 para $3 MB $4'],
    [/^Restored original limits for (.*) PID (\d+): (.*)$/, 'Limites originais restaurados para $1 PID $2: $3'],
    [/^Discarded recovery record for PID (\d+): (.*)$/, 'Registro de recuperação descartado para PID $1: $2'],
    [/^Settings updated: (.*)$/, 'Configurações atualizadas: $1'],
    [/^Local API could not start on port (\d+): (.*)$/, 'A API local não iniciou na porta $1: $2']
  ];
  for (const [pattern, replacement] of mappings) if (pattern.test(message)) return message.replace(pattern, replacement);
  return message;
}
function MSSEnsureStyles() {
  if (document.getElementById('mss-styles')) return;
  const style = document.createElement('style');
  style.id = 'mss-styles';
  style.textContent = `
    .mss-page{max-width:1200px;margin:0 auto;padding:28px 24px 48px;color:var(--spice-text,#f5f5f5);font-family:var(--font-family,inherit)}
    .mss-hero{display:flex;gap:20px;align-items:flex-start;justify-content:space-between;margin-bottom:22px}.mss-kicker{font-size:11px;font-weight:800;letter-spacing:.14em;text-transform:uppercase;color:var(--spice-subtext,#a7a7a7);margin-bottom:8px}.mss-title{font-size:clamp(30px,4vw,42px);line-height:1.08;letter-spacing:-.04em;font-weight:800;margin:0 0 10px}.mss-copy{color:var(--spice-subtext,#a7a7a7);font-size:14px;max-width:690px;line-height:1.6}
    .mss-pill{display:inline-flex;align-items:center;gap:8px;border:1px solid var(--spice-button,#1ed760);border-radius:999px;padding:8px 11px;font-size:12px;font-weight:700;white-space:nowrap}.mss-dot{width:7px;height:7px;border-radius:50%;background:var(--spice-button,#1ed760);box-shadow:0 0 0 4px color-mix(in srgb,var(--spice-button,#1ed760) 17%,transparent)}
    .mss-grid{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:12px;margin:16px 0}.mss-card,.mss-section{background:var(--spice-card,#282828);border:1px solid color-mix(in srgb,var(--spice-text,#fff) 8%,transparent);border-radius:15px;padding:18px;min-width:0}.mss-card-label{font-size:12px;color:var(--spice-subtext,#a7a7a7);font-weight:700;margin-bottom:12px}.mss-number{font-size:31px;line-height:1;font-weight:800;letter-spacing:-.04em;font-variant-numeric:tabular-nums}.mss-unit{font-size:13px;color:var(--spice-subtext,#a7a7a7);font-weight:600;margin-left:5px}.mss-note{font-size:12px;color:var(--spice-subtext,#a7a7a7);line-height:1.48;margin-top:10px}
    .mss-section{margin-top:15px}.mss-section-head{display:flex;justify-content:space-between;align-items:center;gap:12px;margin-bottom:12px}.mss-h2{font-size:17px;font-weight:800;letter-spacing:-.02em;margin:0}.mss-small{font-size:12px;color:var(--spice-subtext,#a7a7a7);line-height:1.5}.mss-list{display:flex;flex-direction:column;gap:2px}.mss-process{display:grid;grid-template-columns:minmax(0,1fr) auto auto;gap:13px;align-items:center;padding:11px 0;border-bottom:1px solid color-mix(in srgb,var(--spice-text,#fff) 8%,transparent)}.mss-process:last-child{border-bottom:0}.mss-process-name{font-size:13px;font-weight:700}.mss-process-meta{font-size:11px;color:var(--spice-subtext,#a7a7a7);margin-top:4px;line-height:1.4}.mss-memory{font-size:13px;font-variant-numeric:tabular-nums;font-weight:800;white-space:nowrap}.mss-tag{font-size:10px;text-transform:uppercase;letter-spacing:.06em;border-radius:6px;background:color-mix(in srgb,var(--spice-text,#fff) 8%,transparent);padding:5px 7px;color:var(--spice-subtext,#a7a7a7);white-space:nowrap}
    .mss-settings{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:16px}.mss-setting{display:flex;flex-direction:column;gap:8px;min-width:0}.mss-setting-label{font-weight:700;font-size:13px}.mss-select{background:var(--spice-main,#121212);border:1px solid color-mix(in srgb,var(--spice-text,#fff) 18%,transparent);color:var(--spice-text,#fff);border-radius:9px;padding:10px 11px;font-size:13px;width:100%}.mss-check{display:flex;align-items:flex-start;gap:10px;font-size:13px;line-height:1.45;cursor:pointer}.mss-check input{accent-color:var(--spice-button,#1ed760);margin-top:3px}.mss-actions{display:flex;align-items:center;gap:9px;flex-wrap:wrap;margin-top:16px}.mss-btn{border:0;border-radius:999px;padding:10px 15px;font-size:12px;font-weight:800;cursor:pointer;color:var(--spice-text,#fff);background:color-mix(in srgb,var(--spice-text,#fff) 10%,transparent)}.mss-btn:hover{filter:brightness(1.12)}.mss-btn:disabled{opacity:.45;cursor:not-allowed}.mss-btn-primary{background:var(--spice-button,#1ed760);color:var(--spice-button-text,#061006)}.mss-btn-secondary{background:transparent;border:1px solid color-mix(in srgb,var(--spice-text,#fff) 20%,transparent)}.mss-notice{border-radius:10px;padding:12px 14px;font-size:12px;line-height:1.5;margin:12px 0;background:color-mix(in srgb,var(--spice-text,#fff) 6%,transparent);color:var(--spice-subtext,#a7a7a7)}.mss-notice-warn{background:color-mix(in srgb,#e7a83e 12%,transparent);color:var(--spice-text,#fff);border:1px solid color-mix(in srgb,#e7a83e 30%,transparent)}.mss-notice-error{background:color-mix(in srgb,#e05252 12%,transparent);color:var(--spice-text,#fff);border:1px solid color-mix(in srgb,#e05252 30%,transparent)}.mss-events{display:flex;flex-direction:column;gap:0}.mss-event{display:grid;grid-template-columns:90px minmax(0,1fr);gap:12px;padding:8px 0;border-bottom:1px solid color-mix(in srgb,var(--spice-text,#fff) 8%,transparent);font-size:12px;line-height:1.45}.mss-event-time{color:var(--spice-subtext,#a7a7a7);font-variant-numeric:tabular-nums}.mss-event-msg{overflow-wrap:anywhere}.mss-empty{padding:16px 0;color:var(--spice-subtext,#a7a7a7);font-size:13px}.mss-footer{margin-top:20px;color:var(--spice-subtext,#a7a7a7);font-size:11px;line-height:1.55}
    @media(max-width:850px){.mss-page{padding:22px 14px 36px}.mss-grid{grid-template-columns:1fr}.mss-hero{flex-direction:column}.mss-settings{grid-template-columns:1fr}.mss-process{grid-template-columns:minmax(0,1fr) auto}.mss-process .mss-memory{grid-column:2;grid-row:1}.mss-tag{display:none}}
  `;
  document.head.appendChild(style);
}

async function MSSRequest(path, options = {}) {
  const response = await fetch(`${MSS_API}${path}`, {
    method: options.method || 'GET', cache: 'no-store', mode: 'cors', credentials: 'omit',
    headers: { Authorization: `Bearer ${MSS_TOKEN}`, ...(options.body ? { 'Content-Type': 'application/json' } : {}) },
    body: options.body ? JSON.stringify(options.body) : undefined
  });
  let payload = {};
  try { payload = await response.json(); } catch (_) { }
  if (!response.ok) throw new Error(payload.error || `${tx('helperApiError')} (HTTP ${response.status}).`);
  return payload;
}
function MSSFormatMB(value) {
  if (value === null || value === undefined || !Number.isFinite(Number(value))) return '—';
  return Number(value).toLocaleString(MSS_LOCALE, { maximumFractionDigits: 1, minimumFractionDigits: 1 });
}
function MSSWindowLabel(value) {
  return T.window[value] || (MSS_TEXT.en.window[value] || 'Unknown state');
}
function MSSRoleLabel(role) {
  if (!MSS_PT) return role;
  return ({ Main: 'Principal', GPU: 'GPU', Renderer: 'Renderização', Utility: 'Utilitário', Crashpad: 'Relatório de falhas', Helper: 'Auxiliar', Audio: 'Áudio' })[role] || role;
}
function MSSOpenGitHub() {
  window.open(MSS_RELEASES_URL, '_blank', 'noopener,noreferrer');
}
function MSSOpenInstallGuide() {
  window.open(MSS_INSTALL_GUIDE_URL, '_blank', 'noopener,noreferrer');
}
function MSSDate(value) {
  return value ? new Date(value).toLocaleTimeString(MSS_LOCALE) : '—';
}
function MSSApp() {
  const R = MSSReact;
  const [status, setStatus] = R.useState(null);
  const [error, setError] = R.useState('');
  const [busy, setBusy] = R.useState(false);
  const [saving, setSaving] = R.useState(false);
  const [settingsLoaded, setSettingsLoaded] = R.useState(false);
  const [config, setConfig] = R.useState({ autoOptimize: true, hiddenDelaySeconds: 2 });

  R.useEffect(() => {
    let alive = true;
    async function refresh() {
      try {
        const data = await MSSRequest('/api/status');
        if (!alive) return;
        setStatus(data); setError('');
        if (!settingsLoaded && data.config) { setConfig({ autoOptimize: data.config.autoOptimize !== false, hiddenDelaySeconds: data.config.hiddenDelaySeconds || 2 }); setSettingsLoaded(true); }
      } catch (err) {
        if (alive) {
          const detail = err && err.message ? String(err.message) : String(err || 'Unknown error');
          setError(tx('statusError') + ' [' + detail + ']');
        }
      }
    }
    refresh();
    const timer = window.setInterval(refresh, 2500);
    return () => { alive = false; window.clearInterval(timer); };
  }, [settingsLoaded]);

  function setField(key, value) { setConfig(previous => ({ ...previous, [key]: value })); }
  async function saveSettings() {
    setSaving(true);
    try {
      const result = await MSSRequest('/api/config', { method: 'POST', body: config });
      setConfig({ autoOptimize: result.config.autoOptimize, hiddenDelaySeconds: result.config.hiddenDelaySeconds });
      setStatus(await MSSRequest('/api/status')); setError('');
    } catch (_) { setError(tx('saveFailed')); }
    finally { setSaving(false); }
  }
  async function restoreLimits() {
    if (!window.confirm(tx('confirmRestore'))) return;
    setBusy(true);
    try { await MSSRequest('/api/action', { method: 'POST', body: { action: 'restoreNow' } }); setStatus(await MSSRequest('/api/status')); setError(''); }
    catch (_) { setError(tx('restoreFailed')); }
    finally { setBusy(false); }
  }

  const spotify = status?.spotify || {};
  const memory = status?.memory || {};
  const telemetry = status?.telemetry || {};
  const helperRunning = Boolean(status?.helper?.running);
  const targets = Array.isArray(memory.targets) ? memory.targets : [];
  const events = Array.isArray(status?.recentEvents) ? status.recentEvents : [];
  const activeCount = status?.cap?.cappedProcesses || 0;
  const card = (label, value, unit, note) => h('section', { className: 'mss-card' }, h('div', { className: 'mss-card-label' }, label), h('div', { className: 'mss-number' }, value, unit ? h('span', { className: 'mss-unit' }, unit) : null), h('div', { className: 'mss-note' }, note));

  return h('main', { className: 'mss-page' },
    h('header', { className: 'mss-hero' },
      h('div', null, h('div', { className: 'mss-kicker' }, tx('kicker')), h('h1', { className: 'mss-title' }, tx('title')), h('div', { className: 'mss-copy' }, tx('intro')),
        h('div', { className: 'mss-small', style: { marginTop: '10px' } }, `${MSSWindowLabel(spotify.windowState || 'not-detected')} · ${MSS_PT ? 'Idioma automático: português' : 'Auto language: English'}`)),
      h('div', { className: 'mss-pill' }, h('span', { className: 'mss-dot', style: { background: error ? '#e05252' : (helperRunning ? 'var(--spice-button,#1ed760)' : '#d6a03d') } }), error ? tx('unavailable') : (helperRunning ? tx('connected') : tx('waiting')))
    ),
    error ? h('section', { className: 'mss-notice mss-notice-error' }, h('strong', null, tx('helperMissingTitle')), h('p', { style: { margin: '7px 0 0' } }, `${tx('helperMissingBody')} ${error}`), h('div', { className: 'mss-actions' }, h('button', { className: 'mss-btn mss-btn-primary', onClick: MSSOpenGitHub }, tx('helperButton')), h('button', { className: 'mss-btn mss-btn-secondary', onClick: MSSOpenInstallGuide }, tx('installGuideButton')))) : null,
    h('div', { className: 'mss-notice mss-notice-warn' }, tx('safety')),
    h('div', { className: 'mss-grid' },
      card(tx('spotifyRam'), MSSFormatMB(memory.spotifyTotalMB), 'MB', tx('ramNote', { count: memory.processCount || 0 })),
      card(tx('optimizedRam'), MSSFormatMB(memory.targetTotalMB), 'MB', tx('managedNote', { count: memory.targetCount || 0 })),
      card(tx('activeLimits'), activeCount.toLocaleString(MSS_LOCALE), '', tx('activeNote')),
      card(tx('pageFaults'), telemetry.pageFaultDelta == null ? '—' : Number(telemetry.pageFaultDelta).toLocaleString(MSS_LOCALE), '', tx('pageFaultNote')),
      card(tx('pageFaultRate'), telemetry.pageFaultsPerSecond == null ? '—' : Number(telemetry.pageFaultsPerSecond).toLocaleString(MSS_LOCALE, { maximumFractionDigits: 1 }), '/s', tx('pageFaultRateNote')),
      card(tx('adaptiveChanges'), Number(telemetry.adaptiveAdjustments || 0).toLocaleString(MSS_LOCALE), '', tx('adaptiveNote')),
      card(tx('restoreDuration'), telemetry.lastRestoreDurationMs == null ? '—' : Number(telemetry.lastRestoreDurationMs).toLocaleString(MSS_LOCALE, { maximumFractionDigits: 1 }), telemetry.lastRestoreDurationMs == null ? '' : 'ms', tx('restoreNote'))
    ),
    h('section', { className: 'mss-section' },
      h('div', { className: 'mss-section-head' }, h('h2', { className: 'mss-h2' }, tx('processes')), h('span', { className: 'mss-small' }, tx('updated', { time: MSSDate(memory.sampledAt) }))),
      targets.length ? h('div', { className: 'mss-list' }, targets.map(process => {
        const statusText = process.protectedAudio ? tx('audioProtected') : (process.capped ? tx(process.adaptiveStatus === 'sustained-high-cpu' ? 'reasonHigh' : (process.adaptiveStatus === 'low-cpu-and-memory-headroom' ? 'reasonLow' : (process.adaptiveStatus === 'baseline' ? 'reasonBaseline' : 'reasonHolding'))) : (process.included ? tx('willOptimize') : tx('notIncluded')));
        const meta = [Number.isFinite(process.cpuPercent) ? tx('cpu', { value: Number(process.cpuPercent).toLocaleString(MSS_LOCALE, { maximumFractionDigits: 1 }) }) : null, process.privateMB != null ? tx('privateMemory', { value: MSSFormatMB(process.privateMB) }) : null, statusText].filter(Boolean).join(' · ');
        return h('div', { className: 'mss-process', key: `${process.pid}:${process.role}` },
          h('div', null, h('div', { className: 'mss-process-name' }, `${MSSRoleLabel(process.role)} · PID ${process.pid}`), h('div', { className: 'mss-process-meta' }, meta)),
          h('span', { className: 'mss-tag' }, process.protectedAudio ? (MSS_PT ? 'Áudio' : 'Audio') : (process.capped ? (MSS_PT ? 'Adaptativo' : 'Adaptive') : process.role)),
          h('div', { className: 'mss-memory' }, process.workingSetMB == null ? '—' : `${MSSFormatMB(process.workingSetMB)} MB${process.capped ? ` / ${process.limitMB} MB` : ''}`)
        );
      })) : h('div', { className: 'mss-empty' }, tx('noneDetected')),
      h('div', { className: 'mss-actions' },
        h('button', { className: 'mss-btn mss-btn-primary', disabled: busy || !helperRunning || !activeCount, onClick: restoreLimits }, busy ? tx('restoring') : tx('restoreNow')),
        h('button', { className: 'mss-btn mss-btn-secondary', disabled: !helperRunning, onClick: async () => { try { setStatus(await MSSRequest('/api/status')); setError(''); } catch (err) { const detail = err && err.message ? String(err.message) : String(err || 'Unknown error'); setError(tx('statusError') + ' [' + detail + ']'); } } }, tx('refresh')),
        telemetry.lastAdaptiveAdjustmentAt ? h('span', { className: 'mss-small' }, tx('lastAdjustment', { time: MSSDate(telemetry.lastAdaptiveAdjustmentAt) })) : null
      ),
      status?.lastAction ? h('div', { className: 'mss-notice' }, tx('lastAction', { count: status.lastAction.applied || 0 })) : null
    ),
    h('section', { className: 'mss-section' },
      h('div', { className: 'mss-section-head' }, h('h2', { className: 'mss-h2' }, tx('behavior')), h('span', { className: 'mss-small' }, helperRunning ? `${status?.helper?.runtime || 'Native'} ${status?.helper?.version || ''}` : (MSS_PT ? 'Auxiliar nativo' : 'Native companion'))),
      h('label', { className: 'mss-check', style: { marginBottom: '18px' } }, h('input', { type: 'checkbox', checked: Boolean(config.autoOptimize), onChange: event => setField('autoOptimize', event.target.checked) }), h('span', null, h('strong', null, tx('active')), h('br'), h('span', { className: 'mss-small' }, tx('autoExplanation')))),
      h('div', { className: 'mss-settings' }, h('label', { className: 'mss-setting' }, h('span', { className: 'mss-setting-label' }, tx('delay')), h('select', { className: 'mss-select', value: String(config.hiddenDelaySeconds), onChange: event => setField('hiddenDelaySeconds', Number(event.target.value)) }, [1, 2, 3, 5, 8, 10, 15, 20, 30].map(n => h('option', { key: n, value: String(n) }, tx('seconds', { value: n })))), h('span', { className: 'mss-small' }, tx('delayExplanation')))),
      h('div', { className: 'mss-actions' }, h('button', { className: 'mss-btn mss-btn-primary', disabled: saving || !helperRunning, onClick: saveSettings }, saving ? tx('saving') : tx('save')), h('span', { className: 'mss-small' }, tx('savedLocal')))
    ),
    h('section', { className: 'mss-section' }, h('div', { className: 'mss-section-head' }, h('h2', { className: 'mss-h2' }, tx('events')), h('span', { className: 'mss-small' }, status?.helper ? `${status.helper.runtime || 'Native'} ${status.helper.version || ''}` : (MSS_PT ? 'Auxiliar nativo' : 'Native companion'))),
      events.length ? h('div', { className: 'mss-events' }, events.map((event, i) => h('div', { className: 'mss-event', key: `${event.time}-${i}` }, h('span', { className: 'mss-event-time' }, MSSDate(event.time)), h('span', { className: 'mss-event-msg' }, MSSLocalizeEvent(event.message))))) : h('div', { className: 'mss-empty' }, tx('noEvents')),
      status?.lastError ? h('div', { className: 'mss-notice mss-notice-error' }, MSSLocalizeEvent(status.lastError)) : null
    ),
    h('footer', { className: 'mss-footer' }, tx('footer'))
  );
}
function render() { MSSEnsureStyles(); return h(MSSApp, {}); }
