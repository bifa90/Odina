#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
"""Disposable corpus + actual stdio host + actual resident. Never logs URLs, hashes of selections or site history.

Optional --chromium uses a NEW Playwright Chromium profile and the real native messaging API.
No HTTP fixture server. No dependency on Playwright for the native witness.
"""

import argparse
import ctypes
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import queue
import shutil
import signal
import socket
import struct
import subprocess as sp
import sys
import tempfile
import threading
import time

ROOT = Path(__file__).resolve().parents[2]
SETUP = ROOT / 'tools/browser-setup.py'
(ORIGIN,) = json.loads((ROOT / 'browser-extension/hosts/chromium.json').read_text())['allowed_origins']


def unix_only():
    # Load before launching any child threads. libseccomp resolves the native syscall numbers.
    lib = ctypes.CDLL('libseccomp.so.2')

    class Comparison(ctypes.Structure):
        _fields_ = [
            ('arg', ctypes.c_uint),
            ('op', ctypes.c_uint),
            ('a', ctypes.c_uint64),
            ('b', ctypes.c_uint64),
        ]

    lib.seccomp_init.argtypes = [ctypes.c_uint32]
    lib.seccomp_init.restype = ctypes.c_void_p
    lib.seccomp_rule_add_array.argtypes = [
        ctypes.c_void_p,
        ctypes.c_uint32,
        ctypes.c_int,
        ctypes.c_uint,
        ctypes.POINTER(Comparison),
    ]
    lib.seccomp_syscall_resolve_name.argtypes = [ctypes.c_char_p]
    lib.seccomp_load.argtypes = [ctypes.c_void_p]
    lib.seccomp_release.argtypes = [ctypes.c_void_p]
    ctx = lib.seccomp_init(0x7FFF0000)  # SCMP_ACT_ALLOW
    if not ctx:
        raise RuntimeError('seccomp init')
    for name in (b'socket', b'socketpair'):
        nr = lib.seccomp_syscall_resolve_name(name)
        cmp = Comparison(0, 1, socket.AF_UNIX, 0)  # SCMP_CMP_NE
        if nr < 0 or lib.seccomp_rule_add_array(ctx, 0x80000000, nr, 1, ctypes.byref(cmp)) != 0:
            raise RuntimeError('seccomp rule')
    if lib.seccomp_load(ctx) != 0:
        raise RuntimeError('seccomp load')
    lib.seccomp_release(ctx)


def guarded(argv, guard):
    # Separate launcher (not Python preexec_fn in a multithreaded process); filter inherited across exec.
    return [sys.executable, str(Path(__file__).resolve()), '--unix-exec', *map(str, argv)] if guard else argv


def run(argv, **kw):
    return sp.run(argv, check=True, capture_output=True, timeout=12, **kw).stdout


def summary(samples):
    ordered = sorted(samples)

    def pct(p):
        return ordered[math.ceil(len(samples) * p) - 1]

    return {
        'n': len(samples),
        'first_ms': samples[0],
        'p50_ms': pct(0.5),
        'p95_ms': pct(0.95),
        'p99_ms': pct(0.99),
        'max_ms': ordered[-1],
        'at_least_100ms': sum(t >= 100 for t in samples),
        'samples_ms': samples,
    }


def native(host, url, *, firefox=False, guard=False, op='inspect', ticket=None):
    body = {'op': op, 'url': url}
    if ticket is not None:
        body['ticket'] = ticket
    data = json.dumps(body).encode()
    argv = [host, *(['manifest', 'context-menu@odina.local'] if firefox else [ORIGIN])]
    t = time.perf_counter_ns()
    reply = run(guarded(argv, guard), input=struct.pack('=I', len(data)) + data)
    assert len(reply) >= 4 and struct.unpack('=I', reply[:4])[0] == len(reply) - 4 <= 128
    result = json.loads(reply[4:])
    return result, (time.perf_counter_ns() - t) / 1e6


def browser_lab(base, urls, host, n):
    from playwright.sync_api import sync_playwright, expect

    assert sys.platform == 'linux', 'isolated registration harness uses the Linux per-user layout'
    profile = base / 'config/chromium'
    ext = base / 'extension'
    env = {
        **os.environ,
        'HOME': str(base),
        'XDG_CONFIG_HOME': str(base / 'config'),
        'XDG_CACHE_HOME': str(base / 'cache'),
    }
    run([sys.executable, SETUP, 'pack', '--browser', 'chromium', '--out', ext])
    run([sys.executable, SETUP, 'install', '--browser', 'chromium', '--binary', host], env=env)
    with sync_playwright() as pw:
        ctx = pw.chromium.launch_persistent_context(
            str(profile),
            channel='chromium',
            headless=True,
            env=env,
            args=[
                f'--disable-extensions-except={ext}',
                f'--load-extension={ext}',
                '--disable-background-networking',
            ],
            offline=True,
        )
        try:
            net = []
            ctx.on(
                'request',
                lambda r: net.append(r.resource_type)
                if r.url.startswith(('https:', 'http:', 'ws:', 'wss:'))
                else None,
            )
            sw = (
                ctx.service_workers[0]
                if ctx.service_workers
                else ctx.wait_for_event('serviceworker', timeout=12000)
            )
            assert sw.url == ORIGIN + 'background.js', sw.url
            local = urls['clean']
            # Call the shipped onClicked listener in its actual worker, not a mocked native API.
            # Timer excludes automation transport, worker wakeup, menu painting and user think time.
            script = """async ([pageUrl, linkUrl]) => {
              const t=performance.now(); const kind=await inspectClick({menuItemId:'odina.inspect',pageUrl,linkUrl});
              return [kind, performance.now()-t];
            }"""
            quiet = len(ctx.pages)
            samples = []
            for _ in range(n):
                kind, dt = sw.evaluate(script, [local, local])
                assert kind == 'clean', 'browser native host was not reached'
                samples.append(dt)
            for remote, selected in [
                (local, 'https://example.invalid/not-fetched'),
                (local, urls['missing']),
                ('https://example.invalid/', local),
            ]:
                assert sw.evaluate(script, [remote, selected])[0] == 'unknown'
            assert len(ctx.pages) == quiet
            with ctx.expect_page(timeout=5000) as opened:
                assert sw.evaluate(script, [local, urls['threat']])[0] == 'threat'
            page = opened.value
            expect(page.locator('#quarantine')).to_be_visible()
            expect(page.locator('#quarantine')).to_be_disabled()
            expect(page.locator('#deep')).to_be_hidden()
            expect(page.locator('#dismiss')).to_be_focused()
            page.locator('#dismiss').click()
            # Suspicion is not produced by this tiny V5 catalog. Drive just those two native responses
            # in the disposable browser; the Rust consent integration separately exercises real frames/reads.
            sw.evaluate(
                """() => {chrome.runtime.sendNativeMessage=async()=>({result:'suspicious',ticket:'abcdef0123456789'})}"""
            )
            for consent in (False, True):
                with ctx.expect_page(timeout=5000) as popup:
                    assert sw.evaluate(script, [local, urls['clean']])[0] == 'suspicious'
                p = popup.value
                expect(p.locator('#deep')).to_be_visible()
                expect(p.locator('#dismiss')).to_be_focused()
                p.evaluate("""() => {globalThis.calls=0; chrome.runtime.sendNativeMessage=async(_,m)=>{
                  if(m.op!=='deep')throw Error(); calls++; return {result:'threat'}; };}""")
                assert p.evaluate('calls') == 0
                if consent:
                    p.locator('#deep').click()  # trusted browser input, no direct .onclick() call
                    expect(p.locator('#quarantine')).to_be_visible()
                    expect(p.locator('#quarantine')).to_be_disabled()
                    assert p.evaluate('calls') == 1
                p.locator('#dismiss').click()
            assert not net, 'unexpected browser request'
            return {
                'engine': sw.evaluate('navigator.userAgent'),
                'timer': 'warm onClicked listener entry to production inspectClick resolution',
                'clean': summary(samples),
                'network_requests_observed_in_context': len(net),
                'offline': True,
                'positive': 'real V5; no block or quarantine performed',
                'consent_UI': 'fixture native replies, trusted buttons',
            }
        finally:
            ctx.close()


def witness(args, base):
    cli, host = args.cli.resolve(strict=True), args.host.resolve(strict=True)
    root = base / 'data'
    shutil.copytree(ROOT / 'tests/fixtures/safe', root)
    note = root / 'positive.bin'
    note.write_bytes(b'X5O!P%@AP[4\\PZX54(P^)7CC)7}$' + b'EICAR-STANDARD-ANTIVIRUS-' + b'TEST-FILE!$H+H*')
    urls = {
        k: p.as_uri()
        for k, p in [
            ('clean', root / 'button.json'),
            ('missing', root / 'missing'),
            ('threat', note),
            ('dir', root),
            ('outside', base / 'elsewhere'),
        ]
    }
    guard = args.deny_network
    if guard:
        for fam in ['AF_INET', 'AF_INET6']:
            cmd = [sys.executable, '-c', 'import socket as s; s.socket(s.' + fam + ')']
            assert sp.run(guarded(cmd, True), cwd=base, timeout=6).returncode == -signal.SIGSYS, (
                'filter control did not fire'
            )
    srv = sp.Popen(
        guarded([cli, 'serve', root, '--service', 'browser'], guard), stdout=sp.PIPE, cwd=base, stderr=sp.PIPE
    )
    try:
        q = queue.Queue()
        threading.Thread(target=lambda: q.put(srv.stdout.readline()), daemon=True).start()
        assert q.get(timeout=10).startswith(b'READY'), (
            'private lab could not bind; do not query an existing service'
        )

        def status():
            return json.loads(run([cli, 'status', '--service', 'browser']))

        deadline = time.monotonic() + 10
        while not status()['watching']:
            if time.monotonic() >= deadline:
                raise TimeoutError('no baseline')
            time.sleep(0.01)
        before = status()
        for case, expected in [
            ('dir', 'threat'),
            ('threat', 'threat'),
            ('clean', 'clean'),
            ('missing', 'unknown'),
            ('outside', 'unknown'),
        ]:
            assert native(host, urls[case], guard=guard)[0] == {'result': expected}, case
        assert native(host, urls['clean'], firefox=True, guard=guard)[0] == {'result': 'clean'}
        for uri in ['https://example.invalid/not-fetched', 'file://server.invalid/share/x']:
            assert native(host, uri, guard=guard)[0] == {'result': 'unknown'}
        assert native(host, urls['threat'], op='deep', ticket='0000000000000000', guard=guard)[0] == {
            'result': 'unknown'
        }
        # Don't time a Python seccomp launcher. The functional guarded run above covers the identical binaries.
        samples = []
        for _ in range(args.samples):
            result, dt = native(host, urls['clean'])
            assert result == {'result': 'clean'}
            samples.append(dt)
        report = {
            'schema': 'odina-browser-lab/v1',
            'platform': platform.platform(),
            'files': 21,
            'cli_sha256': hashlib.sha256(cli.read_bytes()).hexdigest(),
            'host_sha256': hashlib.sha256(host.read_bytes()).hexdigest(),
            'native': {
                'timer': 'before process start to framed JSON decoded after exit',
                'clean': summary(samples),
            },
            'native_socket_guard': 'AF_UNIX only; IPv4/IPv6 positive controls killed'
            if guard
            else 'not measured',
        }
        if args.chromium:
            report['browser'] = browser_lab(base, urls, host, args.samples)
        if args.firefox:
            from firefox_menu import observe

            report['firefox'] = observe(base, urls, host, SETUP)
        after = status()
        assert (
            after['full_scans'] == before['full_scans'] == 1 and after['delta_scans'] == before['delta_scans']
        )
        assert note.stat().st_size == 68
        assert json.loads(run([cli, 'stop', '--service', 'browser']))['stopping'] and srv.wait(timeout=5) == 0
        assert native(host, urls['clean'], guard=guard)[0] == {'result': 'unknown'}, 'offline is not clean'
        return report
    finally:
        if srv.poll() is None:
            srv.kill()
            srv.wait(timeout=5)
        srv.stdout.close()
        srv.stderr.close()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--cli', type=Path, required=True)
    p.add_argument('--host', type=Path, required=True)
    p.add_argument(
        '--firefox', action='store_true', help='Linux desktop menu; needs selenium, geckodriver, firefox'
    )
    p.add_argument('--chromium', action='store_true')
    p.add_argument('--deny-network', action='store_true')
    p.add_argument('--samples', type=int, default=50)
    p.add_argument('--out', type=Path, required=True)
    args = p.parse_args()
    if not 1 <= args.samples <= 10000 or args.out.exists():
        p.error('choose 1–10000 samples and a new output file')
    if args.deny_network and sys.platform != 'linux':
        p.error('--deny-network requires Linux libseccomp')
    with tempfile.TemporaryDirectory(prefix='odina-browser-lab-') as scratch:
        result = witness(args, Path(scratch).resolve())
    with args.out.open('x', encoding='utf-8') as f:
        json.dump(result, f, indent=2)
        f.write('\n')
    for group in ['native', 'browser']:
        if group in result:
            stats = {k: v for k, v in result[group]['clean'].items() if k != 'samples_ms'}
            print(group, stats)
            if os.environ.get('GITHUB_ACTIONS'):
                print(
                    f'::notice title=Quick path {group}::{result[group].get("engine", platform.system())} {stats}'
                )
    if 'firefox' in result:
        print(
            ('::notice title=Firefox menu::' if os.environ.get('GITHUB_ACTIONS') else '')
            + str(result['firefox'])
        )
    # This is a strict gate for THIS sample set; don't drop scheduler outliers.
    return int(any(result[g]['clean']['at_least_100ms'] for g in ['browser', 'native'] if g in result))


if __name__ == '__main__':
    if sys.argv[1:2] == ['--unix-exec']:
        unix_only()
        os.execv(sys.argv[2], sys.argv[2:])
    sys.exit(main())
