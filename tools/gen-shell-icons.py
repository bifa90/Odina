#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
"""Render the shell icons from the approved 1024 px logo, as RGBA.

Tauri refuses a palette or RGB PNG for an icon, and the site assets are RGBA
only by accident. This keeps the window's icons reproducible from the logo
instead of being a folder nobody can regenerate.
"""

import pathlib
import sys

try:
    from PIL import Image
except ImportError:  # pragma: no cover - a developer tool, not a gate
    sys.exit('pillow is required: pip install pillow')

ROOT = pathlib.Path(__file__).resolve().parents[1]
SRC = ROOT / 'assets' / 'logo' / 'png' / 'dark' / 'odina-1024.png'
DST = ROOT / 'src-tauri' / 'icons'

SIZES = [(32, '32x32.png'), (128, '128x128.png'), (256, '128x128@2x.png'), (512, 'icon.png')]


def main() -> int:
    logo = Image.open(SRC).convert('RGBA')
    DST.mkdir(parents=True, exist_ok=True)
    for size, name in SIZES:
        logo.resize((size, size), Image.LANCZOS).save(DST / name)
    logo.resize((256, 256), Image.LANCZOS).save(
        DST / 'icon.ico', sizes=[(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    )
    print(f'wrote {len(SIZES) + 1} icons into {DST.relative_to(ROOT)}')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
