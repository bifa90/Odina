#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
"""A cheap structural check for Rust source, for machines without rustc.

This is not a parser and does not pretend to be one. It catches the class of
mistake that costs a CI cycle when no local toolchain exists: unbalanced
delimiters, an unclosed string, a stray brace inside a doc comment.

The real check is `cargo build`; this only decides whether pushing is worth a
run. Braces inside `//`, `///`, `//!` comments and inside string literals are
ignored — which is where every false positive of a naive counter comes from,
including the one in this repository's own docs.
"""

from __future__ import annotations

import sys
from pathlib import Path

PAIRS = {")": "(", "]": "[", "}": "{"}


def strip_noise(line: str, in_block_comment: bool) -> tuple[str, bool]:
    """Remove comments and literal contents from one line."""
    out = []
    i = 0
    while i < len(line):
        if in_block_comment:
            end = line.find("*/", i)
            if end == -1:
                return "".join(out), True
            i = end + 2
            in_block_comment = False
            continue
        two = line[i : i + 2]
        if two == "//":
            break
        if two == "/*":
            in_block_comment = True
            i += 2
            continue
        ch = line[i]
        # A lifetime (`'a`, `'static`) is not a character literal, and a char
        # literal may hold a brace (`'{'`). Both are common enough that a
        # scanner which confuses them reports noise on correct code.
        if ch == "'":
            if i + 1 < len(line) and line[i + 1] == "\\":
                # escaped char literal: '\'', '\\', '\n'
                end = line.find("'", i + 2)
                i = len(line) if end == -1 else end + 1
                out.append(" ")
                continue
            if i + 2 < len(line) and line[i + 2] == "'":
                i += 3
                out.append(" ")
                continue
            # lifetime: consume the identifier
            j = i + 1
            while j < len(line) and (line[j].isalnum() or line[j] == "_"):
                j += 1
            i = j
            continue
        # Raw strings: r"C:\\x", r#"a "quoted" b"#, r##"..."##. The hashes are
        # the point — without them a path constant containing a quote ends the
        # string early and every later brace is miscounted.
        if ch == "r" and i + 1 < len(line) and line[i + 1] in '#"':
            hashes = 0
            j = i + 1
            while j < len(line) and line[j] == "#":
                hashes += 1
                j += 1
            if j < len(line) and line[j] == '"':
                closing = '"' + "#" * hashes
                end = line.find(closing, j + 1)
                i = len(line) if end == -1 else end + len(closing)
                out.append(" ")
                continue
        if ch == '"':
            quote = ch
            i += 1
            while i < len(line):
                if line[i] == "\\":
                    i += 2
                    continue
                if line[i] == quote:
                    break
                i += 1
            if i >= len(line):
                out.append(" ")
                break
            i += 1
            out.append(" ")
            continue
        out.append(ch)
        i += 1
    return "".join(out), in_block_comment


def check(path: Path) -> list[str]:
    problems: list[str] = []
    stack: list[tuple[str, int]] = []
    in_block = False
    for number, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        code, in_block = strip_noise(raw, in_block)
        for ch in code:
            if ch in "([{":
                stack.append((ch, number))
            elif ch in PAIRS:
                if not stack:
                    problems.append(f"{path}:{number}: stray {ch!r}")
                elif stack[-1][0] != PAIRS[ch]:
                    opened, at = stack[-1]
                    problems.append(f"{path}:{number}: {ch!r} closes {opened!r} opened at {at}")
                    stack.pop()
                else:
                    stack.pop()
    if in_block:
        problems.append(f"{path}: unterminated block comment")
    for opened, at in stack:
        problems.append(f"{path}:{at}: {opened!r} never closed")
    return problems


def main() -> int:
    roots = sys.argv[1:] or ["crates"]
    files = sorted(p for r in roots for p in Path(r).rglob("*.rs"))
    if not files:
        print("no .rs files found", file=sys.stderr)
        return 2
    problems = [p for f in files for p in check(f)]
    for p in problems[:20]:
        print(p, file=sys.stderr)
    if problems:
        print(f"{len(problems)} problem(s) in {len(files)} file(s)", file=sys.stderr)
        return 1
    print(f"{len(files)} rust file(s) balanced")
    return 0


if __name__ == "__main__":
    sys.exit(main())
