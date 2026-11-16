#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
"""Contrast gate — WCAG 2.2 check on the measured palette.

Reads the token table straight out of docs/spec/design-system-v1.md so the
document cannot drift from what CI enforces. If someone edits a colour in
the prose without measuring it, this fails.

Rules (WCAG 2.2):
  * normal text        >= 4.5:1
  * large text (>=24px / 18.66px bold) and UI graphics >= 3.0:1
  * text on a filled button is checked against that button's background

Usage:
    python3 tools/ci/check-contrast.py [--doc PATH] [--quiet]

Exit codes: 0 = pass, 1 = failure, 2 = could not parse the doc.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

HEX = re.compile(r"#[0-9A-Fa-f]{6}")

# Tokens that are allowed to sit below 4.5:1, with the ratio they must clear.
# text3 is a hint colour: 12px is not our use, we only allow it at >=3:1
# and the design system marks it "large text only".
MIN_NORMAL = 4.5
MIN_LARGE = 3.0


def srgb_to_lin(c: float) -> float:
    c = c / 255.0
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def luminance(hex_color: str) -> float:
    h = hex_color.lstrip("#")
    r, g, b = (int(h[i:i + 2], 16) for i in (0, 2, 4))
    return (0.2126 * srgb_to_lin(r)
            + 0.7152 * srgb_to_lin(g)
            + 0.0722 * srgb_to_lin(b))


def contrast(a: str, b: str) -> float:
    la, lb = luminance(a), luminance(b)
    hi, lo = max(la, lb), min(la, lb)
    return (hi + 0.05) / (lo + 0.05)


def palette_from_doc(path: Path) -> dict[str, dict[str, str]]:
    """Pull `| --name | #RRGGBB |` rows, split by the light/dark sections."""
    text = path.read_text(encoding="utf-8")
    themes: dict[str, dict[str, str]] = {}
    current = None
    for line in text.splitlines():
        if line.startswith("###"):
            if "الوضع الفاتح" in line:
                current = "light"
            elif "الوضع الداكن" in line:
                current = "dark"
            elif "الهوية" in line:
                current = None
            continue
        if current is None:
            continue
        m = re.match(r"\|\s*`(--[a-z0-9]+)`\s*\|\s*`(#[0-9A-Fa-f]{6})`", line)
        if m:
            themes.setdefault(current, {})[m.group(1)] = m.group(2).upper()
    return themes


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--doc", default="docs/spec/design-system-v1.md")
    ap.add_argument("--quiet", action="store_true")
    args = ap.parse_args()

    doc = Path(args.doc)
    if not doc.exists():
        print(f"contrast gate: {doc} not found", file=sys.stderr)
        return 2

    themes = palette_from_doc(doc)
    if not themes.get("light") or not themes.get("dark"):
        print("contrast gate: could not parse both palettes from the doc",
              file=sys.stderr)
        return 2

    failures: list[str] = []
    checks = 0

    for theme, tokens in themes.items():
        bg = tokens.get("--bg")
        surface = tokens.get("--surface")
        if not bg:
            failures.append(f"{theme}: no --bg token")
            continue
        for name, color in tokens.items():
            if name in {"--bg", "--surface", "--surface2"}:
                continue
            # Ghost text over the page background, and also over a card.
            for base_name, base in (("bg", bg), ("surface", surface)):
                if not base:
                    continue
                ratio = contrast(color, base)
                checks += 1
                if name == "--text3":
                    need = MIN_LARGE
                else:
                    need = MIN_NORMAL
                if ratio < need:
                    failures.append(
                        f"{theme}: {name} {color} on {base_name} {base} "
                        f"= {ratio:.2f}:1 (needs {need}:1)")

    # Buttons: text colour must reach 4.5:1 on each filled rank.
    button_text = {"light": "#FFFFFF", "dark": "#0B1416"}
    for theme, tokens in themes.items():
        for rank in ("--accent", "--danger"):
            fill = tokens.get(rank)
            if not fill:
                continue
            fg = button_text.get(theme)
            if not fg:
                continue
            ratio = contrast(fg, fill)
            checks += 1
            if ratio < MIN_NORMAL:
                failures.append(
                    f"{theme}: button text {fg} on {rank} {fill} "
                    f"= {ratio:.2f}:1 (needs {MIN_NORMAL}:1)")

    if failures:
        print(f"contrast gate: {len(failures)} failure(s) in {checks} check(s)",
              file=sys.stderr)
        for f in failures:
            print(f"  {f}", file=sys.stderr)
        return 1

    if not args.quiet:
        print(f"contrast gate: {checks} pair(s) pass "
              f"(light + dark, text, hints, buttons)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
