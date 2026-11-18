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
  const screen = document.getElementById('screen-remediation');
  const byId = id => document.getElementById(id);
  const choiceKey = 'odina.preview.surface-blocks.v1';
  const jobKey = 'odina.preview.remediation.v2';
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
  const previous = saved ? null : read('sessionStorage', 'odina.preview.remediation.v1');
  const storedMode = (saved || previous)?.mode;
  let mode = ['on', 'muted', 'off'].includes(storedMode) ? storedMode : 'on';
  // No old demo completion proves the new neutralization/verification steps.
  const sequence = ['scan', 'neutralize', 'clean', 'repair', 'verify'];
  const phases = [...sequence, 'restored'];
  let phase = phases.includes(saved?.phase) ? saved.phase :
    sequence.includes(previous?.phase) ? 'scan' : 'ready';
  const active = () => sequence.includes(phase);
  let failed = active() && saved?.failed === true;
  let foundWhileMuted = saved?.foundWhileMuted === true;
  const reasons = new Map([
    ['recoverable', null], ['standalone', 'q.reason1'], ['unmatched', 'recovery.no_rule'],
    ['missing', 'recovery.no_db'], ['invalid', 'recovery.bad_verify'],
  ]);
  let example = reasons.has(saved?.example) ? saved.example : 'recoverable';
  const scenario = byId('recovery-case');
  const advances = [byId('demo-next'), byId('recovery-next')];
  const fail = byId('demo-fail');
  const selector = byId('demo-mode');
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
    screen.dataset.phase = phase;
    scenario.value = example;
    scenario.disabled = hold || off;
    const status = 'proactive.demo.' + (off ? 'off' : phase);
    for (const next of advances) {
      next.disabled = off;
      bind(next, hold ? 'recovery.next' : 'proactive.demo.start');
    }
    fail.hidden = off || !hold;
    fail.disabled = failed;
    for (const id of ['remediation-error', 'recovery-error']) byId(id).hidden = !failed || off;
    for (const id of ['remediation-status', 'recovery-status']) bind(byId(id), status);
    for (const step of screen.querySelectorAll('[data-step]')) {
      if (step.dataset.step === phase) step.setAttribute('aria-current', 'step');
      else step.removeAttribute('aria-current');
    }
    // A simulated durable file result may appear at VERIFY; all nine blocks
    // still apply. Only the *incident's* clean verification restores choices.
    const resolved = phase === 'verify' || phase === 'restored';
    const why = reasons.get(example);
    const isolated = resolved && !!why;
    byId('recovery-result').hidden = !resolved;
    bind(byId('file-result'), isolated ? 'recovery.file_isolated' : 'recovery.file_ok');
    byId('file-reason').hidden = !isolated;
    if (isolated) {
      bind(byId('file-reason'), why);
      bind(byId('q-demo-reason'), why);
    }
    byId('recovery-vault').hidden = !isolated;
    byId('q-demo').hidden = !isolated;
    const count = document.querySelector('.appbar [data-goto="quarantine"] [data-numeral]');
    count.dataset.numeral = isolated ? '3' : '2';
    count.textContent = formatNumber(Number(count.dataset.numeral));
    bind(byId('repair-db-status'), example === 'missing' ? 'recovery.no_db' : 'settings.repair_db.status');
    const summary = byId('pause-demo');
    summary.hidden = !(foundWhileMuted && mode === 'on');
    const report = summary.querySelector('[data-tmpl]');
    report.dataset.kArgs = JSON.stringify({restored: Number(resolved && !why), isolated: Number(isolated), pending: Number(!resolved)});
    bind(report, 'pause.resume_report');
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
    write('sessionStorage', jobKey, {mode, phase, failed, example, foundWhileMuted});
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
  scenario.addEventListener('change', () => {
    if (!active() && mode !== 'off' && reasons.has(scenario.value)) {
      example = scenario.value;
      phase = 'ready';
      failed = foundWhileMuted = false;
    }
    saveJob();
  });
  for (const next of advances) next.addEventListener('click', () => {
    if (mode === 'off') return;
    if (!active()) foundWhileMuted = mode === 'muted';
    phase = phases[sequence.indexOf(phase) + 1];
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
