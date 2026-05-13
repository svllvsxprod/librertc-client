import { invoke } from '@tauri-apps/api/core';
import appIcon from './assets/app-icon.svg?raw';
import clockIcon from './assets/clock.svg?raw';
import communityIcon from './assets/community.svg?raw';
import donationIcon from './assets/donation.svg?raw';
import downloadIcon from './assets/download.svg?raw';
import proxyIcon from './assets/proxy.svg?raw';
import telegramIcon from './assets/telegram.svg?raw';
import tributeIcon from './assets/tribute.svg?raw';
import uploadIcon from './assets/upload.svg?raw';
import '@fontsource/inter/cyrillic-400.css';
import '@fontsource/inter/latin-400.css';
import './styles.css';

type ClientProfile = {
  name: string;
  subscription_url: string;
  uri: string;
  mode: 'proxy' | 'tun';
  language: Language;
  proxy_auto: boolean;
  socks_host: string;
  socks_port: number;
  dns: string;
  olcrtc_path: string;
  welcome_dismissed: boolean;
};

type ServerProfile = {
  id: string;
  name: string;
  uri: string;
  carrier: string;
  transport: string;
};

type PublicTarget = {
  carrier: string;
  transport: string;
  name: string;
};

type ClientStatus = {
  state: 'disconnected' | 'connecting' | 'connected';
  mode: 'proxy' | 'tun';
  socks: string;
  download_bps: number;
  upload_bps: number;
  download_bytes: number;
  upload_bytes: number;
  started_at: string;
  notice: string;
  target: PublicTarget | null;
  steps: string[];
  logs: string[];
};

type Language = 'en' | 'ru';

const messages = {
  en: {
    connection: 'Connection',
    profiles: 'Profiles',
    logs: 'Runtime Logs',
    settings: 'Settings',
    disconnected: 'Disconnected',
    connecting: 'Connecting',
    connected: 'Connected',
    server: 'Server',
    noServerImported: 'No server imported',
    noServerSelected: 'No server selected',
    importFirst: 'Import a profile first',
    connect: 'Connect',
    disconnect: 'Disconnect',
    localProxy: 'Local proxy',
    started: 'Duration',
    download: 'Download',
    upload: 'Upload',
    runtime: 'Runtime',
    logsTitle: 'Logs',
    logsSubtitle: 'stdout/stderr from olcrtc',
    importLinks: 'Import olcrtc links',
    import: 'Import',
    importPlaceholder: 'Paste one or more olcrtc:// links here',
    imported: 'Imported',
    servers: 'Servers',
    noProfiles: 'No imported profiles yet.',
    delete: 'Delete',
    select: 'Select',
    deleteConfirm: 'Delete this server profile?',
    save: 'Save',
    language: 'Language',
    runtimePath: 'Runtime path',
    dns: 'DNS',
    socksHost: 'SOCKS host',
    socksPort: 'SOCKS port',
    proxyAuto: 'Automatic proxy port',
    proxyAutoHint: 'LibreRTC will choose a free local port on connect.',
    proxyManualHint: 'Disable automatic mode to set host and port manually.',
    proxyNow: 'SOCKS now',
    planned: 'System traffic',
    tunPlanned: 'TUN',
    noLogs: 'No logs yet',
    selected: 'Selected',
    stepSocks: 'Start olcrtc SOCKS runtime',
    stepExpose: 'Expose local proxy',
    stepProxySettings: 'Use app/browser proxy settings',
    stepTun: 'Start sing-box TUN bridge',
    stepKill: 'Route system traffic through TUN',
    welcomeTitle: 'Thank you for downloading LibreRTC',
    welcomeText: 'We would be glad if you subscribed to our Telegram groups to follow updates and support the community. Thank you!',
    welcomeContinue: 'Connect',
    welcomeUpdates: 'Project updates',
    welcomeCommunity: 'Community',
    welcomeTribute: 'Tribute',
    welcomeDonate: 'Donate',
  },
  ru: {
    connection: 'Подключение',
    profiles: 'Профили',
    logs: 'Логи runtime',
    settings: 'Настройки',
    disconnected: 'Отключено',
    connecting: 'Подключение',
    connected: 'Подключено',
    server: 'Сервер',
    noServerImported: 'Серверы не импортированы',
    noServerSelected: 'Сервер не выбран',
    importFirst: 'Сначала импортируйте профиль',
    connect: 'Подключить',
    disconnect: 'Отключить',
    localProxy: 'Локальный proxy',
    started: 'Длительность',
    download: 'Download',
    upload: 'Upload',
    runtime: 'Runtime',
    logsTitle: 'Логи',
    logsSubtitle: 'stdout/stderr от olcrtc',
    importLinks: 'Импорт olcrtc ссылок',
    import: 'Импорт',
    importPlaceholder: 'Вставьте одну или несколько olcrtc:// ссылок',
    imported: 'Импортировано',
    servers: 'Серверы',
    noProfiles: 'Импортированных профилей пока нет.',
    delete: 'Удалить',
    select: 'Выбрать',
    deleteConfirm: 'Удалить этот профиль сервера?',
    save: 'Сохранить',
    language: 'Язык',
    runtimePath: 'Путь к runtime',
    dns: 'DNS',
    socksHost: 'SOCKS host',
    socksPort: 'SOCKS port',
    proxyAuto: 'Автоматический порт proxy',
    proxyAutoHint: 'LibreRTC выберет свободный локальный порт при подключении.',
    proxyManualHint: 'Отключите автоматический режим, чтобы задать host и port вручную.',
    proxyNow: 'SOCKS сейчас',
    planned: 'Весь трафик ПК',
    tunPlanned: 'TUN',
    noLogs: 'Логов пока нет',
    selected: 'Выбран',
    stepSocks: 'Запуск olcrtc SOCKS runtime',
    stepExpose: 'Локальный proxy доступен',
    stepProxySettings: 'Используйте proxy в приложении/браузере',
    stepTun: 'Запуск sing-box TUN bridge',
    stepKill: 'Маршрутизация трафика ПК через TUN',
    welcomeTitle: 'Спасибо за скачивание LibreRTC',
    welcomeText: 'Большое спасибо за то что скачали данный клиент, будем рады если вы подпишитесь на группы в Telegram чтобы следить за обновлениями и поддержать наше коммьюнити, спасибо!',
    welcomeContinue: 'Подключиться',
    welcomeUpdates: 'Обновления проекта',
    welcomeCommunity: 'Коммьюнити',
    welcomeTribute: 'Tribute',
    welcomeDonate: 'Поддержать',
  },
} satisfies Record<Language, Record<string, string>>;

let currentLanguage: Language = 'en';

const app = document.querySelector<HTMLDivElement>('#app');
if (!app) throw new Error('App root is missing');

app.innerHTML = `
  <header class="window-titlebar" id="windowTitlebar" data-tauri-drag-region>
    <div class="window-brand" data-tauri-drag-region>
      <span class="window-brand-icon" data-tauri-drag-region>${appIcon}</span>
      <span data-tauri-drag-region>LibreRTC</span>
    </div>
    <div class="window-controls">
      <button id="windowMinimize" class="window-control" type="button" aria-label="Minimize">−</button>
      <button id="windowClose" class="window-control close" type="button" aria-label="Close to tray">×</button>
    </div>
  </header>
  <main class="app-shell">
    <aside class="rail">
      <div class="brand-mark"><span>LR</span></div>
      <button class="rail-item active" type="button" data-tab="home" data-title-key="connection" title="Connection">⏻</button>
      <button class="rail-item" type="button" data-tab="profiles" data-title-key="profiles" title="Profiles">▤</button>
      <button class="rail-item" type="button" data-tab="logs" data-title-key="logs" title="Logs">⌁</button>
      <button class="rail-item" type="button" data-tab="settings" data-title-key="settings" title="Settings">⚙</button>
      <div class="rail-links" aria-label="LibreRTC links">
        <a class="rail-link" href="https://t.me/svllvsxprod" target="_blank" rel="noreferrer" title="svllvsxprod">${telegramIcon}</a>
        <a class="rail-link" href="https://t.me/openlibrecommunity" target="_blank" rel="noreferrer" title="Open Libre Community">${communityIcon}</a>
        <a class="rail-link" href="https://t.me/tribute/app?startapp=dK9j" target="_blank" rel="noreferrer" title="Tribute">${tributeIcon}</a>
        <a class="rail-link" href="https://nowpayments.io/donation/svllvsx" target="_blank" rel="noreferrer" title="Donate">${donationIcon}</a>
      </div>
    </aside>

    <section class="workspace">
      <header class="topbar glass-card">
        <div>
          <p class="eyebrow">LibreRTC</p>
          <h1 id="pageTitle">Connection</h1>
        </div>
        <div class="topbar-right">
          <span id="modeLabel" class="mode-chip">Proxy</span>
          <span class="state-pill" id="statePill">Disconnected</span>
        </div>
      </header>

      <section class="tab-page home-page active" data-page="home">
        <section class="connect-card glass-card">
          <div class="server-picker">
            <div class="server-field">
              <span data-i18n="server">Server</span>
              <select id="serverSelect" class="native-server-select" tabindex="-1" aria-hidden="true"></select>
              <div class="server-dropdown" id="serverDropdown">
                <button class="server-dropdown-trigger" id="serverDropdownTrigger" type="button" aria-haspopup="listbox" aria-expanded="false">
                  <span id="serverDropdownValue">No server imported</span>
                  <span class="server-dropdown-arrow" aria-hidden="true"></span>
                </button>
                <div class="server-dropdown-menu" id="serverDropdownMenu" role="listbox"></div>
              </div>
            </div>
          </div>

          <button class="power switch" id="powerButton" type="button" aria-label="Connect or disconnect">
            <span class="switch__base-outer"></span>
            <span class="switch__base-inner"></span>
            <svg class="switch__base-neon" viewBox="0 0 40 24" width="40px" height="24px" aria-hidden="true" focusable="false">
              <defs>
                <filter id="switch-glow">
                  <feGaussianBlur result="coloredBlur" stdDeviation="1"></feGaussianBlur>
                  <feMerge>
                    <feMergeNode in="coloredBlur"></feMergeNode>
                    <feMergeNode in="SourceGraphic"></feMergeNode>
                  </feMerge>
                </filter>
                <linearGradient id="switch-gradient1" x1="0" y1="0" x2="1" y2="0">
                  <stop offset="0%" stop-color="hsl(var(--on-hue1),90%,70%)" />
                  <stop offset="100%" stop-color="hsl(var(--on-hue2),90%,70%)" />
                </linearGradient>
                <linearGradient id="switch-gradient2" x1="0.7" y1="0" x2="0.3" y2="1">
                  <stop offset="25%" stop-color="hsla(var(--on-hue1),90%,70%,0)" />
                  <stop offset="50%" stop-color="hsla(var(--on-hue1),90%,70%,0.3)" />
                  <stop offset="100%" stop-color="hsla(var(--on-hue2),90%,70%,0.3)" />
                </linearGradient>
              </defs>
              <path fill="none" filter="url(#switch-glow)" stroke="url(#switch-gradient1)" stroke-width="1" stroke-dasharray="0 100 0" stroke-dashoffset="0" stroke-linecap="round" pathLength="100" d="m.5,12C.5,5.649,5.649.5,12,.5h16c6.351,0,11.5,5.149,11.5,11.5s-5.149,11.5-11.5,11.5H12C5.649,23.5.5,18.351.5,12Z" />
            </svg>
            <span class="switch__knob-shadow"></span>
            <span class="switch__knob-container">
              <span class="switch__knob">
                <svg class="switch__knob-neon" viewBox="0 0 48 48" width="48px" height="48px" aria-hidden="true" focusable="false">
                  <circle fill="none" stroke="url(#switch-gradient2)" stroke-dasharray="0 62.5 0 37.5" stroke-linecap="round" stroke-width="1" pathLength="100" r="23" cx="24" cy="24" transform="rotate(-112.5,24,24)" />
                </svg>
              </span>
            </span>
            <span class="switch__led"></span>
            <span class="switch__text" id="powerIcon">Connect</span>
          </button>

          <p class="notice" id="noticeBox"></p>

            <div class="connection-widgets">
              <div class="metric-card proxy-card">
                <span class="metric-icon">${proxyIcon}</span>
                <div>
                  <span data-i18n="localProxy">Local proxy</span>
                  <strong id="socksValue">127.0.0.1:8808</strong>
                </div>
              </div>
              <div class="metric-card">
                <span class="metric-icon">${clockIcon}</span>
                <div>
                  <span data-i18n="started">Started</span>
                  <strong id="startedValue">-</strong>
                </div>
              </div>
              <div class="metric-card">
                <span class="metric-icon down">${downloadIcon}</span>
                <div>
                  <span data-i18n="download">Download</span>
                  <strong id="downloadValue">0 B/s</strong>
                </div>
              </div>
              <div class="metric-card">
                <span class="metric-icon up">${uploadIcon}</span>
                <div>
                  <span data-i18n="upload">Upload</span>
                  <strong id="uploadValue">0 B/s</strong>
              </div>
            </div>
          </div>
        </section>

      </section>

      <section class="tab-page logs-page" data-page="logs">
        <section class="logs glass-card">
          <div class="section-title compact">
            <div><p class="eyebrow" data-i18n="runtime">Runtime</p><h2 data-i18n="logsTitle">Logs</h2></div>
            <span class="muted" data-i18n="logsSubtitle">stdout/stderr from olcrtc</span>
          </div>
          <pre id="logsBox">Client ready</pre>
        </section>
      </section>

      <section class="tab-page profiles-page" data-page="profiles">
        <section class="glass-card import-card">
          <div class="section-title">
            <div><p class="eyebrow" data-i18n="profiles">Profiles</p><h2 data-i18n="importLinks">Import olcrtc links</h2></div>
            <button class="secondary" id="importButton" type="button" data-i18n="import">Import</button>
          </div>
          <textarea id="importInput" rows="8" spellcheck="false" data-i18n-placeholder="importPlaceholder" placeholder="Paste one or more olcrtc:// links here"></textarea>
        </section>

        <section class="glass-card server-list-card">
          <div class="section-title compact">
            <div><p class="eyebrow" data-i18n="imported">Imported</p><h2 data-i18n="servers">Servers</h2></div>
          </div>
          <div id="serverList" class="server-list"></div>
        </section>
      </section>

      <section class="tab-page settings-page" data-page="settings">
        <form class="settings-card glass-card" id="profileForm">
          <div class="section-title">
            <div><p class="eyebrow" data-i18n="settings">Settings</p><h2 data-i18n="runtime">Runtime</h2></div>
            <button class="secondary" id="saveButton" type="submit" data-i18n="save">Save</button>
          </div>

          <div class="settings-scroll">
            <div class="settings-section">
              <div class="setting-row">
                <div class="setting-copy">
                  <strong data-i18n="language">Language</strong>
                  <span>LibreRTC interface language</span>
                </div>
                <div class="language-dropdown" id="languageDropdown">
                  <select id="languageSelect" class="native-language-select" tabindex="-1" aria-hidden="true"><option value="en">English</option><option value="ru">Русский</option></select>
                  <button class="language-dropdown-trigger" id="languageDropdownTrigger" type="button" aria-haspopup="listbox" aria-expanded="false">
                    <span id="languageDropdownValue">English</span>
                    <span class="language-dropdown-arrow" aria-hidden="true"></span>
                  </button>
                  <div class="language-dropdown-menu" id="languageDropdownMenu" role="listbox">
                    <button class="language-dropdown-option" type="button" role="option" data-language="en"><strong>English</strong><span>Interface language</span></button>
                    <button class="language-dropdown-option" type="button" role="option" data-language="ru"><strong>Русский</strong><span>Язык интерфейса</span></button>
                  </div>
                </div>
              </div>
            </div>

            <div class="settings-section">
              <div class="setting-copy section-copy">
                <strong>Mode</strong>
                <span>TUN starts sing-box next to the app and asks for administrator rights.</span>
              </div>
              <div class="mode-switch" role="radiogroup" aria-label="Connection mode">
                <button class="mode active" type="button" data-mode="proxy"><strong>Proxy</strong><span data-i18n="proxyNow">SOCKS now</span></button>
                <button class="mode" type="button" data-mode="tun"><strong>TUN</strong><span data-i18n="planned">Planned</span></button>
              </div>
            </div>

            <div class="settings-section">
              <div class="setting-row">
                <div class="setting-copy"><strong data-i18n="dns">DNS</strong><span>Runtime DNS endpoint</span></div>
                <input id="dnsInput" autocomplete="off" />
              </div>
              <div class="setting-row">
                <div class="setting-copy"><strong data-i18n="proxyAuto">Automatic proxy port</strong><span data-i18n="proxyAutoHint">LibreRTC will choose a free local port on connect.</span></div>
                <label class="switch-row"><input id="proxyAutoInput" type="checkbox" /><span data-i18n="proxyManualHint">Disable automatic mode to set host and port manually.</span></label>
              </div>
              <div class="setting-row">
                <div class="setting-copy"><strong data-i18n="socksHost">SOCKS host</strong><span>Local proxy bind address</span></div>
                <input id="socksHostInput" autocomplete="off" />
              </div>
              <div class="setting-row">
                <div class="setting-copy"><strong data-i18n="socksPort">SOCKS port</strong><span>Local proxy port</span></div>
                <input id="socksPortInput" type="number" min="1" max="65535" />
              </div>
            </div>
          </div>
        </form>
      </section>
    </section>
  </main>
  <div class="welcome-overlay" id="welcomeOverlay" hidden>
    <section class="welcome-card glass-card">
      <div class="welcome-mark"><span>LR</span></div>
      <p class="eyebrow">LibreRTC</p>
      <h2 data-i18n="welcomeTitle">Спасибо за скачивание LibreRTC</h2>
      <p class="welcome-copy" data-i18n="welcomeText">Большое спасибо за то что скачали данный клиент, будем рады если вы подпишитесь на группы в Telegram чтобы следить за обновлениями и поддержать наше коммьюнити, спасибо!</p>
      <div class="welcome-links">
        <a class="welcome-link" href="https://t.me/svllvsxprod" target="_blank" rel="noreferrer"><span>${telegramIcon}</span><strong data-i18n="welcomeUpdates">Обновления проекта</strong></a>
        <a class="welcome-link" href="https://t.me/openlibrecommunity" target="_blank" rel="noreferrer"><span>${communityIcon}</span><strong data-i18n="welcomeCommunity">Коммьюнити</strong></a>
        <a class="welcome-link" href="https://t.me/tribute/app?startapp=dK9j" target="_blank" rel="noreferrer"><span>${tributeIcon}</span><strong data-i18n="welcomeTribute">Tribute</strong></a>
        <a class="welcome-link" href="https://nowpayments.io/donation/svllvsx" target="_blank" rel="noreferrer"><span>${donationIcon}</span><strong data-i18n="welcomeDonate">Поддержать</strong></a>
      </div>
      <button class="welcome-continue" id="welcomeContinue" type="button" data-i18n="welcomeContinue">Подключиться</button>
    </section>
  </div>
`;

const form = element<HTMLFormElement>('profileForm');
const windowTitlebar = element<HTMLElement>('windowTitlebar');
const windowMinimize = element<HTMLButtonElement>('windowMinimize');
const windowClose = element<HTMLButtonElement>('windowClose');
const statePill = element<HTMLDivElement>('statePill');
const serverSelect = element<HTMLSelectElement>('serverSelect');
const serverDropdown = element<HTMLDivElement>('serverDropdown');
const serverDropdownTrigger = element<HTMLButtonElement>('serverDropdownTrigger');
const serverDropdownValue = element<HTMLSpanElement>('serverDropdownValue');
const serverDropdownMenu = element<HTMLDivElement>('serverDropdownMenu');
const importInput = element<HTMLTextAreaElement>('importInput');
const importButton = element<HTMLButtonElement>('importButton');
const serverList = element<HTMLDivElement>('serverList');
const languageSelect = element<HTMLSelectElement>('languageSelect');
const languageDropdown = element<HTMLDivElement>('languageDropdown');
const languageDropdownTrigger = element<HTMLButtonElement>('languageDropdownTrigger');
const languageDropdownValue = element<HTMLSpanElement>('languageDropdownValue');
const languageDropdownMenu = element<HTMLDivElement>('languageDropdownMenu');
const dnsInput = element<HTMLInputElement>('dnsInput');
const proxyAutoInput = element<HTMLInputElement>('proxyAutoInput');
const socksHostInput = element<HTMLInputElement>('socksHostInput');
const socksPortInput = element<HTMLInputElement>('socksPortInput');
const powerButton = element<HTMLButtonElement>('powerButton');
const powerIcon = element<HTMLSpanElement>('powerIcon');
const modeLabel = element<HTMLSpanElement>('modeLabel');
const noticeBox = element<HTMLParagraphElement>('noticeBox');
const socksValue = element<HTMLElement>('socksValue');
const downloadValue = element<HTMLElement>('downloadValue');
const uploadValue = element<HTMLElement>('uploadValue');
const startedValue = element<HTMLElement>('startedValue');
const logsBox = element<HTMLPreElement>('logsBox');
const pageTitle = element<HTMLElement>('pageTitle');
const welcomeOverlay = element<HTMLDivElement>('welcomeOverlay');
const welcomeContinue = element<HTMLButtonElement>('welcomeContinue');

let currentProfile: ClientProfile | null = null;
let currentServers: ServerProfile[] = [];
let currentStatus: ClientStatus | null = null;
let welcomeCheckInFlight = false;

windowTitlebar.addEventListener('pointerdown', async (event) => {
  if (event.button !== 0) return;
  if (event.target instanceof HTMLElement && event.target.closest('.window-controls')) return;
  await invoke('window_start_dragging');
});

windowMinimize.addEventListener('click', async () => {
  await invoke('window_minimize');
});

windowClose.addEventListener('click', async () => {
  await invoke('window_hide_to_tray');
});

for (const button of document.querySelectorAll<HTMLButtonElement>('.rail-item')) {
  button.addEventListener('click', () => showTab(button.dataset.tab ?? 'home'));
}

for (const link of document.querySelectorAll<HTMLAnchorElement>('.rail-link')) {
  link.addEventListener('click', async (event) => {
    event.preventDefault();
    await invoke('open_external', { url: link.href });
  });
}

for (const link of document.querySelectorAll<HTMLAnchorElement>('.welcome-link')) {
  link.addEventListener('click', async (event) => {
    event.preventDefault();
    await invoke('open_external', { url: link.href });
  });
}

welcomeContinue.addEventListener('click', async () => {
  try {
    currentProfile = await invoke<ClientProfile>('dismiss_welcome');
    welcomeOverlay.hidden = true;
  } catch (error) {
    renderNotice(String(error));
  }
});

for (const button of document.querySelectorAll<HTMLButtonElement>('.mode')) {
  button.addEventListener('click', () => setMode(button.dataset.mode === 'tun' ? 'tun' : 'proxy'));
}

languageSelect.addEventListener('change', async () => {
  await setLanguage(selectedLanguage());
});

proxyAutoInput.addEventListener('change', updateProxyInputs);

serverDropdownTrigger.addEventListener('click', () => {
  const isOpen = serverDropdown.classList.toggle('open');
  serverDropdownTrigger.setAttribute('aria-expanded', String(isOpen));
});

document.addEventListener('click', (event) => {
  if (event.target instanceof Node && serverDropdown.contains(event.target)) return;
  closeServerDropdown();
});

languageDropdownTrigger.addEventListener('click', () => {
  const isOpen = languageDropdown.classList.toggle('open');
  languageDropdownTrigger.setAttribute('aria-expanded', String(isOpen));
});

document.addEventListener('click', (event) => {
  if (event.target instanceof Node && languageDropdown.contains(event.target)) return;
  closeLanguageDropdown();
});

for (const option of languageDropdownMenu.querySelectorAll<HTMLButtonElement>('.language-dropdown-option')) {
  option.addEventListener('click', async () => {
    const language = option.dataset.language === 'ru' ? 'ru' : 'en';
    closeLanguageDropdown();
    await setLanguage(language);
  });
}

form.addEventListener('submit', async (event) => {
  event.preventDefault();
  await saveSettings();
});

serverSelect.addEventListener('change', async () => {
  if (!serverSelect.value) return;
  try {
    currentProfile = await invoke<ClientProfile>('select_server', { id: serverSelect.value });
    renderProfile(currentProfile);
    closeServerDropdown();
    await refreshStatus();
  } catch (error) {
    renderNotice(String(error));
  }
});

importButton.addEventListener('click', async () => {
  try {
    currentServers = await invoke<ServerProfile[]>('import_servers', { raw: importInput.value });
    importInput.value = '';
    renderServers();
    currentProfile = await invoke<ClientProfile>('get_profile');
    renderProfile(currentProfile);
    showTab('home');
  } catch (error) {
    renderNotice(String(error));
  }
});

powerButton.addEventListener('click', async () => {
  try {
    await saveSettings();
    const running = currentStatus?.state === 'connected' || currentStatus?.state === 'connecting';
    if (!running) {
      renderStatus({
        ...(currentStatus ?? await invoke<ClientStatus>('get_status')),
        state: 'connecting',
        mode: selectedMode(),
        notice: '',
      });
    }
    renderStatus(await invoke<ClientStatus>(running ? 'disconnect' : 'connect'));
  } catch (error) {
    renderNotice(String(error));
    await refreshStatus();
  }
});

await boot();
setInterval(refreshStatus, 1000);

async function boot() {
  try {
    currentProfile = await invoke<ClientProfile>('get_profile');
    currentServers = await invoke<ServerProfile[]>('get_servers');
    renderServers();
    renderProfile(currentProfile);
    renderStatus(await invoke<ClientStatus>('get_status'));
    welcomeOverlay.hidden = true;
  } catch (error) {
    renderNotice(String(error));
  }
}

async function refreshStatus() {
  try {
    renderStatus(await invoke<ClientStatus>('get_status'));
  } catch (error) {
    renderNotice(String(error));
  }
}

async function saveSettings() {
  if (!currentProfile) return;
  const profile = {
    ...currentProfile,
    mode: selectedMode(),
    language: selectedLanguage(),
    proxy_auto: proxyAutoInput.checked,
    socks_host: socksHostInput.value,
    socks_port: Number(socksPortInput.value),
    dns: dnsInput.value,
  } satisfies ClientProfile;
  currentProfile = await invoke<ClientProfile>('save_profile', { profile });
  renderProfile(currentProfile);
  renderNotice('');
}

async function setLanguage(language: Language) {
  languageSelect.value = language;
  currentLanguage = language;
  renderLanguageDropdown();
  applyTranslations();
  await saveSettings();
}

function renderProfile(profile: ClientProfile) {
  currentLanguage = normalizeLanguage(profile.language);
  languageSelect.value = currentLanguage;
  renderLanguageDropdown();
  proxyAutoInput.checked = profile.proxy_auto;
  dnsInput.value = profile.dns;
  socksHostInput.value = profile.socks_host;
  socksPortInput.value = String(profile.socks_port);
  setMode(profile.mode);
  updateProxyInputs();
  applyTranslations();
  const selected = currentServers.find((server) => server.uri === profile.uri);
  serverSelect.value = selected?.id ?? '';
  renderServerDropdown();
}

function renderServers() {
  if (currentServers.length === 0) {
    serverSelect.innerHTML = `<option value="">${escapeHtml(t('importFirst'))}</option>`;
    renderServerDropdown();
    serverList.innerHTML = `<p class="empty-state">${escapeHtml(t('noProfiles'))}</p>`;
    return;
  }
  serverSelect.innerHTML = currentServers
    .map((server) => `<option value="${escapeHtml(server.id)}">${escapeHtml(server.name)} · ${escapeHtml(server.carrier)}/${escapeHtml(server.transport)}</option>`)
    .join('');
  renderServerDropdown();
  serverList.innerHTML = currentServers
    .map((server) => `<div class="server-row"><button class="server-main server-select-action" type="button" data-server-id="${escapeHtml(server.id)}"><strong>${escapeHtml(server.name)}</strong><span>${escapeHtml(server.carrier)}/${escapeHtml(server.transport)}</span></button><button class="delete-button server-delete-action" type="button" data-server-id="${escapeHtml(server.id)}">${escapeHtml(t('delete'))}</button></div>`)
    .join('');
  for (const row of serverList.querySelectorAll<HTMLButtonElement>('.server-select-action')) {
    row.addEventListener('click', async () => {
      currentProfile = await invoke<ClientProfile>('select_server', { id: row.dataset.serverId ?? '' });
      renderProfile(currentProfile);
      showTab('home');
    });
  }
  for (const row of serverList.querySelectorAll<HTMLButtonElement>('.server-delete-action')) {
    row.addEventListener('click', async () => {
      if (!confirm(t('deleteConfirm'))) return;
      currentServers = await invoke<ServerProfile[]>('delete_server', { id: row.dataset.serverId ?? '' });
      currentProfile = await invoke<ClientProfile>('get_profile');
      renderServers();
      renderProfile(currentProfile);
      await refreshStatus();
    });
  }
}

function renderServerDropdown() {
  const selected = currentServers.find((server) => server.id === serverSelect.value) ?? currentServers[0];
  serverDropdown.classList.toggle('empty', currentServers.length === 0);
  serverDropdownValue.textContent = selected ? `${selected.name} · ${selected.carrier}/${selected.transport}` : t('importFirst');
  serverDropdownMenu.innerHTML = currentServers.length === 0
    ? `<div class="server-dropdown-empty">${escapeHtml(t('importFirst'))}</div>`
    : currentServers
      .map((server) => {
        const active = server.id === serverSelect.value ? ' active' : '';
        return `<button class="server-dropdown-option${active}" type="button" role="option" aria-selected="${server.id === serverSelect.value}" data-server-id="${escapeHtml(server.id)}"><strong>${escapeHtml(server.name)}</strong><span>${escapeHtml(server.carrier)}/${escapeHtml(server.transport)}</span></button>`;
      })
      .join('');
  for (const option of serverDropdownMenu.querySelectorAll<HTMLButtonElement>('.server-dropdown-option')) {
    option.addEventListener('click', () => selectServer(option.dataset.serverId ?? ''));
  }
}

async function selectServer(id: string) {
  if (!id) return;
  try {
    serverSelect.value = id;
    currentProfile = await invoke<ClientProfile>('select_server', { id });
    renderProfile(currentProfile);
    closeServerDropdown();
    await refreshStatus();
  } catch (error) {
    renderNotice(String(error));
  }
}

function closeServerDropdown() {
  serverDropdown.classList.remove('open');
  serverDropdownTrigger.setAttribute('aria-expanded', 'false');
}

function renderLanguageDropdown() {
  const language = selectedLanguage();
  languageDropdownValue.textContent = language === 'ru' ? 'Русский' : 'English';
  for (const option of languageDropdownMenu.querySelectorAll<HTMLButtonElement>('.language-dropdown-option')) {
    const active = option.dataset.language === language;
    option.classList.toggle('active', active);
    option.setAttribute('aria-selected', String(active));
  }
}

function closeLanguageDropdown() {
  languageDropdown.classList.remove('open');
  languageDropdownTrigger.setAttribute('aria-expanded', 'false');
}

function renderStatus(status: ClientStatus) {
  const previousState = currentStatus?.state;
  currentStatus = status;
  const running = status.state === 'connected' || status.state === 'connecting';
  statePill.textContent = label(status.state);
  statePill.dataset.state = status.state;
  powerButton.classList.toggle('running', running);
  powerButton.classList.toggle('connecting', status.state === 'connecting');
  powerButton.disabled = status.state === 'connecting';
  powerIcon.textContent = status.state === 'connecting' ? t('connecting') : running ? t('disconnect') : t('connect');
  modeLabel.textContent = status.mode === 'tun' ? t('tunPlanned') : 'Proxy';
  socksValue.textContent = status.socks;
  downloadValue.textContent = formatSpeed(status.download_bps);
  uploadValue.textContent = formatSpeed(status.upload_bps);
  startedValue.textContent = formatStarted(status.started_at);
  logsBox.textContent = status.logs.length > 0 ? status.logs.slice().reverse().join('\n') : t('noLogs');
  renderNotice(status.notice);
  if (previousState !== 'connected' && status.state === 'connected') {
    window.setTimeout(() => void showWelcomeAfterFirstConnect(), 800);
  }
}

async function showWelcomeAfterFirstConnect() {
  if (!currentProfile || currentProfile.welcome_dismissed || welcomeCheckInFlight || !welcomeOverlay.hidden) return;
  welcomeCheckInFlight = true;
  try {
    const online = await invoke<boolean>('check_public_internet');
    if (!online) return;
    currentProfile = await invoke<ClientProfile>('get_profile');
    if (!currentProfile.welcome_dismissed) {
      welcomeOverlay.hidden = false;
    }
  } catch (error) {
    console.warn('welcome internet check failed', error);
  } finally {
    welcomeCheckInFlight = false;
  }
}

function showTab(tab: string) {
  for (const page of document.querySelectorAll<HTMLElement>('.tab-page')) {
    page.classList.toggle('active', page.dataset.page === tab);
  }
  for (const button of document.querySelectorAll<HTMLButtonElement>('.rail-item')) {
    button.classList.toggle('active', button.dataset.tab === tab);
  }
  pageTitle.textContent = tab === 'profiles' ? t('profiles') : tab === 'settings' ? t('settings') : tab === 'logs' ? t('logs') : t('connection');
}

function setMode(mode: 'proxy' | 'tun') {
  for (const button of document.querySelectorAll<HTMLButtonElement>('.mode')) {
    button.classList.toggle('active', button.dataset.mode === mode);
  }
}

function selectedMode(): 'proxy' | 'tun' {
  const active = document.querySelector<HTMLButtonElement>('.mode.active');
  return active?.dataset.mode === 'tun' ? 'tun' : 'proxy';
}

function selectedLanguage(): Language {
  return languageSelect.value === 'ru' ? 'ru' : 'en';
}

function updateProxyInputs() {
  socksPortInput.disabled = proxyAutoInput.checked;
  socksPortInput.placeholder = proxyAutoInput.checked ? 'auto' : '8808';
}

function normalizeLanguage(value: string): Language {
  return value === 'ru' ? 'ru' : 'en';
}

function t(key: keyof typeof messages.en) {
  return messages[currentLanguage][key] ?? messages.en[key];
}

function applyTranslations() {
  for (const item of document.querySelectorAll<HTMLElement>('[data-i18n]')) {
    const key = item.dataset.i18n as keyof typeof messages.en;
    item.textContent = t(key);
  }
  for (const item of document.querySelectorAll<HTMLInputElement | HTMLTextAreaElement>('[data-i18n-placeholder]')) {
    const key = item.dataset.i18nPlaceholder as keyof typeof messages.en;
    item.placeholder = t(key);
  }
  for (const item of document.querySelectorAll<HTMLElement>('[data-title-key]')) {
    const key = item.dataset.titleKey as keyof typeof messages.en;
    item.title = t(key);
  }
  renderServers();
  if (currentStatus) renderStatus(currentStatus);
  const active = document.querySelector<HTMLElement>('.rail-item.active');
  showTab(active?.dataset.tab ?? 'home');
}

function renderNotice(message: string) {
  noticeBox.textContent = message;
  noticeBox.hidden = message.length === 0;
}

function formatStarted(value: string) {
  if (!value) return '-';
  const seconds = Number(value);
  if (!Number.isFinite(seconds) || seconds <= 0) return value;
  const elapsed = Math.max(0, Math.floor(Date.now() / 1000) - seconds);
  const hours = Math.floor(elapsed / 3600);
  const minutes = Math.floor((elapsed % 3600) / 60);
  const remainingSeconds = elapsed % 60;
  const two = (part: number) => String(part).padStart(2, '0');
  return hours > 0 ? `${hours}:${two(minutes)}:${two(remainingSeconds)}` : `${minutes}:${two(remainingSeconds)}`;
}

function formatSpeed(value: number) {
  if (!Number.isFinite(value) || value <= 0) return '0 B/s';
  if (value < 1024) return `${value.toFixed(0)} B/s`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB/s`;
  return `${(value / 1024 / 1024).toFixed(1)} MB/s`;
}

function label(state: ClientStatus['state']) {
  if (state === 'connected') return t('connected');
  if (state === 'connecting') return t('connecting');
  return t('disconnected');
}

function escapeHtml(value: string) {
  return value.replace(/[&<>'"]/g, (char) => {
    const entities: Record<string, string> = {
      '&': '&amp;',
      '<': '&lt;',
      '>': '&gt;',
      "'": '&#39;',
      '"': '&quot;',
    };
    return entities[char] ?? char;
  });
}

function element<T extends HTMLElement>(id: string): T {
  const value = document.getElementById(id);
  if (!value) throw new Error(`Element ${id} is missing`);
  return value as T;
}
