// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
'use strict';
(async () => {
  const api = globalThis.browser ?? chrome;
  const t = key => api.i18n.getMessage(key);
  document.documentElement.lang = t('language');
  document.documentElement.dir = t('language') === 'ar' ? 'rtl' : 'ltr';
  const $ = id => document.getElementById(id);
  const id = location.hash.slice(1);
  if (!/^[0-9a-f-]{36}$/.test(id)) return window.close();
  let offer;
  try { offer = await api.runtime.sendMessage({op: 'take', id}); } catch { /* worker restarted */ }
  if (!offer) return window.close();
  const dismiss = $('dismiss'), deep = $('deep');
  dismiss.onclick = () => window.close();
  document.addEventListener('keydown', e => { if (e.key === 'Escape') window.close(); });
  $('item').textContent = offer.label;
  function show(kind, verified) {
    if (!['suspicious', 'threat'].includes(kind)) return window.close();
    const threat = kind === 'threat';
    document.querySelector('main').classList.toggle('threat', threat);
    $('state').textContent = t(threat ? 'confirmed' : verified ? 'stillSuspect' : 'suspect');
    $('effect').textContent = threat ? t('limits') : verified ? '' : t('consequence');
    deep.textContent = t('deep');
    deep.hidden = threat || verified;
    $('quarantine').textContent = t('quarantine');
    $('quarantine').hidden = !threat; // no vault capability: never promise a move
    dismiss.textContent = t(deep.hidden ? 'close' : 'ignore');
    dismiss.hidden = false;
    dismiss.focus(); // opening a warning + Enter is NOT consent to a deep scan
  }
  deep.onclick = async e => {
    if (!e.isTrusted || !offer.ticket || deep.disabled) return;
    deep.disabled = true;
    deep.hidden = true;
    dismiss.textContent = t('close'); // closing does not cancel a consented read
    const {url, ticket} = offer;
    offer = {}; // consume once, before any await; nothing persisted by the worker
    $('state').textContent = t('working');
    try { show((await api.runtime.sendNativeMessage('org.odina.context', {op: 'deep', url, ticket})).result, true); }
    catch { window.close(); }
  };
  show(offer.result, false);
})();
