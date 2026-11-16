#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)

"""Regenerate fonts/manifests/*.json from fonts/vendor/*.

Run after updating any font package. Every file gets name + size + sha256 so the
application can reject tampered or truncated font files (see docs/cdn-fonts.md).
"""
import glob
import hashlib
import json
import os

META = {
    'inter': dict(family='Inter', lang=['en', 'es', 'pt-BR', 'de', 'fr'],
                  mode='bundled', upstream='@fontsource/inter@5.x', license='OFL-1.1'),
    'ibm-plex-sans-arabic': dict(family='IBM Plex Sans Arabic', lang=['ar'],
                                 mode='bundled', upstream='@fontsource/ibm-plex-sans-arabic@5.x',
                                 license='OFL-1.1'),
    'pt-sans': dict(family='PT Sans', lang=['ru'], mode='bundled',
                    upstream='@fontsource/pt-sans@5.x', license='OFL-1.1',
                    note='no 600 weight upstream; 700 used for semibold'),
    'noto-sans-devanagari': dict(family='Noto Sans Devanagari', lang=['hi'],
                                 mode='bundled', upstream='@fontsource/noto-sans-devanagari@5.x',
                                 license='OFL-1.1'),
    'noto-sans-sc': dict(family='Noto Sans SC', lang=['zh-Hans'], mode='cdn',
                         upstream='@fontsource/noto-sans-sc@5.3.1', license='OFL-1.1'),
    'noto-sans-tc': dict(family='Noto Sans TC', lang=['zh-Hant'], mode='cdn',
                         upstream='@fontsource/noto-sans-tc@5.3.1', license='OFL-1.1'),
    'noto-sans-jp': dict(family='Noto Sans JP', lang=['ja'], mode='cdn',
                         upstream='@fontsource/noto-sans-jp@5.3.1', license='OFL-1.1'),
}

VENDOR = os.path.join(os.path.dirname(__file__), '..', 'fonts', 'vendor')
MANIFESTS = os.path.join(os.path.dirname(__file__), '..', 'fonts', 'manifests')


def sha256(path):
    h = hashlib.sha256()
    with open(path, 'rb') as f:
        for chunk in iter(lambda: f.read(1 << 20), b''):
            h.update(chunk)
    return h.hexdigest()


def main():
    os.makedirs(MANIFESTS, exist_ok=True)
    index = {'schema': 'odina-fonts/v1', 'generated': '2026-11-15', 'packs': []}
    tot_b = tot_c = 0

    for pkg, meta in META.items():
        d = os.path.join(VENDOR, pkg)
        if not os.path.isdir(d):
            print(f'  skip (missing): {pkg}')
            continue
        files = []
        for p in sorted(glob.glob(os.path.join(d, '*.woff2'))):
            files.append({'file': os.path.basename(p),
                          'bytes': os.path.getsize(p),
                          'sha256': sha256(p)})
        size = sum(f['bytes'] for f in files)
        if meta['mode'] == 'bundled':
            tot_b += size
        else:
            tot_c += size
        index['packs'].append(dict(pack=pkg, **meta,
                                   file_count=len(files),
                                   total_bytes=size,
                                   files=files))
        print(f"  {pkg:22s} {meta['mode']:8s} {len(files):4d} files {size/1048576:7.2f} MB")

    index['totals'] = {'bundled_bytes': tot_b, 'cdn_bytes': tot_c, 'all_bytes': tot_b + tot_c}
    with open(os.path.join(MANIFESTS, 'index.json'), 'w', encoding='utf-8') as f:
        json.dump(index, f, indent=2, ensure_ascii=False)
    print(f"\nbundled={tot_b/1048576:.2f}MB  cdn={tot_c/1048576:.2f}MB "
          f"total={(tot_b+tot_c)/1048576:.2f}MB")


if __name__ == '__main__':
    main()
