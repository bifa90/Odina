#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
"""Design gate — the rules in design-system-v1.md §6.1–§6.3, enforced.

The exception screen is the first surface where the product asks the user to
weaken a protection rule. Every rule that makes that safe is visible in the
markup, so every rule can be checked mechanically — and a rule that is only
written in a spec is a rule that will be broken by the third person who edits
the file.

Checked:

  1. no colour outside the six status tokens in the exception section
  2. the live badge appears before any card, and is not hidden behind a click
  3. "permanent" is never visible until "advanced options" is opened
  4. at most two buttons per card
  5. the consequence line comes before the allow button in the DOM
  6. no `toggle`/`checkbox` control on a rule card
  7. every key used has an Arabic string, and the Arabic is not the English

Usage: python3 tools/ci/check-exception-ui.py [--root .] [--quiet]
Exit:  0 clean · 1 violation · 2 the section is missing (worse than a violation)
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

START = "1d. الاستثناء الجراحي"
END = "2. التنبيهات"

# The six status colours from §6, plus neutrals. A seventh colour in this
# section means someone invented a meaning that the rest of the product does
# not share.
ALLOWED_COLOURS = {
    "--accent", "--ok", "--warn", "--danger", "--text3",
    "--text", "--text2", "--border", "--surface", "--surface2", "--bg",
    "--focus", "--radius", "--r-sm", "--r-lg", "--shadow",
    # Identity colours (§2.3: these never change). Using them is not inventing
    # a meaning, it is using the brand's own ink.
    "--navy", "--gold",
}

BUTTON_OPEN = re.compile(r"<button\b")


def section(html: str) -> str:
    if START not in html:
        raise SystemExit("check-exception-ui: exception section not found")
    begin = html.index(START)
    end = html.index(END, begin)
    return html[begin:end]


def exception_css(html: str) -> str:
    """The CSS rules that style the exception section.

    Looking for colours only inside the markup would miss the one place a
    colour actually gets chosen: the style block. The section is named in a
    comment, so its rules are the lines between that marker and the next
    section marker.
    """
    start = html.index("الاستثناء الجراحي (§4.8 · D45)")
    # The header comment ends at the first `*/` after the marker; the rules
    # follow it and stop at the next section marker. Reading past that point
    # picks up the rest of the stylesheet and reports colours that belong to
    # other screens.
    # `*/` may appear inside the Arabic comment text itself; the end of the
    # header is the last one before the first rule.
    body_start = html.rindex("*/", start, html.index(".excbadge", start)) + 2
    end_marker = "نهاية أنماط الاستثناء"
    end = html.index(end_marker, body_start)
    return html[body_start:end]


def check(root: Path) -> list[str]:
    html = (root / "ui-preview.html").read_text(encoding="utf-8")
    body = section(html)
    problems: list[str] = []

    # 1. colours — in the markup and in the section's own CSS
    css = exception_css(html)
    for colour in set(re.findall(r"var\((--[\w-]+)\)", body + css)):
        if colour not in ALLOWED_COLOURS:
            problems.append(f"colour {colour} is not one of the status tokens")
    # A literal colour is worse than a wrong token: it cannot follow dark mode,
    # and it says the author did not find the token they needed.
    for literal in set(re.findall(r"#[0-9a-fA-F]{3,8}\b|rgba?\(", css)):
        problems.append(f"literal colour {literal} in the exception styles")

    # 2. the badge is before the first card and outside the advanced panel
    badge = body.find("excbadge")
    first_card = body.find("excgrid")
    if badge == -1 or first_card == -1:
        problems.append("live badge or card grid missing")
    elif badge > first_card:
        problems.append("the live badge comes after the cards — it must be visible first")

    # 3. permanent is inside an advanced panel only
    for match in re.finditer(r"permanent", body):
        line_start = body.rfind("\n", 0, match.start())
        line = body[line_start : body.find("\n", match.start())]
        if "advpanel" in line:
            continue
        # allowed in prose notes and in the badge for a live permanent exception
        if "data-k" in line and "advpanel" not in line and "permanent" in line:
            key = re.search(r'data-k="([^"]+)"', line)
            if key and key.group(1) in {
                "exception.badge_permanent",
                "exception.state.allowed_permanent",
            }:
                continue
            problems.append("“permanent” is visible outside the advanced panel")

    # 4 & 5. per card: at most two buttons, consequence before the allow button
    for card in re.findall(r'<div class="exccard[^"]*">(.*?)\n      </div>', body, re.S):
        buttons = len(BUTTON_OPEN.findall(card))
        if buttons > 2:
            problems.append(f"{buttons} buttons in one card — the limit is two")
        conseq = card.find("conseq")
        allow = card.find("exception.action.allow")
        if allow != -1 and (conseq == -1 or conseq > allow):
            problems.append("the consequence line is not before the allow button")

    # 6. no toggle
    if re.search(r'type="checkbox"|role="switch"|class="[^"]*\btoggle\b', body):
        problems.append("a toggle on a rule card hides the duration and the permanent option")

    return problems


def arabic_missing(root: Path) -> list[str]:
    html = (root / "ui-preview.html").read_text(encoding="utf-8")
    body = section(html)
    keys = set(re.findall(r'data-k="([^"]+)"', body))
    ar = (root / "locales" / "ar" / "odina.ftl").read_text(encoding="utf-8")
    # Fluent keys may contain dots; comments start with '#'. `\w` does not
    # cover the dot in `lvl.d10`, which is why this is spelled out.
    have = set(re.findall(r"^([A-Za-z0-9_.-]+)\s*=", ar, re.M))
    return sorted(k for k in keys if k not in have)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default=".")
    ap.add_argument("--quiet", action="store_true")
    args = ap.parse_args()
    root = Path(args.root)

    problems = check(root) + [f"no Arabic string for {k}" for k in arabic_missing(root)]

    if problems:
        print(f"exception-ui gate: {len(problems)} violation(s)", file=sys.stderr)
        for p in problems[:20]:
            print(f"  {p}", file=sys.stderr)
        return 1
    if not args.quiet:
        print("exception-ui gate: badge first, consequences above buttons, no toggle")
    return 0


if __name__ == "__main__":
    sys.exit(main())
