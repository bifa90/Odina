#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
"""Manual A/B for the deferred-PE change in PR 13. No checkout changes or uploads.
Run outside the scan folders: python3 verify-b9.py [--quick] DIR [DIR ...]
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

REVS = {"before": "51d7984d00ef08982212f3c1ca5e9f5761ae319a",
        "after": "96aa0228465133afb113eaf573cd9258faed85c3"}
CARGO = ["--release", "--locked", "-p", "odina-cli", "--features", "instrument"]


def build(label, work):
    rev = REVS[label]
    src, target = work / label, work / (label + "-build")
    print(f"\n{label}: {rev} — تنزيل المصدر وبناء نسخة القياس", flush=True)
    package = work / (label + ".zip")
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
    args = [*CARGO, "--target-dir", str(target)]
    subprocess.run(["cargo", "build", *args], cwd=src, check=True)
    if label == "after":
        # The test actually invokes scan/check; selftest alone only checks hashes.
        subprocess.run(["cargo", "test", *args, "--test", "g1"], cwd=src, check=True)
    return target / "release" / ("odina.exe" if os.name == "nt" else "odina")


def scan(binary, folder, output):
    rc = subprocess.run([str(binary), "check", str(folder), "--out", str(output)]).returncode
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
    return signature, {"seconds": elapsed, "files": s["files"], "bytes": s["bytes"],
                       "files_per_sec": metrics["throughput_files_per_sec"].get("value"),
                       "cpu_percent_one_core": cpu.get("value"), "B9": cpu["status"]}


def measure(index, folder, binaries, results, quick):
    out = results / f"disk-{index}"
    out.mkdir()
    rows = {key: [] for key in REVS}
    # One uncounted priming scan, then AB / BA / AB. Quick is exploratory only.
    order = ["before", "after"] if quick else ["warmup", "before", "after", "after", "before", "before", "after"]
    first = None
    for num, tag in enumerate(order, 1):
        print(f"\ndisk-{index}: {num}/{len(order)} {tag}", flush=True)
        sig, row = scan(binaries["before" if tag == "warmup" else tag], folder, out / f"{num}-{tag}.json")
        if first is not None and sig != first:
            raise ValueError(f"disk-{index}: تغيرت العدّادات أو الأحكام أو تقارير التنبيه؛ لا تقارن السرعة.")
        first = sig
        if tag != "warmup":
            rows[tag].append(row)
    data = {"runs": rows, "quick": quick, "counts_and_alerts_match": True}
    def column(tag, name):
        return [r[name] for r in rows[tag]]
    times = {k: column(k, "seconds") for k in REVS}
    for tag in REVS:
        cpu = column(tag, "cpu_percent_one_core")
        cpustr = "غير مقيس" if None in cpu else f"{median(cpu):.2f}%"
        print(f"{tag}: {median(times[tag]):.3f} s (min–max {min(times[tag]):.3f}–{max(times[tag]):.3f}); CPU: {cpustr}")
    a, b = map(median, [times["before"], times["after"]])
    if a > 0 and b > 0:
        data["time_reduction_percent"] = round(100 * (1 - b / a), 2)
        print(f"تقليص زمن الفحص: {data['time_reduction_percent']:+.2f}% (السالب = أبطأ)")
    too_short = min(times["before"] + times["after"]) < 1
    noisy = any(max(t) - min(t) > 0.1 * median(t) for t in times.values())
    data["speed_observed"] = "inconclusive"
    data["B9_this_workload"] = "inconclusive"
    if quick or too_short:
        print("نتيجة استطلاعية فقط: استخدم مجلدًا ممثلًا والوضع الكامل لإثبات الأداء.")
    elif noisy:
        print("التفاوت يتجاوز 10%؛ أعد القياس.")
    else:
        if max(times["after"]) < min(times["before"]):
            data["speed_observed"] = "faster"
            print("الجديدة أسرع في جميع هذه الجولات، مع نفس العدّادات والتنبيهات.")
        elif max(times["before"]) < min(times["after"]):
            data["speed_observed"] = "slower"
            print("الجديدة أبطأ في جميع هذه الجولات.")
        else:
            print("نطاقا الأزمنة متداخلان؛ فرق الوسيط وحده لا يحسم التحسّن.")
        status = column("after", "B9")
        if "fail" in status:
            data["B9_this_workload"] = "fail"
        elif all(r["B9"] == "pass" and r["cpu_percent_one_core"] is not None
                 and r["cpu_percent_one_core"] < 3 for r in rows["after"]):
            data["B9_this_workload"] = "pass"
    print("B9 لهذا الحمل فقط:", data["B9_this_workload"], "(المطلوب أقل من 3% من نواة واحدة)")
    return data


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--quick", action="store_true", help="زوج واحد فقط؛ نتيجة استطلاعية")
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
            meta["disks"][f"disk-{i}"] = measure(i, folder, binaries, results, args.quick)
            out.write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")
    print("\nملخص بلا مسارات الملفات:", out)


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, BadZipFile, subprocess.CalledProcessError) as exc:
        sys.exit(f"توقف التحقق: {exc}")
