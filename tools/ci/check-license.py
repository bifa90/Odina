#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
"""License gate — fails the build when a source file has no SPDX header.

Rule (see LICENSING.md §4):
  tools/**            -> MIT
  fonts/vendor/**     -> OFL (own LICENSE per pack, no per-file tag needed)
  assets/logo/**      -> brand rules, no tag needed
  docs/design/**      -> brand rules, no tag needed
  everything else     -> AGPL-3.0-or-later

Only files that can carry a comment are checked. Data, SVG art, fonts,
locales and lock files are skipped: a header inside them would either be
invalid or change the artifact's hash.

Usage:
    python3 tools/ci/check-license.py [--root .] [--fix] [--quiet]

Exit codes: 0 = clean, 1 = violations found, 2 = usage error.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

# Extensions that support comments and are hand-written source.
CHECKED = {
    ".py", ".rs", ".js", ".mjs", ".cjs", ".ts", ".tsx", ".jsx",
    ".c", ".h", ".cpp", ".hpp", ".cs", ".go", ".java", ".kt",
    ".sh", ".ps1", ".toml", ".yml", ".yaml",
}

# Directories never scanned.
SKIP_DIRS = {
    ".git", "node_modules", "target", "dist", "build", "out",
    "fonts", "assets", "locales", "__pycache__", ".venv", "venv",
}

SPDX_RE = re.compile(r"SPDX-License-Identifier:\s*([A-Za-z0-9.\-+]+)")

# Files that legitimately cannot carry a header (config, generated, workflow).
EXEMPT_NAMES = {
    ".gitignore", ".gitattributes", ".editorconfig", ".npmrc",
    "package-lock.json", "Cargo.lock", "tsconfig.json",
    "manifest.json", "index.json",
}


def expected_for(path: Path, root: Path) -> str:
    rel = path.relative_to(root)
    parts = rel.parts
    if parts and parts[0] == "tools":
        return "MIT"
    return "AGPL-3.0-or-later"


def header_lines(path: Path) -> list[str]:
    """First 12 non-empty lines — a header must live at the top."""
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return []
    out: list[str] = []
    for line in text.splitlines():
        if line.strip():
            out.append(line)
        if len(out) >= 12:
            break
    return out


def comment_prefix(path: Path) -> str:
    suffix = path.suffix.lower()
    if suffix in {".py", ".sh", ".yml", ".yaml", ".ps1"}:
        return "# "
    if suffix == ".toml":
        return "# "
    return "// "


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default=".")
    ap.add_argument("--fix", action="store_true", help="insert a missing header")
    ap.add_argument("--quiet", action="store_true")
    args = ap.parse_args()

    root = Path(args.root).resolve()
    violations: list[tuple[Path, str, str]] = []  # path, expected, reason
    checked = 0

    for path in sorted(root.rglob("*")):
        if not path.is_file():
            continue
        rel = path.relative_to(root)
        if any(part in SKIP_DIRS for part in rel.parts):
            continue
        if path.suffix.lower() not in CHECKED:
            continue
        if path.name in EXEMPT_NAMES:
            continue
        checked += 1

        lines = header_lines(path)
        hit = None
        for line in lines:
            m = SPDX_RE.search(line)
            if m:
                hit = m.group(1)
                break

        exp = expected_for(path, root)
        if hit is None:
            violations.append((rel, exp, "missing SPDX header"))
        elif hit != exp:
            violations.append((rel, exp, f"found {hit!r}, expected {exp!r}"))

    if args.fix:
        fixed = 0
        for rel, exp, reason in list(violations):
            if "missing" not in reason:
                continue
            p = root / rel
            text = p.read_text(encoding="utf-8")
            prefix = comment_prefix(p)
            marker = f"{prefix}SPDX-License-Identifier: {exp}\n"
            note = f"{prefix}Copyright (c) 2026 bifa90 — Odina\n"
            lines = text.splitlines(keepends=True)
            insert_at = 1 if lines and lines[0].startswith("#!") else 0
            lines.insert(insert_at, note)
            lines.insert(insert_at, marker)
            p.write_text("".join(lines), encoding="utf-8")
            fixed += 1
            violations = [v for v in violations if v[0] != rel]
        if fixed and not args.quiet:
            print(f"fixed {fixed} file(s)")

    if violations:
        print(f"license gate: {len(violations)} violation(s) in {checked} checked file(s)",
              file=sys.stderr)
        for rel, exp, reason in violations:
            print(f"  {rel}: {reason} (expected {exp})", file=sys.stderr)
        print("\nSee LICENSING.md §4 for the rule.", file=sys.stderr)
        return 1

    if not args.quiet:
        print(f"license gate: {checked} file(s) ok")
    return 0


if __name__ == "__main__":
    sys.exit(main())
