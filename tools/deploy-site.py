#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
"""deploy-site.py — publish the Odina showcase site to Firebase Hosting.

The old script was 1136 lines of repair logic for symptoms: patch
firebase.json, guess which files are too big, add their paths to an ignore
list, restore the file afterwards, and retry twelve times when a hash
mismatch appears. Every one of those symptoms came from a single cause:

    it deployed the whole repository instead of a website.

This version removes the cause. It builds a staging directory from an
explicit manifest (site/site.json), checks it, and deploys only that. There
is no ignore list, no size heuristic, no firebase.json patching and no
restore step, because nothing outside site/ is ever handed to Firebase.

    python3 tools/deploy-site.py            # build + check + deploy
    python3 tools/deploy-site.py --dry-run  # build + check only
    python3 tools/deploy-site.py --serve    # build, then serve locally

Only publishing requires firebase-tools and `firebase login`. Building and
serving the staged site need only the Python standard library. Self-update
from git is gone: publishing must not be able to overwrite the working tree.

Exit codes: 0 ok, 1 failure, 2 usage/config error, 130 interrupted.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import time
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "site" / "site.json"
STAGE = ROOT / "site" / "_build"
FIREBASE_JSON = ROOT / "firebase.json"

# Hard ceiling for the whole site. The point is not the number: it is that a
# human notices when a 23 MB folder sneaks in, instead of the upload failing
# at 40% with "content hash doesn't match".
MAX_SITE_BYTES = 8 * 1024 * 1024
MAX_FILE_BYTES = 2 * 1024 * 1024


# ── console ────────────────────────────────────────────────────────────
class C:
    R = "\033[0m"; B = "\033[1m"; D = "\033[2m"
    RED = "\033[31m"; GRN = "\033[32m"; YEL = "\033[33m"
    BLU = "\033[34m"; CYN = "\033[36m"


def _p(sym: str, colour: str, msg: str) -> None:
    print(f"   {colour}{sym}{C.R} {msg}")


def ok(m: str) -> None:    _p("✔", C.GRN, m)
def info(m: str) -> None:  _p("·", C.BLU, m)
def warn(m: str) -> None:  _p("!", C.YEL, m)


def fail(m: str) -> None:
    _p("✖", C.RED, m)
    print(f"\n   {C.D}لم يُنشر شيء.{C.R}\n")
    sys.exit(1)


def head(title: str) -> None:
    line = "─" * 58
    print(f"\n{C.CYN}{line}{C.R}\n  {C.B}{title}{C.R}\n{C.CYN}{line}{C.R}")


def human(n: int) -> str:
    for unit in ("B", "KB", "MB", "GB"):
        if n < 1024:
            return f"{n:.1f} {unit}"
        n /= 1024
    return f"{n:.1f} TB"


# ── manifest ───────────────────────────────────────────────────────────
def load_manifest() -> dict:
    if not MANIFEST.exists():
        fail(f"{MANIFEST.relative_to(ROOT)} غير موجود — لا شيء لنشره.")
    try:
        data = json.loads(MANIFEST.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        fail(f"site/site.json غير صالح: {exc}")
    if not data.get("include"):
        fail("site/site.json لا يحتوي أي ملف في include.")
    return data


def expand(patterns: list[str]) -> list[Path]:
    """Expand globs against the repo. Later patterns do not override earlier."""
    seen: dict[Path, None] = {}
    for pattern in patterns:
        matches = sorted(ROOT.glob(pattern))
        if not matches:
            warn(f"النمط لا يطابق شيئًا: {pattern}")
        for path in matches:
            if path.is_file():
                seen.setdefault(path, None)
    return list(seen)


# ── build ──────────────────────────────────────────────────────────────
def build(manifest: dict) -> dict:
    head("1 · بناء مجلد النشر من المانيفست")
    if STAGE.exists():
        shutil.rmtree(STAGE)
    STAGE.mkdir(parents=True)

    files = expand(manifest["include"])
    if not files:
        fail("لم يُطابق أي ملف — تحقّق من المسارات في site/site.json.")

    rename = manifest.get("rename", {})
    fonts_base = (manifest.get("fonts_cdn_base") or "").strip()
    if fonts_base and not fonts_base.endswith("/"):
        fonts_base += "/"

    total = 0
    rewritten = 0
    for src in files:
        rel = src.relative_to(ROOT).as_posix()
        dest_rel = rename.get(rel, rel)
        dest = STAGE / dest_rel
        dest.parent.mkdir(parents=True, exist_ok=True)

        if dest.suffix.lower() in {".html", ".css"}:
            text = src.read_text(encoding="utf-8")
            if fonts_base:
                text, n = re.subn(r"fonts/vendor/", lambda _: fonts_base, text)
                rewritten += n
            else:
                # The disk preview can use vendored fonts. The default site ships
                # none: use the same CSS fallback stacks without a round of 404s.
                text = re.sub(r"<style data-vendor-fonts>.*?</style>\s*", "", text, flags=re.S)
            dest.write_text(text, encoding="utf-8")
            size = dest.stat().st_size
        else:
            shutil.copy2(src, dest)
            size = dest.stat().st_size

        total += size
        info(f"{dest_rel}  {C.D}{human(size)}{C.R}")

    return {
        "files": len(files),
        "bytes": total,
        "fonts_rewritten": rewritten,
        "fonts_base": fonts_base,
    }


# ── checks ─────────────────────────────────────────────────────────────
def check(stats: dict) -> None:
    head("2 · فحص ما سيُنشر فعليًا")

    problems: list[str] = []

    if (STAGE / "index.html").exists() is False:
        problems.append("index.html غير موجود — الموقع سيعرض صفحة فارغة.")

    for path in STAGE.rglob("*"):
        if not path.is_file():
            continue
        rel = path.relative_to(STAGE).as_posix()
        size = path.stat().st_size
        if size > MAX_FILE_BYTES:
            problems.append(f"ملف كبير غير متوقع: {rel} ({human(size)})")

    if stats["bytes"] > MAX_SITE_BYTES:
        problems.append(
            f"حجم الموقع {human(stats['bytes'])} يتجاوز السقف "
            f"{human(MAX_SITE_BYTES)} — هل أُضيفت مجلدات لا تنتمي للموقع؟")

    # no source or tooling must ever reach the public site
    forbidden = {".git", "node_modules", "fonts", "docs", "tools",
                 "locales", "__pycache__"}
    for path in STAGE.rglob("*"):
        parts = set(path.relative_to(STAGE).parts)
        hit = parts & forbidden
        if hit:
            problems.append(f"مجلد لا يجب نشره: {sorted(hit)[0]}/")

    # broken local references in the shipped HTML
    for html in STAGE.rglob("*.html"):
        text = html.read_text(encoding="utf-8")
        for ref in re.findall(r'(?:src|href)="([^"#:]+)"', text):
            if ref.startswith(("http", "//", "mailto:", "data:")):
                continue
            target = (html.parent / ref).resolve()
            try:
                target.relative_to(STAGE)
            except ValueError:
                problems.append(f"{html.name}: مرجع خارج المجلد ({ref})")
                continue
            if not target.exists():
                problems.append(f"{html.name}: مرجع مكسور ({ref})")

    if problems:
        seen = set()
        for p in problems:
            if p in seen:
                continue
            seen.add(p)
            print(f"   {C.RED}✖{C.R} {p}", file=sys.stderr)
        fail(f"{len(seen)} مشكلة في الموقع المبني")

    ok(f"index.html موجود")
    ok(f"لا مجلدات مصدر أو خطوط مضمّنة")
    ok(f"كل المراجع المحلية تُحلّ")
    ok(f"{stats['files']} ملفًا · {human(stats['bytes'])} "
       f"(السقف {human(MAX_SITE_BYTES)})")
    if stats["fonts_base"]:
        ok(f"روابط الخطوط تُعاد كتابتها إلى {stats['fonts_base']} "
           f"({stats['fonts_rewritten']} موضعًا)")
    else:
        info("لا خطوط مضمنة — الواجهة تستخدم خطوط النظام (وهذا هو التصميم).")


# ── firebase.json ──────────────────────────────────────────────────────
def write_firebase_json() -> None:
    """One small, complete config. Nothing is patched or restored later."""
    head("3 · تهيئة إعدادات Firebase")
    config = {
        "hosting": {
            "public": "site/_build",
            "ignore": ["firebase.json", "**/.*", "**/node_modules/**"],
            "cleanUrls": True,
            "headers": [
                {
                    "source": "**/*.@(svg|png|ico|woff2)",
                    "headers": [
                        {"key": "Cache-Control",
                         "value": "public, max-age=604800"},
                    ],
                },
                {
                    "source": "**",
                    "headers": [
                        {"key": "X-Content-Type-Options", "value": "nosniff"},
                        {"key": "Referrer-Policy", "value": "no-referrer"},
                    ],
                },
            ],
        }
    }
    FIREBASE_JSON.write_text(
        json.dumps(config, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    ok("firebase.json مكتوب (public = site/_build)")
    info("هذا الملف جزء من المستودع — لا يُعدَّل سرًا ثم يُستعاد.")


# ── deploy ─────────────────────────────────────────────────────────────
def require_firebase() -> None:
    if shutil.which("firebase") is None:
        fail("firebase غير مثبّت. ثبّته بـ: npm i -g firebase-tools")


def deploy(project: str | None, concurrency: int) -> None:
    head("4 · النشر")
    require_firebase()

    cmd = ["firebase", "deploy", "--only", "hosting", "--non-interactive"]
    if project:
        cmd += ["--project", project]

    env = os.environ.copy()
    env["FIREBASE_HOSTING_UPLOAD_CONCURRENCY"] = str(concurrency)

    started = time.time()
    result = subprocess.run(cmd, cwd=ROOT, env=env, text=True)
    elapsed = time.time() - started

    if result.returncode != 0:
        fail(f"فشل النشر بعد {elapsed:.0f} ثانية")

    ok(f"نُشر في {elapsed:.0f} ثانية")


# ── serve ──────────────────────────────────────────────────────────────
def serve(host: str, port: int) -> None:
    head("معاينة محلية — بلا نشر")
    handler = partial(SimpleHTTPRequestHandler, directory=str(STAGE))
    with ThreadingHTTPServer((host, port), handler) as server:
        info(f"http://{host}:{server.server_port}  — أوقف بـ Ctrl+C")
        server.serve_forever()


# ── main ───────────────────────────────────────────────────────────────
def main() -> int:
    ap = argparse.ArgumentParser(description="نشر موقع Odina")
    ap.add_argument("--dry-run", action="store_true",
                    help="بناء وفحص بلا نشر")
    ap.add_argument("--serve", action="store_true",
                    help="بناء ثم معاينة محلية")
    ap.add_argument("--bind", default="127.0.0.1", help="عنوان الاستماع للمعاينة فقط")
    ap.add_argument("--port", type=int, default=5000)
    ap.add_argument("--project", default=None,
                    help="معرّف مشروع Firebase (اختياري)")
    ap.add_argument("--concurrency", type=int, default=2,
                    help="عدد الرفع المتزامن (افتراضي 2)")
    args = ap.parse_args()

    print(f"\n{C.B}Odina — نشر الموقع{C.R}")
    print(f"  {C.D}المجلد:{C.R} {ROOT}")
    print(f"  {C.D}المانيفست:{C.R} site/site.json")

    manifest = load_manifest()
    stats = build(manifest)
    check(stats)

    if args.serve:
        serve(args.bind, args.port)
        return 0

    if args.dry_run:
        head("تم — Dry run")
        info("لم يُنشر شيء. أزل --dry-run للنشر الفعلي.")
        return 0

    write_firebase_json()
    deploy(args.project, args.concurrency)
    print(f"\n{C.GRN}{C.B}تم.{C.R}  site/_build لم يُحذف — راجعه إن أردت.\n")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except KeyboardInterrupt:
        print("\n\nأُلغي.")
        sys.exit(130)
