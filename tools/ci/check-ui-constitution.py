#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
"""UI constitution gate — design-system-v1.md §6أ, enforced.

Five rules the owner asked for, each one a failure the moment it is broken.
Written as a gate rather than as guidance because three of the five were
already written as guidance in v1 and were being broken in the preview.

  1. Short text: every on-screen hint is at most 90 characters, and every
     `data-tipkey` exists in the glossary. Explanation lives in a tooltip or
     in a search, never in a paragraph on the screen.
  2. One decision: at most one filled (primary) button per card, and at most
     two visible buttons. The rest belong inside a menu.
  3. Every screen is reachable: every screen in the map appears in the
     preview, and the map is drawn on the page rather than described in a doc.
  4. Icons explain themselves: every icon used is declared in the sprite, and
     every icon-only control has an accessible name.
  5. Search engines follow the language: zh-Hans maps to Baidu, ja to Yahoo
     Japan, ru to Yandex, and the rest to Google — never one engine for all.

Also checks that the generated translation editor is current, because an
editor that drifted from the locale files is how a reviewer's work gets lost.

Usage: python3 tools/ci/check-ui-constitution.py [--root .] [--quiet]
Exit:  0 clean · 1 violation · 2 the preview is missing
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

HINT_LIMIT = 90

# From docs/spec/glossary-v1.md §4. One engine per language, by rule not by
# accident: Google is not reachable in China, Yahoo Japan is the incumbent in
# Japan, Yandex reads Russian better than anyone.
SEARCH_ENGINE = {
    "zh-Hans": "baidu",
    "ja": "yahoo",
    "ru": "yandex",
}
DEFAULT_ENGINE = "google"

SCREEN_IDS = [
    "home", "quarantine", "proactive", "alerts", "settings",
    "language", "privacy", "exception",
]


def strip_comments(html: str) -> str:
    return re.sub(r"<!--.*?-->", "", html, flags=re.S)


def check_hints(html: str) -> list[str]:
    problems = []
    # class="hint" blocks are the one place a paragraph is allowed, so their
    # length is the measure of "the screen explains itself too much".
    for match in re.finditer(r'<(p|div)[^>]*class="[^"]*\bhint\b[^"]*"[^>]*>(.*?)</\1>',
                             html, re.S):
        body = re.sub(r"<[^>]+>", "", match.group(2))
        body = re.sub(r"\s+", " ", body).strip()
        if len(body) > HINT_LIMIT:
            problems.append(
                f"hint is {len(body)} characters (limit {HINT_LIMIT}): {body[:60]}…")
    return problems


def check_tips(html: str, root: Path) -> list[str]:
    problems = []
    glossary = (root / "docs" / "spec" / "glossary-v1.md").read_text(encoding="utf-8")
    declared = set(re.findall(r"`(tip\.[A-Za-z0-9_]+)`", glossary))
    # data-why carries the on-demand explanation, and it is a term like any
    # other: a long text nobody declared is a text nobody reviewed.
    used = set(re.findall(r'data-tipkey="([^"]+)"', html))
    used |= set(re.findall(r'data-why="([^"]+)"', html))
    used |= set(re.findall(r'data-list-tip="(tip\.[^"]+)"', html))
    for key in sorted(used - declared):
        problems.append(f"tooltip {key} is used but not in glossary-v1.md")
    return problems


def check_primary_buttons(html: str) -> list[str]:
    problems = []
    for card in re.findall(r'<div class="(?:alert|exccard|row)[^"]*"[^>]*>(.*?)</div>\s*</div>',
                           html, re.S):
        primaries = len(re.findall(r'class="[^"]*\bprimary\b', card))
        # A field utility (copy the path, reveal a value) is not a decision:
        # it is marked data-utility and never counts against the two-button rule.
        visible = len(re.findall(r"<button(?![^>]*data-utility)", card))
        if primaries > 1:
            problems.append(f"{primaries} filled buttons in one card — one decision only")
        if visible > 2 and "advanced" not in card:
            problems.append(f"{visible} visible buttons in one card — move the rest into a menu")
    return problems


def check_map(html: str, root: Path) -> list[str]:
    problems = []
    # The map must be drawn on the page, not only described in the spec.
    if 'class="appmap"' not in html:
        problems.append("the app map is not drawn in the preview (missing .appmap)")

    for screen in SCREEN_IDS:
        if f'data-screen="{screen}"' not in html:
            problems.append(f"screen {screen!r} is not in the map")
        if f'id="screen-{screen}"' not in html:
            problems.append(f"screen {screen!r} is named in the map but has no section")

    # Every way into a screen must exist as a link or button in the page.
    for target in re.findall(r'data-goto="([^"]+)"', html):
        if f'id="screen-{target}"' not in html:
            problems.append(f"data-goto={target!r} points at a screen that does not exist")
    return problems


def check_icons(html: str) -> list[str]:
    problems = []
    symbols = set(re.findall(r'<symbol id="([^"]+)"', html))
    # Only markup counts: a <use> assembled at run time (ICON(id)) cannot be
    # resolved statically, and pretending it can produced a false alarm.
    markup = re.sub(r"<script.*?</script>", "", html, flags=re.S)
    for name in set(re.findall(r'<use href="#([^"]+)"', markup)):
        if name not in symbols:
            problems.append(f"icon {name!r} is not declared in the sprite")
    # Icon-only controls need a name a screen reader can say.
    for button in re.findall(r"<button[^>]*>\s*<svg[^>]*>.*?</svg>\s*</button>", html, re.S):
        if "aria-label" not in button and "data-k" not in button and "title=" not in button:
            problems.append("an icon-only button has no accessible name")
    return problems


def check_visual_first(html: str) -> list[str]:
    """Rule four: the picture before the sentence.

    A screen that only works when its text is read fails the user who reads
    slowly, or does not read the language yet. So: every navigable target on
    the home screen carries an icon, and the app map is a diagram, not a list.
    """
    problems = []

    home = re.search(r'<section[^>]*id="screen-home"[^>]*>(.*?)</section>', html, re.S)
    if home:
        body = home.group(1)
        for m in re.finditer(r'<button([^>]*)>(.*?)</button>', body, re.S):
            attrs, inner = m.group(1), m.group(2)
            if "data-goto" not in attrs:
                continue
            if "<svg" not in inner:
                label = re.search(r'data-k="([^"]+)"', attrs)
                name = label.group(1) if label else inner.strip()[:30]
                problems.append(
                    f"home action {name!r} has no icon — it relies on the word alone"
                )
        # The drawer of destinations must be present on the screen itself,
        # not only inside the diagram: a map you must open is not a map.
        if 'class="appbar"' not in body:
            problems.append("home has no visible navigation bar (.appbar)")

    # Every counter on the screen must obey the user's numeral choice (§6أ.6).
    for badge in re.findall(r'<span class="nb[^"]*"[^>]*>([^<]*)</span>', html):
        if any(ch.isdigit() for ch in badge) and 'data-numeral' not in badge:
            pass  # attribute checked below
    for m in re.finditer(r'<span class="nb[^"]*"([^>]*)>', html):
        if 'data-numeral' not in m.group(1):
            problems.append("a counter shows a number that ignores the numeral setting")

    # The numeral picker must exist and offer both systems.
    if 'data-num="arabic"' not in html or 'data-num="hindi"' not in html:
        problems.append("the numeral choice is missing (arabic + hindi)")

    # Rule four must be stated in the constitution section of the page.
    if 'data-k="visual.rule"' not in html:
        problems.append("the fourth rule (visual first) is not on the page")
    return problems


def check_reachability(html: str) -> list[str]:
    """A screen nobody can open is not a screen (§6أ.3).

    Two failures this catches, both of which shipped once:
      · a data-goto pointing at an id that does not exist — a dead link
      · a screen with an id and no entry — a page behind a wall
    """
    problems = []
    screens = set(re.findall(r'data-screen="([^"]+)"', html))
    ids = set(re.findall(r'id="screen-([^"]+)"', html))
    gotos = set(re.findall(r'data-goto="([^"]+)"', html))

    for target in sorted(gotos - ids):
        problems.append(f"data-goto={target!r} points at no screen — a dead link")
    for screen in sorted(screens - gotos - {"home"}):
        problems.append(f"screen {screen!r} has no entry — nobody can open it")
    for screen in sorted(ids - screens):
        problems.append(f"id=screen-{screen!r} has no data-screen — half a screen")
    return problems


def check_templates(html: str) -> list[str]:
    """A placeholder that no code fills is printed to the user verbatim.

    data-tmpl + data-k-args only work because renderString() exists; if that
    function is gone, every { $n } on the page becomes visible text.
    """
    problems = []
    # renaming the definition leaves the call behind: a call with no body is
    # how the { $n } bug actually looks.
    if 'data-tmpl=' in html:
        # The definition is the only occurrence followed by a parameter list.
        defined = re.search(r'function\s+renderString\s*\(', html) is not None
        if not defined:
            problems.append("data-tmpl is used but renderString() is missing — { $var } would show")
    if 'data-tmpl=' in html and 'formatNumber' not in html:
        problems.append("templated numbers bypass the numeral choice")
    # every templated element must carry its arguments
    for m in re.finditer(r'<[^>]*data-tmpl="([^"]+)"[^>]*>', html):
        if 'data-k-args' not in m.group(0):
            problems.append(f"templated {m.group(1)!r} has no data-k-args — it would render empty")
    return problems


def check_markup(html: str, page: str) -> list[str]:
    """Balanced tags — because a nested <section> renders as chaos, not an error.

    This shipped once: a second <section> opened inside the alerts screen,
    every gate passed, and the page was structurally wrong. A browser hides
    the damage; a parser does not.
    """
    from html.parser import HTMLParser

    void = {"br", "img", "meta", "link", "input", "hr", "use", "path", "circle",
            "rect", "source", "area", "base", "col", "embed", "param", "track", "wbr"}

    class Checker(HTMLParser):
        def __init__(self) -> None:
            super().__init__()
            self.stack: list[tuple[str, int]] = []
            self.errors: list[str] = []

        def handle_starttag(self, tag, attrs):
            if tag not in void:
                self.stack.append((tag, self.getpos()[0]))

        def handle_endtag(self, tag):
            if tag in void:
                return
            if not self.stack:
                self.errors.append(f"stray </{tag}> on line {self.getpos()[0]}")
                return
            if self.stack[-1][0] != tag:
                self.errors.append(
                    f"line {self.getpos()[0]}: </{tag}> closes "
                    f"<{self.stack[-1][0]}> opened on line {self.stack[-1][1]}"
                )
                for i in range(len(self.stack) - 1, -1, -1):
                    if self.stack[i][0] == tag:
                        del self.stack[i:]
                        break
            else:
                self.stack.pop()

    checker = Checker()
    checker.feed(html)
    problems = [f"{page}: {e}" for e in checker.errors[:3]]
    for tag, line in checker.stack[:3]:
        problems.append(f"{page}: <{tag}> opened on line {line} is never closed")
    return problems


def check_editable(html: str, root: Path) -> list[str]:
    """Every string on the page must be editable per language.

    A key that lives only in the page cannot be reached by a translator, and
    the editor would export a file that silently drops it. So the page and the
    locale files must agree in both directions.
    """
    problems = []
    locales = sorted((root / "locales").glob("*/odina.ftl"))
    if not locales:
        return ["no locale files found"]

    # Navigation descriptions are bound to [data-k] at boot. Rule/setting
    # descriptions already live in their disclosure in the HTML.
    list_tips = set(re.findall(r'data-list-tip="([^"]+)"', html))
    for tag in re.findall(r'<[^>]*\brole="tooltip"[^>]*>', html):
        list_tips.update(re.findall(r'data-k="([^"]+)"', tag))
    used = set(re.findall(r'data-k="([^"]+)"', html)) | list_tips
    for path in locales:
        lang = path.parent.name
        values = dict(re.findall(r"^([A-Za-z0-9_.-]+) = (.*)$", path.read_text(encoding="utf-8"), re.M))
        missing = sorted(used - values.keys())
        if missing:
            problems.append(
                f"{lang}: {len(missing)} key(s) on the page are not in the locale "
                f"file, so they cannot be edited: {', '.join(missing[:4])}"
            )
        for key in sorted(list_tips & values.keys()):
            if not values[key].strip() or len(values[key]) > HINT_LIMIT:
                problems.append(f"{lang}: list tip {key} must be 1–{HINT_LIMIT} characters")

    # The signing page is the other editable surface; it must stay data-driven.
    signing = root / "signing.html"
    if signing.is_file():
        text = signing.read_text(encoding="utf-8")
        if 'data-k="signing.title"' not in text:
            problems.append("signing.html is not driven by signing.* keys")
        # A literal is only a problem when it has no key: text inside a
        # data-k element is the fallback a translator replaced.
        hard = re.findall(
            r"<(?:h1|h2|p|li)(?![^>]*data-k)[^>]*>\s*([^<\s][^<]{5,})\s*</(?:h1|h2|p|li)>",
            text,
        )
        if hard:
            problems.append(
                f"signing.html has {len(hard)} untranslated literal(s), e.g. {hard[0][:60]!r}"
            )
    return problems


def check_search_engine(html: str) -> list[str]:
    problems = []
    # The mapping must exist in the page as data, keyed by language.
    found = dict(re.findall(r'["\']([a-zA-Z-]+)["\']\s*:\s*["\'](google|baidu|yahoo|yandex)["\']',
                            html))
    if not found:
        return ["no search-engine table found in the preview"]
    for lang, engine in SEARCH_ENGINE.items():
        if found.get(lang) != engine:
            problems.append(
                f"{lang} must search {engine}, the page says {found.get(lang)!r}")
    for lang, engine in found.items():
        if lang in SEARCH_ENGINE:
            continue
        if engine != DEFAULT_ENGINE:
            problems.append(f"{lang} searches {engine} — only zh-Hans/ja/ru differ")
    return problems


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default=".")
    ap.add_argument("--quiet", action="store_true")
    args = ap.parse_args()
    root = Path(args.root)

    preview = root / "ui-preview.html"
    if not preview.exists():
        print("ui-constitution: ui-preview.html is missing", file=sys.stderr)
        return 2

    html = strip_comments(preview.read_text(encoding="utf-8"))
    problems = (
        check_hints(html)
        + check_tips(html, root)
        + check_primary_buttons(html)
        + check_map(html, root)
        + check_icons(html)
        + check_search_engine(html)
        + check_editable(html, root)
        + check_visual_first(html)
        + check_reachability(html)
        + check_templates(html)
        + check_markup(preview.read_text(encoding="utf-8"), "ui-preview.html")
    )

    # The editor must be regenerated whenever a locale changes, or a reviewer
    # edits stale strings and the work is thrown away at the merge.
    gen = root / "tools" / "gen-translation-editor.py"
    if gen.exists():
        result = subprocess.run([sys.executable, str(gen), "--check"],
                                capture_output=True, text=True, cwd=root)
        if result.returncode != 0:
            problems.append((result.stderr or result.stdout).strip().splitlines()[0])

    if problems:
        print(f"ui-constitution gate: {len(problems)} violation(s)", file=sys.stderr)
        for p in problems[:20]:
            print(f"  {p}", file=sys.stderr)
        return 1
    if not args.quiet:
        print("ui-constitution gate: short text · one decision · reachable · named icons · right engine")
    return 0


if __name__ == "__main__":
    sys.exit(main())
