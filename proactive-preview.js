// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
// Interactive example of odina-core::proactive, not a Windows event source.
// This page models one remediation job. Only explicit preference edits write
// the local baseline; the tab's simulated job survives refresh separately.
(() => {
  'use strict';
  const root = document.getElementById('screen-proactive');
  const rows = [...root.querySelectorAll('[data-surface]')];
  const editable = new Set(rows.filter(r => r.closest('[data-tier]').dataset.tier !== 'mandatory')
    .map(r => r.dataset.surface));
  const choiceKey = 'odina.preview.surface-blocks.v1';
  const jobKey = 'odina.preview.remediation.v1';
  function read(store, key) {
    try { return JSON.parse(window[store].getItem(key)); } catch { return null; }
  }
  function write(store, key, value) {
    try { window[store].setItem(key, JSON.stringify(value)); } catch { /* Private mode: in memory only. */ }
  }
  function loadChoices() {
    const saved = read('localStorage', choiceKey);
    return new Set(Array.isArray(saved) ? saved.filter(id => editable.has(id)) : []);
  }
  let blocked = loadChoices();
  const saved = read('sessionStorage', jobKey);
  let mode = ['on', 'muted', 'off'].includes(saved?.mode) ? saved.mode : 'on';
  let phase = ['scan', 'clean', 'repair', 'restored'].includes(saved?.phase) ? saved.phase : 'ready';
  const active = () => ['scan', 'clean', 'repair'].includes(phase);
  let failed = active() && saved?.failed === true;

  const next = root.querySelector('#demo-next');
  const fail = root.querySelector('#demo-fail');
  const selector = root.querySelector('#demo-mode');
  const steps = {
    ready: ['proactive.demo.start', 'scan'], restored: ['proactive.demo.start', 'scan'],
    scan: ['scan.complete', 'clean'], clean: ['proactive.demo.cleaned', 'repair'],
    repair: ['proactive.demo.repaired', 'restored'],
  };
  function bind(el, key) {
    el.dataset.k = key;
    if (el.hasAttribute('data-tmpl')) el.dataset.tmpl = key;
    const text = renderString(window.ODINA_STRINGS[document.documentElement.lang][key], el);
    if (el.textContent !== text) el.textContent = text;
  }
  function render() {
    const off = mode === 'off';
    const hold = active();
    root.dataset.phase = phase;
    selector.value = mode;
    // New turn-off requests go through the real consent flow (§4.5); here
    // the selector illustrates an already-off device, not a way out of a job.
    selector.querySelector('[value="off"]').disabled = hold && !off;
    next.disabled = off;
    bind(next, steps[phase][0]);
    fail.hidden = off || !hold;
    fail.disabled = failed;
    root.querySelector('#remediation-error').hidden = !failed || off;
    bind(root.querySelector('#remediation-status'), 'proactive.demo.' + (off ? 'off' : phase));
    for (const row of rows) {
      const id = row.dataset.surface;
      const baseline = !editable.has(id) || blocked.has(id);
      const effective = off ? 'off' : hold ? 'contained' : baseline ? 'blocked' : 'allowed';
      row.dataset.effective = effective;
      bind(row.querySelector('.surface-state'), 'proactive.state.' + effective);
      const action = row.querySelector('[data-set-block]');
      if (action) {
        action.disabled = hold || off;
        bind(action.querySelector('span'), baseline ? 'button.allow' : 'button.block');
      }
    }
    // A saved WSH grant on the linked exceptions screen must not look like
    // effective permission while the surface is blocked. Its expiry is unchanged.
    const exc = document.getElementById('screen-exception');
    const reason = off ? 'proactive.demo.off' : hold ? 'proactive.demo.' + phase :
      blocked.has('wsh_script') ? 'proactive.user_wsh_block' : null;
    bind(exc.querySelector('.excbadge b'), reason ? 'proactive.saved_until' : 'exception.badge');
    bind(exc.querySelector('.excbadge .when'), reason || 'exception.live_note');
    bind(exc.querySelector('.exccard.live .st'), reason ? 'proactive.saved_until' : 'exception.state.allowed_until');
    bind(exc.querySelector('.exccard.live .conseq'), reason || 'exception.consequence.allow');
  }
  function saveJob() {
    write('sessionStorage', jobKey, {mode, phase, failed});
    render();
  }
  root.addEventListener('click', event => {
    const action = event.target.closest('[data-set-block]');
    if (!action || active() || mode === 'off') return;
    const id = action.closest('[data-surface]').dataset.surface;
    if (!editable.has(id)) return;
    if (blocked.has(id)) blocked.delete(id); else blocked.add(id);
    write('localStorage', choiceKey, [...blocked]);
    render();
  });
  selector.addEventListener('change', () => {
    if (!(active() && selector.value === 'off')) mode = selector.value;
    saveJob();
  });
  next.addEventListener('click', () => {
    if (mode === 'off') return;
    phase = steps[phase][1];
    failed = false;
    saveJob();
  });
  fail.addEventListener('click', () => {
    if (!active() || mode === 'off') return;
    failed = true;
    saveJob(); // No phase advance and, in particular, no preference write.
  });
  window.addEventListener('storage', event => {
    if (event.key === choiceKey || event.key === null) { blocked = loadChoices(); render(); }
  });
  render();
})();
