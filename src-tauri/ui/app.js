/* Copyright (c) 2026 bifa90 — Odina */
/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* The window asks; it never decides. Every word drawn here came from the
   locale files inside the binary, and every state came from the resident. */

const LANGS = ['ar', 'de', 'en', 'es', 'fr', 'hi', 'ja', 'pt-BR', 'ru', 'zh-Hans'];
const invoke = window.__TAURI__ ? window.__TAURI__.core.invoke : null;

const el = {
  home: document.getElementById('screen-home'),
  langScreen: document.getElementById('screen-lang'),
  title: document.getElementById('title'),
  chip: document.getElementById('chip'),
  month: document.getElementById('month'),
  version: document.getElementById('version'),
  detail: document.getElementById('detail'),
  theme: document.getElementById('theme'),
  toLang: document.getElementById('to-lang'),
  langTitle: document.getElementById('lang-title'),
  langSub: document.getElementById('lang-sub'),
  langNote: document.getElementById('lang-note'),
  langs: document.getElementById('langs'),
};

let strings = {};
let current = 'ar';

function stored(key) {
  try {
    return localStorage.getItem(key);
  } catch (e) {
    return null;
  }
}

function store(key, value) {
  try {
    localStorage.setItem(key, value);
  } catch (e) {
    /* a locked-down profile simply forgets the choice */
  }
}

/* The language comes from the profile, then from the system, then Arabic. */
function pickLang() {
  const saved = stored('odina.lang');
  if (saved && LANGS.includes(saved)) return saved;
  const system = (navigator.language || 'ar').toLowerCase();
  const hit = LANGS.find((code) => code.toLowerCase() === system);
  if (hit) return hit;
  const prefix = LANGS.find((code) => system.startsWith(code.toLowerCase() + '-'));
  return prefix || 'ar';
}

/* The state colour follows the answer, not the mood. */
function paint(health) {
  const map = {
    active: 'var(--ok)',
    suspended: 'var(--warn)',
    stopped: 'var(--danger)',
    disconnected: 'var(--text3)',
  };
  document.documentElement.style.setProperty('--state', map[health] || 'var(--text3)');
}

function applyTheme(theme) {
  document.documentElement.dataset.theme = theme;
  el.theme.setAttribute('aria-pressed', String(theme === 'dark'));
  el.theme.textContent = theme === 'dark' ? strings['settings.theme_light'] || '' : strings['settings.theme_dark'] || '';
}

function systemTheme() {
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

async function loadStrings(lang) {
  if (!invoke) return;
  strings = await invoke('strings', { lang });
  el.langTitle.textContent = strings['lang.title'] || '';
  el.langSub.textContent = strings['lang.sub'] || '';
  el.langNote.textContent = strings['lang.draft'] || '';
  applyTheme(document.documentElement.dataset.theme || systemTheme());
}

function checkMark() {
  const ns = 'http://www.w3.org/2000/svg';
  const svg = document.createElementNS(ns, 'svg');
  svg.setAttribute('viewBox', '0 0 24 24');
  svg.setAttribute('width', '14');
  svg.setAttribute('height', '14');
  svg.setAttribute('aria-hidden', 'true');
  const path = document.createElementNS(ns, 'path');
  path.setAttribute('d', 'M4 13 L9 18 L20 6');
  path.setAttribute('fill', 'none');
  path.setAttribute('stroke', 'currentColor');
  path.setAttribute('stroke-width', '2.5');
  path.setAttribute('stroke-linecap', 'round');
  path.setAttribute('stroke-linejoin', 'round');
  svg.append(path);
  return svg;
}

async function loadLanguages() {
  if (!invoke) return;
  const rows = await invoke('languages');
  el.langs.replaceChildren();
  for (const row of rows) {
    const li = document.createElement('li');
    const button = document.createElement('button');
    button.type = 'button';
    button.className = 'langcard';
    button.setAttribute('aria-pressed', String(row.code === current));
    const name = document.createElement('span');
    name.className = 'nm';
    name.textContent = row.name;
    const code = document.createElement('span');
    code.className = 'code';
    code.textContent = row.code;
    const mark = document.createElement('span');
    if (row.code === current) {
      mark.className = 'check';
      mark.append(checkMark());
    } else if (row.draft) {
      mark.className = 'draft';
      mark.textContent = strings['lang.draft'] || '';
    }
    button.append(name, code, mark);
    button.addEventListener('click', () => choose(row.code));
    li.append(button);
    el.langs.append(li);
  }
  el.toLang.textContent = strings['lang.title'] || '';
}

async function refresh() {
  if (!invoke) {
    el.version.textContent = 'odina-shell: no IPC';
    return;
  }
  const home = await invoke('home', { lang: current });
  document.documentElement.lang = home.lang;
  document.documentElement.dir = home.dir;
  el.title.textContent = home.title;
  el.version.textContent = home.version;
  el.month.textContent = home.month || '';
  el.month.hidden = !home.month;
  el.detail.textContent = home.detail || '';
  el.detail.hidden = !home.detail;
  if (home.chip) {
    el.chip.textContent = home.chip;
    el.chip.hidden = false;
  } else {
    el.chip.hidden = true;
  }
  paint(home.health);
}

function show(which) {
  el.home.hidden = which !== 'home';
  el.langScreen.hidden = which !== 'lang';
}

async function choose(code) {
  current = code;
  store('odina.lang', code);
  await loadStrings(code);
  await loadLanguages();
  await refresh();
  show('home');
}

el.toLang.addEventListener('click', () => {
  const open = el.langScreen.hidden;
  show(open ? 'lang' : 'home');
  if (open) loadLanguages();
});

el.theme.addEventListener('click', () => {
  const next = document.documentElement.dataset.theme === 'dark' ? 'light' : 'dark';
  store('odina.theme', next);
  applyTheme(next);
});

/* No stored choice: follow the system, and follow it while running (D123). */
const systemDark = window.matchMedia('(prefers-color-scheme: dark)');
applyTheme(stored('odina.theme') || systemTheme());
systemDark.addEventListener('change', () => {
  if (!stored('odina.theme')) applyTheme(systemTheme());
});

current = pickLang();
document.documentElement.dir = current === 'ar' ? 'rtl' : 'ltr';

(async () => {
  await loadStrings(current);
  await loadLanguages();
  await refresh();
})().catch(() => {
  el.title.textContent = '—';
  paint('disconnected');
});

setInterval(() => {
  if (!el.home.hidden) refresh().catch(() => {});
}, 2000);
