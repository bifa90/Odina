#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
"""doc-lint — the eighth gate (D99).

The prompt and the decision register must agree with each other and with the
owner's decisions. The failure mode this gate exists to prevent is the one v18
had: the same number stated in four places with four different values, and a
decision quietly contradicted somewhere else in the same file.

Four checks, in increasing subtlety:

  1. anchor    every ``]( #foo )`` link targets an ``id="foo"``.
  2. decisions every ``D##`` cited exists in decisions.md, and D## has no gaps.
  3. canon     every budgeted quantity carries exactly one value, and that value
               is the canonical one below (which is what §2 promises the user).
  4. retired   numbers the owner retired (40MB installer, 120MB database, 0% CPU
               idle, "18 weeks", review batches §20.5) must not reappear.

Check 4 is the important one: it makes a decision enforceable instead of a note.

Usage:
    python3 tools/ci/check-doc-consistency.py [--quiet]

Exit codes: 0 = clean, 1 = violations, 2 = usage error.
"""

from __future__ import annotations

import argparse
import re
import sys
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PROMPT = ROOT / "docs" / "odina-agent-prompt-v19.md"
DECISIONS = ROOT / "docs" / "decisions.md"

# ---------------------------------------------------------------------------
# 3. canonical budgets — one value each, the value §2 promises the user.
#    (key, human label, canonical, unit, regex that must capture that value)
#    Every regex is deliberately narrow: it must anchor on the label, not on a
#    bare "N MB", otherwise "20MB RAM" would read as an installer budget.
# ---------------------------------------------------------------------------
CANON = [
    ("ram_idle",   "RAM idle",           "20",  "MB", r"RAM\s+خمول\s*<?\s*(?P<v>[\d.]+)\s*MB"),
    ("cpu_idle",   "CPU idle",           "0.1", "%",  r"CPU\s+خمول\s*(?:≤|<)\s*(?P<v>[\d.]+)\s*%"),
    ("cpu_scan_bg","CPU background scan","3",   "%",  r"CPU\s+فحص\s+خلفي\s*<(?P<v>[\d.]+)\s*%"),
    ("cpu_scan_usb","CPU USB scan peak", "5",   "%",  r"CPU\s+فحص\s+USB\s*≤\s*(?P<v>[\d.]+)\s*%\s*ذروة"),
    ("flash_ram_ex","USB RAM added",     "30",  "MB", r"≤\s*(?P<v>[\d.]+)\s*MB\s+إضافية"),
    ("flash_ram_pk","USB RAM peak",      "50",  "MB", r"≤\s*(?P<v>[\d.]+)\s*MB\s+ذروة"),
    ("installer",  "installer target",   "25",  "MB",
     (r"هدف\s+المُثبِّت\s*(?:≤\s*)?(?P<v>[\d.]+)\s*MB",
      r"≤\s*(?P<v2>[\d.]+)\s*MB\s+للمُثبِّت")),
    ("langpack",   "language pack",      "150", "KB", r"≤\s*(?P<v>[\d.]+)\s*KB\s+نص"),
    ("amsi",       "AMSI added latency", "15",  "ms", r"≤\s*(?P<v>[\d.]+)\s*ms\s+لكل\s+سكربت"),
    ("langswitch","language switch",     "100", "ms",
     r"تبديل\s+اللغة[^\n]{0,30}?<\s*(?P<v>[\d.]+)\s*ms"),
]

# ---------------------------------------------------------------------------
# 4. retired by the 2026-11-20 session. A hit here is a decision not applied.
#    (D-id, human label, regex)
# ---------------------------------------------------------------------------
RETIRED = [
    ("D64", "installer hard cap 40MB",      r"≤\s*40\s*MB|40MB\s*دائمًا|سقف داخلي قاسٍ"),
    ("D61", "database budget 120MB",        r"(?:≤\s*120\s*MB|120MB\s*مضغوط)"),
    ("D52", "CPU idle stated as 0%",        r"CPU[^\n]{0,25}?(?:0%|=\s*0\b)|(?:^|\|)\s*0%\s*\|"),
    ("D92", "estimate in weeks",            r"\d+\s*أسبوع"),
    ("D71", "service described as idle",    r"الخدمة\s+خاملة|الخدمة\s+«?خاملة»?"),
    ("D82", "review batches",               r"دفعات\s+المراجعة"),
    ("D85", "cookie/trace of location",     r"وموقعه|GeoIP\s*=\s*نعم"),
    ("D96", "emoji in the docs",            None),  # handled by check_emoji
]

# Emoji and pictographs. Arrows and Arabic-Indic digits are legitimate
# typography in RTL prose, so U+2190-U+21FF is deliberately absent.
EMOJI_RANGES = [
    (0x1F300, 0x1FAFF), (0x1F000, 0x1F2FF), (0x2600, 0x27BF),
    (0x2B00, 0x2BFF), (0xFE0F, 0xFE0F), (0x1F1E6, 0x1F1FF),
]

ANCHOR_DEF = re.compile(r'<a id="([^"]+)"></a>|<a id=\'([^\']+)\'></a>|id="([a-zA-Z][\w-]*)"')
ANCHOR_REF = re.compile(r"\]\(#([\w-]+)\)")
DECISION_DEF = re.compile(r"^\|\s*\*{0,2}(D\d+)\*{0,2}\s*\|", re.M)
DECISION_REF = re.compile(r"\bD(\d{1,3})\b")


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def check_anchor(text: str, errors: list[str]) -> None:
    defined = set()
    for m in ANCHOR_DEF.finditer(text):
        defined.add(next(g for g in m.groups() if g))
    for ref in sorted(set(ANCHOR_REF.findall(text))):
        if ref not in defined:
            errors.append(f"anchor: #{ref} has no target")


def check_decisions(prompt: str, register: str, errors: list[str]) -> None:
    defined = set(DECISION_DEF.findall(register))
    if not defined:
        errors.append("decisions: register defines no D## ids")
        return
    cited = {f"D{n}" for n in DECISION_REF.findall(prompt)}
    for d in sorted(cited - defined, key=lambda s: int(s[1:])):
        errors.append(f"decisions: {d} cited in the prompt but absent from decisions.md")
    nums = sorted(int(d[1:]) for d in defined)
    if nums[0] != 1:
        errors.append("decisions: register does not start at D1")
    for a, b in zip(nums, nums[1:]):
        if b != a + 1:
            errors.append(f"decisions: numbering gap between D{a} and D{b}")


def flatten(line: str) -> str:
    """Drop markdown emphasis so **هدف** matches like هدف."""
    line = re.sub(r"[*_`]{1,3}", "", line)
    return re.sub(r"\s+", " ", line)


def check_canon(errors: list[str]) -> None:
    """Every canonical budget appears at least once, always with one value."""
    for key, label, canonical, unit, pattern in CANON:
        patterns = pattern if isinstance(pattern, tuple) else (pattern,)
        rxs = [re.compile(p) for p in patterns]
        found: dict[str, list[str]] = {}
        for path in (PROMPT, DECISIONS):
            for no, line in enumerate(read(path).splitlines(), 1):
                flat = flatten(line)
                for rx in rxs:
                    for m in rx.finditer(flat):
                        value = next(g for g in m.groups() if g)
                        found.setdefault(value, []).append(f"{path.name}:{no}")
        if not found:
            errors.append(f"canon[{key}]: {label} ({canonical}{unit}) not stated anywhere")
            continue
        for value, locs in found.items():
            if value != canonical:
                errors.append(
                    f"canon[{key}]: {label} says {value}{unit}, "
                    f"canonical is {canonical}{unit} ({locs[0]})"
                )


# Words that mark a line as *describing* a retirement rather than using the value.
# Matched after stripping Arabic diacritics, so "تُحذف" and "حُذف" are one word.
DIACRITICS = re.compile(r"[\u064B-\u0652\u0670]")
RETIREMENT_WORDS = re.compile(r"تحذف|يحذف|حذف|ملغى|الغى|لا يبقى|ممنوع|مرجع تاريخي|retired")


def check_retired(errors: list[str]) -> None:
    for did, label, pattern in RETIRED:
        if pattern is None:
            continue
        rx = re.compile(pattern)
        for path in (PROMPT, DECISIONS):
            for no, line in enumerate(read(path).splitlines(), 1):
                if rx.search(line) and not RETIREMENT_WORDS.search(DIACRITICS.sub("", line)):
                    errors.append(f"retired[{did}]: {label} reappears at {path.name}:{no}")


def check_emoji(prompt: str, register: str, errors: list[str]) -> None:
    for name, text in (("prompt", prompt), ("decisions", register)):
        for no, line in enumerate(text.splitlines(), 1):
            for ch in line:
                cp = ord(ch)
                if any(lo <= cp <= hi for lo, hi in EMOJI_RANGES):
                    errors.append(
                        f"emoji[{name}]:{no}: U+{cp:04X} {unicodedata.name(ch, '?')}"
                    )
                    break


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--quiet", action="store_true")
    args = ap.parse_args()

    if not PROMPT.exists() or not DECISIONS.exists():
        print("doc-lint: missing docs/odina-agent-prompt-v19.md or docs/decisions.md")
        return 2

    prompt, register = read(PROMPT), read(DECISIONS)
    errors: list[str] = []
    check_anchor(prompt, errors)
    check_decisions(prompt, register, errors)
    check_canon(errors)
    check_retired(errors)
    check_emoji(prompt, register, errors)

    if errors:
        print(f"doc-lint: {len(errors)} violation(s)")
        for e in errors[:40]:
            print("  " + e)
        return 1

    if not args.quiet:
        cited = len({f"D{n}" for n in DECISION_REF.findall(prompt)})
        print(
            f"doc-lint: ok — anchors resolve · {cited} decisions cited, numbering contiguous · "
            f"{len(CANON)} budgets single-valued · {len([r for r in RETIRED if r[2]])} retired items absent"
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())
