/* Copyright (c) 2026 bifa90 — Odina */
/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* The window asks; it never decides. Everything drawn here came from the
   `home` command, and the words came from the locale files inside the binary. */

const LANGS = ['ar', 'de', 'en', 'es', 'fr', 'hi', 'ja', 'pt-BR', 'ru', 'zh-Hans'];
const invoke = window.__TAURI__ ? window.__TAURI__.core.invoke : null;

const el = {
  title: document.getElementById('title'),
  chip: document.getElementById('chip'),
  month: document.getElementById('month'),
  version: document.getElementById('version'),
  detail: document.getElementById('detail'),
  theme: document.getElementById('theme'),
  shield: document.getElementById('shield'),
};

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

/* The state colour follows the answer, not the mood: a stopped guard is not
   painted green because the page loaded. */
function paint(health) {
  const map = { active: 'var(--ok)', suspended: 'var(--warn)', stopped: 'var(--danger)', disconnected: 'var(--text3)' };
  document.documentElement.style.setProperty('--state', map[health] || 'var(--text3)');
  el.title.dataset.health = health;
}

function applyTheme(theme) {
  document.documentElement.dataset.theme = theme;
  el.theme.setAttribute('aria-pressed', String(theme === 'dark'));
}

function systemTheme() {
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

let lang = pickLang();

async function refresh() {
  if (!invoke) {
    /* Opened in a plain browser: say so instead of drawing a fake state. */
    el.version.textContent = 'odina-shell: no IPC';
    return;
  }
  const home = await invoke('home', { lang });
  document.documentElement.lang = home.lang;
  document.documentElement.dir = home.dir;
  el.title.textContent = home.title;
  el.version.textContent = home.version;
  if (home.month) {
    el.month.textContent = home.month;
    el.month.hidden = false;
  } else {
    el.month.hidden = true;
  }
  if (home.detail) {
    el.detail.textContent = home.detail;
    el.detail.hidden = false;
  } else {
    el.detail.hidden = true;
  }
  if (home.chip) {
    el.chip.textContent = home.chip;
    el.chip.dataset.on = 'on';
  } else {
    el.chip.dataset.on = 'off';
  }
  paint(home.health);
}

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

refresh().catch(() => {
  el.title.textContent = '—';
  paint('disconnected');
});
setInterval(() => refresh().catch(() => {}), 2000);
