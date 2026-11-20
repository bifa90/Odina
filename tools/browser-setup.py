#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
"""Offline, per-user packaging/registration. No elevation, network, policy or browser preference changes."""

import argparse
import json
import os
from pathlib import Path
import runpy
import shutil
import sys

ROOT = Path(__file__).resolve().parent.parent
EXT = ROOT / 'browser-extension'
NAME = 'org.odina.context'
VENDORS = {
    'chrome': ('google-chrome', r'Google\Chrome'),
    'chromium': ('chromium', 'Chromium'),
    'edge': ('microsoft-edge', r'Microsoft\Edge'),
    'firefox': ('mozilla', 'Mozilla'),
}
WIN = sys.platform == 'win32'


def catalogs():
    source = runpy.run_path(str(ROOT / 'tools/gen-locale-js.py'))
    for lang in source['ORDER']:
        table = source['parse_ftl'](ROOT / 'locales' / lang / 'odina.ftl')
        items = {k.split('.', 1)[1]: {'message': v} for k, v in table.items() if k.startswith('browser.')}
        items['language'] = {'message': lang}
        yield {'zh-Hans': 'zh_CN', 'pt-BR': 'pt_BR'}.get(lang, lang), items


def json_text(value):
    return json.dumps(value, indent=2, ensure_ascii=False) + '\n'


def new_file(path, text, mode=0o600):
    path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    # Never truncate an existing manifest or follow a destination link.
    with os.fdopen(
        os.open(path, os.O_CREAT | os.O_EXCL | os.O_WRONLY, mode), 'w', encoding='utf-8', newline='\n'
    ) as f:
        f.write(text)
    return path


def pack(browser, dest):
    dest.mkdir(mode=0o700, parents=True, exist_ok=False)
    for f in ('background.js', 'notice.html', 'notice.css', 'notice.js'):
        shutil.copyfile(EXT / f, dest / f)
    source = 'manifest.firefox.json' if browser == 'firefox' else 'manifest.json'
    shutil.copyfile(EXT / source, dest / 'manifest.json')
    for locale, messages in catalogs():
        new_file(dest / '_locales' / locale / 'messages.json', json_text(messages))
    return dest


def location(browser):
    if WIN:
        return (
            Path(os.environ['LOCALAPPDATA']) / 'Odina' / 'NativeMessagingHosts' / browser / (NAME + '.json')
        )
    if browser == 'firefox':
        return Path.home() / '.mozilla/native-messaging-hosts' / (NAME + '.json')
    config = Path(os.environ.get('XDG_CONFIG_HOME') or Path.home() / '.config')
    return config / VENDORS[browser][0] / 'NativeMessagingHosts' / (NAME + '.json')


def reg_key(browser):
    return 'Software\\' + VENDORS[browser][1] + '\\NativeMessagingHosts\\' + NAME


def reg_get(key, name=''):
    import winreg as w

    try:
        with w.OpenKey(w.HKEY_CURRENT_USER, key) as k:
            return w.QueryValueEx(k, name)[0]
    except FileNotFoundError:
        return None


def reg_set(key, name, value):
    import winreg as w

    with w.CreateKey(w.HKEY_CURRENT_USER, key) as k:
        w.SetValueEx(k, name, 0, w.REG_SZ, value)


def reg_delete(key):
    import winreg as w

    w.DeleteKey(w.HKEY_CURRENT_USER, key)


def binary_path(raw):
    p = raw.resolve(strict=True)
    if not p.is_file() or (WIN and p.suffix.lower() != '.exe') or not os.access(p, os.X_OK):
        raise ValueError('expected a built, executable odina-browser-host')
    return p


def register(browser, binary=None):
    dest = location(browser)
    template = json.loads(
        (EXT / 'hosts' / ('firefox.json' if browser == 'firefox' else 'chromium.json')).read_text()
    )
    key = reg_key(browser)
    if binary is not None:
        if WIN and reg_get(key) is not None:
            raise ValueError('registration already exists; uninstall it first')
        template['path'] = str(binary_path(binary))
        new_file(dest, json_text(template))
        if WIN:
            try:
                reg_set(key, '', str(dest))
            except OSError:
                dest.unlink()
                raise
    else:
        if dest.is_symlink():
            raise ValueError('not removing a linked manifest')
        doc = json.loads(dest.read_text(encoding='utf-8'))
        if {**doc, 'path': template['path']} != template:
            raise ValueError('not an Odina installer manifest')
        if WIN:
            if reg_get(key) != str(dest):
                raise ValueError('not our registry entry')
            reg_delete(key)
        dest.unlink()
    return dest


def main():
    p = argparse.ArgumentParser(description=__doc__)
    subs = p.add_subparsers(dest='action', required=True)
    for action in ('pack', 'install', 'uninstall', 'check-locales'):
        s = subs.add_parser(action)
        if action in ('pack', 'install', 'uninstall'):
            s.add_argument('--browser', choices=VENDORS, required=True)
        if action == 'pack':
            s.add_argument('--out', type=Path, required=True, help='NEW directory')
        if action == 'install':
            s.add_argument('--binary', type=Path, required=True)
    args = p.parse_args()
    try:
        if args.action == 'pack':
            print(pack(args.browser, args.out))
        elif args.action == 'check-locales':
            for locale, messages in catalogs():
                p = EXT / '_locales' / locale / 'messages.json'
                if p.read_text(encoding='utf-8') != json_text(messages):
                    raise ValueError(f'browser catalog {locale} differs from its Fluent source')
        elif WIN or sys.platform == 'linux':
            print(register(args.browser, getattr(args, 'binary', None)))
        else:
            raise ValueError('resident supported on Linux and Windows only')
    except (OSError, ValueError) as e:
        p.exit(1, f'Odina: {e}\n')


if __name__ == '__main__':
    main()
