# SPDX-License-Identifier: MIT
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
"""Firefox desktop menu witness, disposable profile only. No HTTP test site.

WebDriver's loopback test control channel is not part of Odina. Browser chrome context
is used only to activate the same menu item a person selects; do not attach to a user profile.
"""

import html
import json
import os
import shutil
import subprocess as sp
import sys
import zipfile

from selenium import webdriver
from selenium.webdriver import ActionChains
from selenium.webdriver.common.by import By
from selenium.webdriver.firefox.service import Service
from selenium.webdriver.support.ui import WebDriverWait


def observe(base, urls, host, setup):
    ext = base / 'firefox-extension'
    env = {**os.environ, 'HOME': str(base), 'MOZ_HEADLESS': '1'}
    sp.run([sys.executable, setup, 'pack', '--browser', 'firefox', '--out', ext], check=True)
    sp.run([sys.executable, setup, 'install', '--browser', 'firefox', '--binary', host], env=env, check=True)
    # Observer in this private COPY only. No delays, fake native replies or shipping timing API.
    bg = ext / 'background.js'
    listener = 'api.contextMenus.onClicked.addListener(inspectClick);'
    source = bg.read_text(encoding='utf-8')
    assert source.count(listener) == 1
    received = "const response = await api.runtime.sendNativeMessage(host, {op: 'inspect', url});"
    assert source.count(received) == 1
    source = source.replace(
        received, received + ' globalThis.labNative=performance.timeOrigin+performance.now();'
    )
    bg.write_text(
        source.replace(
            listener,
            """const witness=new BroadcastChannel('lab-menu-clock');
      api.contextMenus.onClicked.addListener(async info=>{
        globalThis.labNative=null; const entered=performance.timeOrigin+performance.now(); const kind=await inspectClick(info);
        witness.postMessage({kind,entered,native:globalThis.labNative,done:performance.timeOrigin+performance.now()});
      });""",
        ),
        encoding='utf-8',
    )
    (ext / 'clock.html').write_text('<output></output><script src="clock.js"></script>', encoding='utf-8')
    (ext / 'clock.js').write_text(
        "const c=new BroadcastChannel('lab-menu-clock'); c.onmessage=e=>document.querySelector('output').textContent=JSON.stringify(e.data);",
        encoding='utf-8',
    )
    addon = base / 'addon.xpi'
    with zipfile.ZipFile(addon, 'x') as z:
        for p in ext.rglob('*'):
            if p.is_file():
                z.write(p, p.relative_to(ext))
    urls = {**urls, 'remote': 'https://example.invalid/never-opened'}
    index = base / 'links.html'
    index.write_text(
        ' '.join(f'<a id="{k}" href="{html.escape(v, quote=True)}">{k}</a>' for k, v in urls.items()),
        encoding='utf-8',
    )
    gecko = shutil.which('geckodriver')
    firefox = shutil.which('firefox')
    if not gecko or not firefox:
        raise RuntimeError('requires desktop Firefox and geckodriver on PATH')
    opts = webdriver.FirefoxOptions()
    opts.binary_location = firefox
    opts.add_argument('-headless')
    opts.set_preference('intl.locale.requested', 'en-US')
    service_args = []
    if '--allow-system-access' in sp.check_output([gecko, '--help'], text=True):
        service_args.append('--allow-system-access')
    else:
        opts.add_argument('--remote-allow-system-access')
    with webdriver.Firefox(
        service=Service(gecko, env=env, service_args=service_args), options=opts
    ) as driver:
        driver.install_addon(str(addon), temporary=True)
        driver.set_context('chrome')
        driver.execute_script('Services.io.manageOfflineStatus=false; Services.io.offline=true;')
        uuid = driver.execute_script(
            'return JSON.parse(Services.prefs.getStringPref("extensions.webextensions.uuids"))["context-menu@odina.local"]'
        )
        assert uuid
        driver.set_context('content')
        driver.get(index.as_uri())
        fixture = driver.current_window_handle
        driver.switch_to.new_window('tab')
        reader = driver.current_window_handle
        driver.get(f'moz-extension://{uuid}/clock.html')
        wait = WebDriverWait(driver, 8, poll_frequency=0.05)
        records = []
        for case, expected in [
            ('clean', 'clean'),
            ('clean', 'clean'),
            ('missing', 'unknown'),
            ('remote', 'unknown'),
            ('dir', 'threat'),
        ]:
            driver.execute_script("document.querySelector('output').textContent=''")
            driver.switch_to.window(fixture)
            ActionChains(driver).context_click(driver.find_element(By.ID, case)).perform()
            driver.set_context('chrome')
            select = """const menu=document.getElementById('contentAreaContextMenu');
              const items=[...menu.querySelectorAll('menuitem')].filter(i=>i.label==='Scan with Odina' && !i.hidden);
              if(menu.state!=='open' || !items.length) return null;
              if(items.length!==1) throw Error('multiple Odina choices');
              const start=performance.timeOrigin+performance.now();menu.activateItem(items[0]);return start;"""
            chosen = wait.until(
                lambda d: d.execute_script(select), message='one Firefox menu item on local link'
            )
            driver.set_context('content')
            driver.switch_to.window(reader)
            receipt = json.loads(
                wait.until(
                    lambda d: d.execute_script("return document.querySelector('output').textContent"),
                    message='actual menu decision',
                )
            )
            assert receipt['kind'] == expected, (case, receipt['kind'])
            assert chosen <= receipt['entered'] <= receipt['done'], 'incomparable browser clocks'
            native = receipt['native']
            assert (native is None) == (case == 'remote')
            if native is not None:
                assert receipt['entered'] <= native <= receipt['done']
            records.append(
                {
                    'case': case,
                    'before_listener_ms': round(receipt['entered'] - chosen, 3),
                    'listener_to_native_ms': None
                    if native is None
                    else round(native - receipt['entered'], 3),
                    'result': expected,
                    'dispatch_to_reply_ms': round(receipt['done'] - chosen, 3),
                    'listener_to_reply_ms': round(receipt['done'] - receipt['entered'], 3),
                }
            )
        # Only the confirmed directory result should have opened a warning.
        (popup,) = set(driver.window_handles) - {fixture, reader}
        driver.switch_to.window(popup)
        wait.until(lambda d: d.find_element(By.ID, 'quarantine').is_displayed())
        assert not driver.find_element(By.ID, 'quarantine').is_enabled()
        driver.find_element(By.ID, 'dismiss').click()
        return {'version': driver.capabilities['browserVersion'], 'menu_entries': 1, 'samples': records}
