#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
"""Before/after B9: deferred PE vs the prefix Bloom + private index (empty and filled).
Run outside the scan folders: python3 verify-b9.py [--quick] [--cache-state cold|warm|both] DIR [DIR ...]
Requires stable Rust/Cargo, Python 3.8+, and network for the pinned sources/deps.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import shutil
from statistics import median
import subprocess
import sys
import tempfile
from urllib.request import urlopen
from zipfile import ZipFile, BadZipFile

REVS = {"before": "96aa0228465133afb113eaf573cd9258faed85c3",
        "after": "e5e48fb958ef38e073eb51807852c6f0dcf97cf9"}
CARGO = ["--release", "--locked", "-p", "odina-cli", "--features", "instrument"]


def fetch_source(rev, src, package):
    with urlopen(f"https://codeload.github.com/bifa90/Odina/zip/{rev}", timeout=90) as resp, package.open("wb") as out:
        shutil.copyfileobj(resp, out)
    with ZipFile(package) as archive:
        for item in archive.infolist():
            parts = PurePosixPath(item.filename).parts
            if not parts or parts[0] != "Odina-" + rev or ".." in parts or any(c in item.filename for c in "\\:"):
                raise ValueError("اسم غير متوقع داخل أرشيف المصدر")
            # Only files used by the locked workspace and its regression tests.
            if item.is_dir() or len(parts) < 2 or parts[1] not in {"Cargo.toml", "Cargo.lock", "crates", "components", "tests"}:
                continue
            dest = src.joinpath(*parts[1:])
            dest.parent.mkdir(parents=True, exist_ok=True)
            with archive.open(item) as inp, dest.open("xb") as out:
                shutil.copyfileobj(inp, out)


def build(label, work):
    rev = REVS[label]
    src, target = work / label, work / (label + "-build")
    print(f"\n{label}: {rev} — تنزيل المصدر وبناء نسخة القياس", flush=True)
    fetch_source(rev, src, work / (label + ".zip"))
    args = [*CARGO, "--target-dir", str(target)]
    subprocess.run(["cargo", "build", *args], cwd=src, check=True)
    if label == "after":
        # The test actually invokes scan/check; selftest alone only checks hashes.
        subprocess.run(["cargo", "test", *args, "--test", "g1"], cwd=src, check=True)
    return target / "release" / ("odina.exe" if os.name == "nt" else "odina")


def scan(binary, folder, output, cache_dir=None):
    cmd = [str(binary), "check", str(folder), "--out", str(output)]
    if cache_dir is not None:  # NEVER pass the option to the old binary
        cmd.extend(["--cache-dir", str(cache_dir)])
    rc = subprocess.run(cmd).returncode
    if rc not in (0, 2):  # 2 is a detection, not a timing failure
        raise ValueError(f"فشل check برمز {rc}: {output.name}")
    doc = json.loads(output.read_text(encoding="utf-8"))
    s = doc["scan"].copy()
    elapsed = s.pop("elapsed_ms") / 1000
    if doc["schema"] != "odina-perf/v1" or not s["files"] or s["truncated"] or s["unreadable"] or s["read_errors"]:
        raise ValueError(f"فحص فارغ أو ناقص أو به أخطاء: {output.name}")
    # Counts (including verdicts, bytes, links), alert fields and exit code.
    signature = hashlib.sha256(json.dumps([s, doc["reports"], rc], sort_keys=True).encode()).hexdigest()
    metrics = {m["gate"]: m for m in doc["metrics"]}
    cpu = metrics["cpu_scan_avg_percent"]
    pct = cpu.get("value")
    row = {"seconds": elapsed, "files": s["files"], "bytes": s["bytes"],
           "files_per_sec": metrics["throughput_files_per_sec"].get("value"),
           "cpu_percent_one_core": pct, "B9": cpu["status"],
           "cpu_seconds_approx": None if pct is None else round(elapsed * pct / 100, 5)}
    if cache_dir is not None:
        w = doc["work"]
        if (any(type(w[k]) is not int or w[k] < 0 for k in ("sha_files", "bloom_negatives", "cache_hits", "streamed_bytes"))
                or w["sha_files"] + w["bloom_negatives"] + w["cache_hits"] != s["files"]
                or w["streamed_bytes"] > s["bytes"]
                or (not w["cache_hits"] and w["streamed_bytes"] != s["bytes"])):
            raise ValueError(f"عدّادات عمل غير قابلة للمقارنة أو قراءة أُعيدت أثناء تعديل: {output.name}")
        row["work"] = w
        # Conservative even in a mixed corpus: some SHA calls can be for alerts.
        row["clean_sha_skip_lower_bound_percent"] = (
            (100_000 * max(0, s["clean"] - w["sha_files"]) // s["clean"]) / 1000 if s["clean"] else None)
    return signature, row


def compare(before, after, quick):
    times = [[r["seconds"] for r in group] for group in (before, after)]
    a, b = map(median, times)
    data = {"median_seconds": b, "baseline_median_seconds": a,
            "time_reduction_percent": round(100 * (1 - b / a), 2) if a and b else None,
            "speed_observed": "inconclusive", "B9_this_workload": "inconclusive"}
    work = [[r["cpu_seconds_approx"] for r in group] for group in (before, after)]
    for group, cpu_work, prefix in zip((before, after), work, ("baseline_", "")):
        util = [r["cpu_percent_one_core"] for r in group]
        data[prefix + "median_cpu_percent_one_core"] = None if None in util else median(util)
        data[prefix + "median_cpu_seconds_approx"] = None if None in cpu_work else median(cpu_work)
    if all(c is not None and c > 0 for group in work for c in group):
        data["cpu_seconds_reduction_percent"] = round(100 * (1 - median(work[1]) / median(work[0])), 2)
    noisy = any(max(t) - min(t) > 0.1 * median(t) for t in times)
    reason = ("quick" if quick else "under_one_second" if min(times[0] + times[1]) < 1 else
              "time_spread_over_10_percent" if noisy else None)
    failed = sum(r["B9"] == "fail" for r in after)
    data["B9_failed_runs"] = failed
    if reason:
        data["qualification_limit"] = reason
    else:
        if max(times[1]) < min(times[0]):
            data["speed_observed"] = "faster"
        elif max(times[0]) < min(times[1]):
            data["speed_observed"] = "slower"
        if failed:
            data["B9_this_workload"] = "fail"
        elif all(r["B9"] == "pass" and r["cpu_percent_one_core"] is not None and
                 r["cpu_percent_one_core"] < 3 for r in after):
            data["B9_this_workload"] = "pass"
    bounds = [r["clean_sha_skip_lower_bound_percent"] for r in after]
    data["clean_sha_skip_lower_bound_percent_min"] = None if None in bounds else min(bounds)
    saving = data["time_reduction_percent"]
    change = "n/a" if saving is None else f"{saving:+.2f}%"
    print(f"{a:.3f} [{min(times[0]):.3f}–{max(times[0]):.3f}] → "
          f"{b:.3f} [{min(times[1]):.3f}–{max(times[1]):.3f}] s; time saving: {change}; "
          f"{data['speed_observed']} ({reason or 'full runs'})")
    print(f"CPU-s (approx): {data['baseline_median_cpu_seconds_approx']} → {data['median_cpu_seconds_approx']}; "
          f"CPU % (one core): {data['baseline_median_cpu_percent_one_core']} → {data['median_cpu_percent_one_core']}; "
          f"B9: {data['B9_this_workload']} ({failed}/{len(after)} failed)")
    print("SHA files / cache hits / streamed bytes:", [(r["work"]["sha_files"], r["work"]["cache_hits"],
                                                        r["work"]["streamed_bytes"]) for r in after])
    print("Clean SHA skip %, lower bound:", data["clean_sha_skip_lower_bound_percent_min"])
    return data


def measure(index, folder, binaries, results, quick, cache_state="both"):
    out = results / f"disk-{index}"
    out.mkdir()
    phases = ["cold", "warm"] if cache_state == "both" else [cache_state]
    arms = ["before", *phases]
    rows = {tag: [] for tag in arms}
    # Prime OS pages for the ordinary comparison. An empty index is not the
    # same as a physically cold drive. Rotate arms; never pool cold and warm.
    order = [] if quick else ["prime-os"]
    if phases == ["warm"]:
        order.append("seed-index")
    for i in range(1 if quick else 3):
        j = i % len(arms)
        order.extend(arms[j:] + arms[:j])
    first = None
    with tempfile.TemporaryDirectory(prefix="private-", dir=out) as scratch:
        warm = Path(scratch) / "index"
        for n, tag in enumerate(order, 1):
            print(f"\ndisk-{index}: {n}/{len(order)} {tag}", flush=True)
            base = tag in ("before", "prime-os")
            cache = None if base else warm
            if tag == "cold":
                # The first measured cold pass also seeds the warm arm. Each
                # further cold pass gets a new name, including on failed writes.
                cache = warm if not rows["cold"] else Path(scratch) / f"empty-{n}"
            sig, row = scan(binaries["before" if base else "after"], folder, out / f"{n}-{tag}.json", cache)
            if first is not None and sig != first:
                raise ValueError(f"disk-{index}: تغيرت العدّادات أو الأحكام أو تقارير التنبيه؛ لا تقارن السرعة.")
            first = sig
            if tag == "cold" and row["work"]["cache_hits"]:
                raise ValueError("كان يجب أن يبدأ هذا الذراع بفهرس فارغ")
            if tag in rows:
                rows[tag].append(row)
    data = {"runs": rows, "quick": quick, "counts_and_alerts_match": True}
    for phase in phases:
        print(f"\ndisk-{index} / {phase}:")
        data[phase] = compare(rows["before"], rows[phase], quick)
        if phase == "warm" and not any(r["work"]["cache_hits"] for r in rows[phase]):
            print("لا إصابة للكاش هنا؛ تحقق من نظام الملفات والأحجام. لا تنسب المكسب للفهرس.")
    return data


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--quick", action="store_true", help="جولة واحدة لكل ذراع؛ نتيجة استطلاعية")
    p.add_argument("--cache-state", choices=("both", "cold", "warm"), default="both",
                   help="حالة فهرس Odina، لا ذاكرة صفحات النظام")
    p.add_argument("folders", nargs="+", type=Path, metavar="DIR")
    args = p.parse_args()
    folders = [f.resolve() for f in args.folders]
    for folder in folders:
        if not folder.is_dir():
            p.error(f"ليس مجلدًا متاحًا: {folder}")
        if folder == Path.cwd() or folder in Path.cwd().parents:
            p.error("ابدأ من مجلد عمل خارج كل مجلد تفحصه، كي لا تفحص البرنامج أو نتائجه.")
    if not shutil.which("cargo") or not shutil.which("rustc"):
        p.error("يلزم Rust/Cargo للبناء. لا حاجة لإعادة تنزيل المشروع يدويًا.")
    results = Path(tempfile.mkdtemp(prefix="odina-b9-", dir=Path.cwd()))
    print("النتائج المحلية:", results, flush=True)
    meta = {"commits": REVS, "disks": {}}
    for tool in ("rustc", "cargo"):
        meta[tool] = subprocess.check_output([tool, "--version"], text=True).strip()
    with tempfile.TemporaryDirectory(dir=results) as tmp:
        binaries = {tag: build(tag, Path(tmp)) for tag in REVS}
        meta["binary_sha256"] = {tag: hashlib.sha256(path.read_bytes()).hexdigest() for tag, path in binaries.items()}
        if len(set(meta["binary_sha256"].values())) != 2:
            raise ValueError("بصمة الملفّين التنفيذيين واحدة؛ هذه ليست مقارنة بين نسختين.")
        out = results / "comparison.json"
        out.write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")
        for i, folder in enumerate(folders, 1):
            meta["disks"][f"disk-{i}"] = measure(i, folder, binaries, results, args.quick, args.cache_state)
            out.write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")
    print("\nملخص بلا مسارات الملفات:", out)


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, BadZipFile, subprocess.CalledProcessError) as exc:
        sys.exit(f"توقف التحقق: {exc}")
