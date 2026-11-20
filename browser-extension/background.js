// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
'use strict';
const api = globalThis.browser ?? chrome;
const host = 'org.odina.context';
const menu = 'odina.inspect';
const notice = api.runtime.getURL('notice.html');
const pending = new Map(); // one-shot handoffs to our own newly opened window, never disk/history

async function installMenu() {
  await api.contextMenus.removeAll();
  api.contextMenus.create({id: menu, title: api.i18n.getMessage('menu'), contexts: ['link', 'page']});
}
api.runtime.onInstalled.addListener(installMenu);
api.runtime.onStartup.addListener(installMenu);

function isLocal(url) {
  return typeof url === 'string' && url.startsWith('file:///') && !url.startsWith('file:////') &&
    url.length <= 16384 && !/[?#\\\u0000-\u001f]/u.test(url);
}

// Also the entry point timed by the browser harness. No navigation/download listener.
async function inspectClick(info) {
  if (info.menuItemId !== menu || !isLocal(info.pageUrl)) return 'unknown';
  if ((info.frameId > 0 || info.frameUrl) && !isLocal(info.frameUrl)) return 'unknown';
  const url = info.linkUrl ?? info.frameUrl ?? info.pageUrl;
  if (!isLocal(url)) return 'unknown'; // never forward HTTP(S), even to the native host
  let id;
  try {
    const response = await api.runtime.sendNativeMessage(host, {op: 'inspect', url});
    if (response?.result === 'clean') return 'clean';
    const suspect = response?.result === 'suspicious' && /^[0-9a-f]{16}$/.test(response.ticket);
    if (!suspect && response?.result !== 'threat') return 'unknown';
    id = crypto.randomUUID();
    const label = decodeURIComponent(url.replace(/\/$/, '').split('/').pop() || '/')
      .replace(/[\u0000-\u001f\u007f-\u009f\u202a-\u202e\u2066-\u2069]/gu, '').slice(0, 120);
    pending.set(id, {result: response.result, label, ...(suspect ? {url, ticket: response.ticket} : {})});
    // No page data in the window URL. Closing it never implies consent.
    const timer = setTimeout(() => pending.delete(id), 20000);
    try { await api.windows.create({url: `${notice}#${id}`, type: 'popup', width: 460, height: 300}); }
    catch (err) { clearTimeout(timer); throw err; }
    return response.result;
  } catch {
    if (id) pending.delete(id);
    return 'unknown'; // offline/unregistered is not a clean verdict
  }
}
api.contextMenus.onClicked.addListener(inspectClick);
api.runtime.onMessage.addListener((msg, sender, send) => {
  if (msg?.op !== 'take' || typeof msg.id !== 'string' || sender.id !== api.runtime.id ||
      sender.url !== `${notice}#${msg.id}`) return;
  const item = pending.get(msg.id);
  pending.delete(msg.id);
  send(item ?? null);
});
