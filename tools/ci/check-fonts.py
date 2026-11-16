#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
"""Font-manifest gate — the CDN contract must stay provable.

fonts/manifests/index.json promises a sha256 for every file the user is
expected to upload to their own domain (docs/cdn-fonts.md). That promise
is worth nothing if the manifest drifts from the files on disk, so this
gate recomputes every hash and fails on any mismatch.

Also enforced:
  * every pack directory has its own LICENSE (OFL obligations)
  * every pack has at least one 400 and one 600 face (or 700 for PT Sans)
  * the totals in index.json match the sum of the files
  * a CDN pack does not sneak into the bundled set (size budget, §36.2)

Usage:
    python3 tools/ci/check-fonts.py [--root .] [--quiet] [--json]

Exit codes: 0 = ok, 1 = violation, 2 = manifest missing/unreadable.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

MANIFEST = Path("fonts/manifests/index.json")
VENDOR = Path("fonts/vendor")


def sha256_of(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default=".")
    ap.add_argument("--quiet", action="store_true")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    root = Path(args.root)
    manifest_path = root / MANIFEST
    if not manifest_path.exists():
        print(f"font gate: {MANIFEST} not found", file=sys.stderr)
        return 2

    data = json.loads(manifest_path.read_text(encoding="utf-8"))
    packs = data.get("packs", [])
    if not packs:
        print("font gate: manifest has no packs", file=sys.stderr)
        return 2

    problems: list[str] = []
    stats = {"files": 0, "bytes": 0, "bundled_bytes": 0, "cdn_bytes": 0}

    seen_files: set[str] = set()

    for pack in packs:
        name = pack.get("pack", "?")
        mode = pack.get("mode", "?")
        pack_dir = root / VENDOR / name
        if not pack_dir.is_dir():
            problems.append(f"{name}: directory missing ({pack_dir})")
            continue

        if not any(p.name.upper().startswith(("LICENSE", "OFL")) for p in pack_dir.iterdir()):
            problems.append(f"{name}: no LICENSE file in the pack")

        declared = pack.get("files", [])
        if not declared:
            problems.append(f"{name}: no files listed")

        weights: set[str] = set()
        on_disk = {p.name for p in pack_dir.glob("*.woff2")}
        declared_names = {f["file"] for f in declared}

        for entry in declared:
            fname = entry["file"]
            if fname in seen_files:
                problems.append(f"{name}: duplicate file across packs: {fname}")
            seen_files.add(fname)
            fpath = pack_dir / fname
            if not fpath.exists():
                problems.append(f"{name}/{fname}: listed but missing on disk")
                continue
            actual_bytes = fpath.stat().st_size
            if actual_bytes != entry.get("bytes"):
                problems.append(
                    f"{name}/{fname}: size {actual_bytes} != manifest {entry.get('bytes')}")
            actual_hash = sha256_of(fpath)
            if actual_hash != entry.get("sha256"):
                problems.append(f"{name}/{fname}: sha256 mismatch")
            # weight detection from the fontsource naming convention
            for w in ("400", "600", "700"):
                if f"-{w}-" in fname:
                    weights.add(w)
            stats["files"] += 1
            stats["bytes"] += actual_bytes
            if mode == "bundled":
                stats["bundled_bytes"] += actual_bytes
            elif mode == "cdn":
                stats["cdn_bytes"] += actual_bytes

        extra = on_disk - declared_names
        if extra:
            problems.append(f"{name}: {len(extra)} file(s) on disk not in manifest "
                            f"(e.g. {sorted(extra)[0]})")

        if not weights:
            problems.append(f"{name}: no weight markers found in file names")
        elif not ({"400"} <= weights):
            problems.append(f"{name}: no 400 face")
        elif not ({"600", "700"} & weights):
            problems.append(f"{name}: no 600/700 face")

        for field, actual in (("file_count", len(declared)),
                              ("total_bytes", sum(e.get("bytes", 0) for e in declared))):
            if field in pack and pack[field] != actual:
                problems.append(f"{name}: {field} {pack[field]} != recomputed {actual}")

    totals = data.get("totals", {})
    if totals:
        if totals.get("bundled_bytes") != stats["bundled_bytes"]:
            problems.append("totals.bundled_bytes does not match the files")
        if totals.get("cdn_bytes") != stats["cdn_bytes"]:
            problems.append("totals.cdn_bytes does not match the files")
        if totals.get("all_bytes") != stats["bytes"]:
            problems.append("totals.all_bytes does not match the files")

    if args.json:
        print(json.dumps({"packs": len(packs), **stats,
                          "problems": problems}, ensure_ascii=False, indent=2))
    else:
        for p in problems:
            print(f"  {p}", file=sys.stderr)
        if not args.quiet:
            mb = lambda n: f"{n / 1_000_000:.2f} MB"
            print(f"font gate: {len(packs)} pack(s) · {stats['files']} file(s) · "
                  f"bundled {mb(stats['bundled_bytes'])} · cdn {mb(stats['cdn_bytes'])}")

    if problems:
        print(f"font gate: {len(problems)} problem(s)", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
