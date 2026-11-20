#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
"""Generate the translation editor from the live locale files and the screen list.

Why generated rather than written by hand: the editor must never drift from
`locales/*/odina.ftl`. A hand-written editor holds a second copy of the strings,
and a second copy is a copy that will be edited and not saved back. Here the
strings are embedded from the files themselves at build time, and the export
writes the same Fluent syntax the linter already checks.

What the editor does that matters for a reviewer:

  * shows the Arabic and English side by side for every key, in screen groups,
    so a translator does not read a flat list of 156 identifiers;
  * keeps the variables of a string visible and blocks an export that drops one
    — `$until` lost in a translation is a runtime crash, not a typo;
  * refuses to save a file with an unbalanced brace or a duplicate key, which
    are the two defects `i18n-lint` would otherwise catch an hour later;
  * exports exactly `locales/<lang>/odina.ftl`, same header comment, same key
    order, so `git diff` shows only the strings a human actually changed.

Usage:
    python3 tools/gen-translation-editor.py            # write docs/design/translate.html
    python3 tools/gen-translation-editor.py --check    # fail if it is stale
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LOCALES = ROOT / "locales"
OUT = ROOT / "docs" / "design" / "translate.html"
REFERENCE = "ar"
LAYOUT_BASE = "en"

KEY_LINE = re.compile(r"^([A-Za-z0-9_.-]+)\s*=\s*(.*)$")
# Section headers look like `## ── هوية التطبيق ──────`; the label is what
# sits between the dashes, and the trailing dashes must not be swallowed.
SECTION_LINE = re.compile(r"^##\s*(?:──\s*)?(.+?)\s*(?:─{2,})?\s*$")
VAR = re.compile(r"\{\s*\$([A-Za-z0-9_]+)\s*\}")

# Screens, in the order the product presents them. A key is assigned to the
# first prefix that matches; anything unmatched lands in "other" and the build
# fails, which is how a new key gets a home instead of a corner.
SCREENS: list[tuple[str, str, tuple[str, ...]]] = [
    ("main", "الرئيسية", ("state.", "app.", "wall.", "action.", "scan.", "home.",
                "shield.")),
    ("alerts", "التنبيهات", ("alert.", "alerts.", "badge.", "button.", "path.", "word.",
                             "error.")),
    ("remediation", "المعالجة", ("recovery.",)),
    ("quarantine", "صندوق العزل", ("quarantine.", "q.")),
    ("usb", "USB", ("usb.",)),
    ("pause", "الإيقاف", ("pause.", "disable.", "lvl.", "lvl1.", "lvl2.", "tip.lvl1",
                          "tip.lvl2", "tip.no_permanent")),
    ("exception", "الاستثناءات", ("exception.",)),
    ("proactive", "الحماية الاستباقية", ("proactive.", "home.")),
    ("trust", "أسس الثقة", ("trust.", "notes.", "sec.", "visual.")),
    ("update", "التحديث", ("update.",)),
    ("signing", "سياسة التوقيع", ("signing.",)),
    ("browser", "قائمة المتصفح", ("browser.",)),
    ("map", "خريطة البرنامج", ("map.", "nav.", "back", "menu.")),
    ("help", "الشرح عند الطلب", ("help.", "tips.", "tip.")),
    ("numbers", "الأرقام", ("num.",)),
    ("settings", "الإعدادات", ("settings.", "privacy.", "lang.")),
]


def parse_ftl(path: Path) -> tuple[list[tuple[str, str]], dict[str, str]]:
    """Return (ordered key/value pairs, key -> section header as written)."""
    pairs: list[tuple[str, str]] = []
    sections: dict[str, str] = {}
    current = ""
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.rstrip()
        # Section headers start with `##` too, so they must be read before the
        # blanket comment skip — otherwise every key lands in "no section".
        header = SECTION_LINE.match(line)
        if header:
            current = header.group(1).strip()
            continue
        if line.startswith("#"):
            continue
        m = KEY_LINE.match(line)
        if m:
            key, value = m.group(1), m.group(2)
            pairs.append((key, value))
            sections[key] = current
    return pairs, sections


def screen_of(key: str) -> str:
    for name, _label, prefixes in SCREENS:
        for prefix in prefixes:
            if key == prefix or key.startswith(prefix):
                return name
    return "other"


def collect() -> dict:
    languages: dict[str, list[tuple[str, str]]] = {}
    section_of: dict[str, str] = {}
    for directory in sorted(p for p in LOCALES.iterdir() if p.is_dir()):
        path = directory / "odina.ftl"
        if not path.exists():
            continue
        pairs, sections = parse_ftl(path)
        languages[directory.name] = pairs
        if directory.name == REFERENCE:
            section_of = sections

    if REFERENCE not in languages:
        raise SystemExit(f"gen-translation-editor: {REFERENCE} locale is missing")

    order = [key for key, _ in languages[REFERENCE]]
    # Keys present in another language but missing from the reference are
    # reported by i18n-lint; the editor shows them so the fix happens here.
    extra = sorted({key for pairs in languages.values() for key, _ in pairs} - set(order))

    keys = []
    for key in order + extra:
        keys.append({
            "key": key,
            "screen": screen_of(key),
            "section": section_of.get(key, ""),
            "vars": sorted(set(VAR.findall(dict(languages[REFERENCE]).get(key, "")))),
        })

    return {
        "languages": sorted(languages),
        "reference": REFERENCE,
        "layout_base": LAYOUT_BASE,
        "screens": [{"id": name, "label": label} for name, label, _ in SCREENS]
        + [{"id": "other", "label": "غير مصنّف"}],
        "keys": keys,
        "strings": {lang: dict(pairs) for lang, pairs in languages.items()},
    }


def build(data: dict) -> str:
    payload = json.dumps(data, ensure_ascii=False, separators=(",", ":"))
    template = (ROOT / "tools" / "translate-template.html").read_text(encoding="utf-8")
    marker = "/*__DATA__*/null"
    if marker not in template:
        raise SystemExit("gen-translation-editor: template marker missing")
    html = template.replace(marker, payload, 1)
    # The template is an MIT developer tool; the page it emits lives inside the
    # product, so the emitted page carries the product licence, not the tool's.
    return html.replace(
        "<!-- SPDX-License-Identifier: MIT -->\n"
        "<!-- Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS) -->",
        "<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->\n"
        "<!-- Copyright (c) 2026 bifa90 — Odina -->",
        1,
    )


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true",
                    help="fail if the generated editor differs from the sources")
    args = ap.parse_args()

    data = collect()
    html = build(data)
    OUT.parent.mkdir(parents=True, exist_ok=True)

    unassigned = sorted(k["key"] for k in data["keys"] if k["screen"] == "other")
    if unassigned:
        print("gen-translation-editor: keys with no screen:", ", ".join(unassigned[:8]),
              file=sys.stderr)
        return 1

    if args.check:
        current = OUT.read_text(encoding="utf-8") if OUT.exists() else ""
        if current != html:
            print("gen-translation-editor: translate.html is stale — regenerate it",
                  file=sys.stderr)
            return 1
        print(f"translate editor: current · {len(data['keys'])} keys · "
              f"{len(data['languages'])} languages")
        return 0

    OUT.write_text(html, encoding="utf-8")
    print(f"wrote {OUT.relative_to(ROOT)} · {len(data['keys'])} keys · "
          f"{len(data['languages'])} languages")
    return 0


if __name__ == "__main__":
    sys.exit(main())
