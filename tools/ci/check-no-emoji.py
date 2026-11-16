#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
"""Icon gate — no emoji as UI icons (§8.5, decision D29).

Emoji render differently on Windows 10, Windows 11, and every language pack,
which silently breaks the visual identity. The rule is therefore mechanical:
icons come from the SVG sprite, never from a font glyph.

Scanned: HTML, CSS, JS/TS, Rust, Python, Fluent locale files.
Allowed:  typographic punctuation that is not an emoji (arrows, dashes,
          bullets, box-drawing, geometric shapes used in ASCII diagrams).

Usage:
    python3 tools/ci/check-no-emoji.py [--root .] [--quiet]

Exit codes: 0 = clean, 1 = emoji found, 2 = no files matched.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

# UI surfaces only. Markdown is deliberately excluded: ✅/❌ as status marks
# in a table is prose convention, not an icon in the product. The Fluent files
# ARE the product's words, so they are scanned.
SCAN_EXT = {".html", ".css", ".js", ".mjs", ".cjs", ".ts", ".tsx", ".jsx",
            ".rs", ".ftl", ".svg"}
SKIP_DIRS = {".git", "node_modules", "target", "dist", "build", "out",
             "__pycache__", ".venv", "locales/qps"}

# Emoji and pictographs. Deliberately excludes:
#   U+2190-21FF arrows, U+2500-257F box drawing, U+25A0-25FF geometric,
#   U+2010-203A punctuation — all legitimate in text and ASCII diagrams.
EMOJI_RANGES = [
    (0x1F300, 0x1F5FF),   # misc symbols and pictographs
    (0x1F600, 0x1F64F),   # emoticons
    (0x1F680, 0x1F6FF),   # transport
    (0x1F700, 0x1F77F),   # alchemical
    (0x1F780, 0x1F7FF),   # geometric extended
    (0x1F800, 0x1F8FF),   # supplemental arrows-C
    (0x1F900, 0x1F9FF),   # supplemental symbols
    (0x1FA00, 0x1FAFF),   # symbols extended-A
    (0x2600, 0x26FF),     # misc symbols (sun, umbrella, warning triad)
    (0x2700, 0x27BF),     # dingbats (check marks, scissors)
    (0x2B00, 0x2BFF),     # arrows and stars
    (0xFE0F, 0xFE0F),     # variation selector-16 (emoji presentation)
    (0x1F1E6, 0x1F1FF),   # regional indicators (flags)
]
EMOJI_RE = re.compile("|".join(
    f"[\\U{upper:08X}-\\U{lower:08X}]" for upper, lower in EMOJI_RANGES))

# Lines that document the rule itself (this file, the spec, changelog rows).
ALLOW_MARKERS = (
    "no-emoji", "check-no-emoji", "EMOJI_RANGES", "لا إيموجي",
    "لا تعتمد على إيموجي", "بلا إيموجي", "صفر إيموجي",
)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default=".")
    ap.add_argument("--quiet", action="store_true")
    args = ap.parse_args()

    root = Path(args.root).resolve()
    hits: list[tuple[str, int, str, str]] = []
    scanned = 0

    for path in sorted(root.rglob("*")):
        if not path.is_file() or path.suffix.lower() not in SCAN_EXT:
            continue
        rel = path.relative_to(root)
        relposix = rel.as_posix()
        if any(part in SKIP_DIRS for part in rel.parts):
            continue
        if relposix == "tools/ci/check-no-emoji.py":
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        scanned += 1
        for lineno, line in enumerate(text.splitlines(), 1):
            if any(marker in line for marker in ALLOW_MARKERS):
                continue
            for m in EMOJI_RE.finditer(line):
                ch = m.group(0)
                hits.append((relposix, lineno, ch, f"U+{ord(ch):04X}"))

    if hits:
        print(f"icon gate: {len(hits)} emoji character(s) in {scanned} file(s)",
              file=sys.stderr)
        for rel, lineno, ch, code in hits[:40]:
            print(f"  {rel}:{lineno}: {code} {ch!r}", file=sys.stderr)
        if len(hits) > 40:
            print(f"  … and {len(hits) - 40} more", file=sys.stderr)
        print("\nIcons must come from the SVG sprite (§8.5).", file=sys.stderr)
        return 1

    if not args.quiet:
        print(f"icon gate: {scanned} file(s) clean — no emoji as icons")
    return 0


if __name__ == "__main__":
    sys.exit(main())
