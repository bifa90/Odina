#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
"""Copy the four woff2 files the window draws with into the shell bundle.

The window must render with no network and with the same faces the preview was
approved with, so it carries the files rather than asking a CDN. Only the four
that `app.css` names are copied: the repository keeps 682 font files for the
site, and a desktop installer should not weigh what a web page needs.
"""

import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / 'fonts' / 'vendor'
DST = ROOT / 'src-tauri' / 'ui' / 'fonts'

FILES = [
    'inter/inter-latin-400-normal.woff2',
    'inter/inter-latin-600-normal.woff2',
    'ibm-plex-sans-arabic/ibm-plex-sans-arabic-arabic-400-normal.woff2',
    'ibm-plex-sans-arabic/ibm-plex-sans-arabic-arabic-600-normal.woff2',
]


def main() -> int:
    DST.mkdir(parents=True, exist_ok=True)
    for name in FILES:
        src = SRC / name
        if not src.is_file():
            print(f'missing font: {name}')
            return 1
        shutil.copyfile(src, DST / src.name)
        shutil.copyfile(SRC / name.split('/')[0] / 'LICENSE', DST / f'{name.split("/")[0]}-LICENSE')
    print(f'copied {len(FILES)} font files into {DST.relative_to(ROOT)}')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
