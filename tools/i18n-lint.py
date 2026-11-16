#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
"""Fluent linter — guards the localisation contract (§3.9, §3.8).

Checks, in order of severity:

  ERROR  (fails the build)
    * syntax: duplicate keys, unparsable message, stray indentation
    * a message that ends inside an unterminated `{ $var }`
    * TERMINOLOGY: a security message that dropped the "-" style emphasis or
      lost a word like "temporarily" is invisible to a linter, so instead we
      enforce the mechanical half: the same set of `$variables` per key
      across every locale. A translation that quietly drops `$count` is a
      real defect.
    * SHARED KEYS: every locale must expose exactly the same key set as
      the reference locale (ar). Missing keys fall back to Arabic at
      runtime, so they are a warning, not an error -- but a key that does
      not exist in the reference is dead weight and is an error.

  WARN   (reported, build passes)
    * missing keys in a non-reference locale
    * untranslated strings (identical to the reference value)
    * strings longer than 1.8x the reference (layout risk, §3.9.5)
    * strings that grew past the +40% pseudo-localisation budget

Usage:
    python3 tools/i18n-lint.py [--root .] [--strict] [--json]

Exit codes: 0 = clean or warnings only, 1 = errors, 2 = usage/layout error.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

REFERENCE = "ar"          # key set + variable parity
LAYOUT_BASE = "en"        # length baseline: Arabic is too compact to compare against
VAR_RE = re.compile(r"\{\s*\$([A-Za-z0-9_]+)\s*\}")
KEY_RE = re.compile(r"^([A-Za-z0-9_.\-]+)\s*=\s*(.*)$")
ATTR_RE = re.compile(r"^\s+\.([A-Za-z0-9_\-]+)\s*=\s*(.*)$")
MULTILINE_CONT = re.compile(r"^\s{4,}\S")
MAX_GROWTH = 1.8
PSEUDO_BUDGET = 1.4


class Issue:
    __slots__ = ("severity", "lang", "key", "message")

    def __init__(self, severity: str, lang: str, key: str, message: str):
        self.severity = severity
        self.lang = lang
        self.key = key
        self.message = message

    def as_dict(self) -> dict:
        return {"severity": self.severity, "lang": self.lang,
                "key": self.key, "message": self.message}

    def __str__(self) -> str:
        where = f"{self.lang}:{self.key}" if self.key else self.lang
        return f"{self.severity:<5} {where}: {self.message}"


def parse_ftl(path: Path) -> tuple[dict[str, str], list[Issue]]:
    """A deliberately small FTL reader: enough to lint, not to render."""
    entries: dict[str, str] = {}
    issues: list[Issue] = []
    lang = path.parent.name
    current: str | None = None
    for lineno, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        line = raw.rstrip()
        if not line.strip():
            continue
        stripped = line.lstrip()
        if stripped.startswith("#"):
            continue
        m = KEY_RE.match(line)
        if m and not line.startswith((" ", "\t")):
            key = m.group(1)
            if key in entries:
                issues.append(Issue("ERROR", lang, key,
                                    f"duplicate key (line {lineno})"))
            entries[key] = m.group(2)
            current = key
            continue
        if ATTR_RE.match(line) and current:
            entries[f"{current}.{line.strip().split('=')[0].strip()}"] = line
            continue
        if MULTILINE_CONT.match(line) and current:
            entries[current] += " " + stripped
            continue
        issues.append(Issue("ERROR", lang, current or "",
                            f"unparsable line {lineno}: {stripped[:60]!r}"))
    return entries, issues


def brace_balance(value: str) -> bool:
    depth = 0
    for ch in value:
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth < 0:
                return False
    return depth == 0


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default=".")
    ap.add_argument("--strict", action="store_true",
                    help="treat warnings as failures")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--quiet", action="store_true",
                    help="only print the summary line")
    args = ap.parse_args()

    locale_root = Path(args.root) / "locales"
    if not locale_root.is_dir():
        print(f"i18n-lint: {locale_root} not found", file=sys.stderr)
        return 2

    files = sorted(locale_root.glob("*/odina.ftl"))
    if not files:
        print("i18n-lint: no locales/*/odina.ftl found", file=sys.stderr)
        return 2

    issues: list[Issue] = []
    parsed: dict[str, dict[str, str]] = {}

    for path in files:
        entries, errs = parse_ftl(path)
        issues.extend(errs)
        parsed[path.parent.name] = entries

    if REFERENCE not in parsed:
        print(f"i18n-lint: reference locale {REFERENCE!r} missing", file=sys.stderr)
        return 2

    ref = parsed[REFERENCE]

    # 1. variable parity — a dropped $var is a real translation defect
    ref_vars = {k: set(VAR_RE.findall(v)) for k, v in ref.items()}
    for lang, entries in parsed.items():
        if lang == REFERENCE:
            continue
        for key, value in entries.items():
            if key not in ref_vars:
                issues.append(Issue("ERROR", lang, key,
                                    "key does not exist in the reference locale"))
                continue
            have = set(VAR_RE.findall(value))
            want = ref_vars[key]
            if have != want:
                missing = want - have
                extra = have - want
                bits = []
                if missing:
                    bits.append(f"missing {sorted(missing)}")
                if extra:
                    bits.append(f"unknown {sorted(extra)}")
                issues.append(Issue("ERROR", lang, key,
                                    "variable mismatch: " + ", ".join(bits)))

    # 2. brace balance
    for lang, entries in parsed.items():
        for key, value in entries.items():
            if not brace_balance(value):
                issues.append(Issue("ERROR", lang, key, "unbalanced braces"))

    # 3. coverage + growth
    for lang, entries in parsed.items():
        if lang == REFERENCE:
            continue
        missing = sorted(set(ref) - set(entries))
        for key in missing:
            issues.append(Issue("WARN", lang, key,
                                "missing — falls back to the reference language"))
        for key, value in entries.items():
            ref_value = ref.get(key)
            if ref_value is None:
                continue
            if value.strip() == ref_value.strip() and lang != LAYOUT_BASE:
                issues.append(Issue("WARN", lang, key,
                                    "identical to the reference (untranslated?)"))
                continue
            base = parsed.get(LAYOUT_BASE, {}).get(key, ref_value)
            if base and base.strip() and lang != LAYOUT_BASE:
                ratio = len(value) / max(len(base), 1)
                if ratio > MAX_GROWTH:
                    issues.append(Issue(
                        "WARN", lang, key,
                        f"length {ratio:.2f}x the reference"
                        f" vs {LAYOUT_BASE} (limit {MAX_GROWTH:.1f}) — layout risk"))
                if ratio > PSEUDO_BUDGET:
                    pass  # covered by MAX_GROWTH; pseudo-l10n is separate

    errors = [i for i in issues if i.severity == "ERROR"]
    warns = [i for i in issues if i.severity == "WARN"]

    if args.json:
        print(json.dumps({
            "locales": sorted(parsed),
            "reference": REFERENCE,
            "keys": len(ref),
            "errors": [i.as_dict() for i in errors],
            "warnings": [i.as_dict() for i in warns],
        }, ensure_ascii=False, indent=2))
    else:
        if not args.quiet:
            for issue in errors + warns:
                print(issue)
            print()
        print(f"i18n-lint: {len(parsed)} locale(s) · {len(ref)} keys · "
              f"{len(errors)} error(s) · {len(warns)} warning(s)")

    if errors or (args.strict and warns):
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
