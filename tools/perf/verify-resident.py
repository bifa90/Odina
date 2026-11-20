#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
"""External Linux/Windows observer. Run with a release instrumented binary, psutil, and a stable test corpus.
No attachment to an existing process; no writes inside the corpus. No IPC during idle sampling.
"""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import secrets
import subprocess
import sys
import tempfile
import time


def calls(binary, *args, allowed=(0,), timeout=10):
    r = subprocess.run([str(binary), *map(str, args)], stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                       timeout=timeout, encoding="utf-8", errors="replace")
    if r.returncode not in allowed:
        raise ValueError(f"{args[0]} exited {r.returncode}; no performance conclusion")
    return r.returncode, r.stdout


def signature(report, code):
    s = dict(report["scan"])
    s.pop("elapsed_ms")
    if (not s["files"] or s["read_errors"] or s["unreadable"] or s["truncated"] or
            report["schema"] != "odina-perf/v1"):
        raise ValueError("empty, partial or unreadable scan")
    return s, report["reports"], code


def high_water(pid, info):
    if sys.platform == "linux":
        for line in Path(f"/proc/{pid}/status").read_text().splitlines():
            if line.startswith("VmHWM:"):
                return int(line.split()[1]) * 1024
    return getattr(info, "peak_wset", None)


def sample(p):
    cpu = p.cpu_times()
    ram = p.memory_info()
    io = p.io_counters()
    return (cpu.user + cpu.system, ram.rss, io.read_count, io.write_count, high_water(p.pid, ram))


def idle(p, seconds):
    begin = sample(p)
    start = last = time.monotonic()
    civil = time.time()
    rss, gap, samples = begin[1], 0.0, 1
    # The ONLY periodic wakeup is in this observer, not the resident.
    while last - start < seconds:
        time.sleep(min(1, seconds - (last - start)))
        end = sample(p)
        now = time.monotonic()
        gap = max(gap, now - last)
        last = now
        rss = max(rss, end[1])
        samples += 1
    wall = last - start
    cpu = end[0] - begin[0]
    if min(cpu, end[2] - begin[2], end[3] - begin[3]) < 0:
        raise ValueError("process counters moved backwards")
    disturbed = gap > 2.5 or abs((time.time() - civil) - wall) > 2.5
    window = "interrupted" if disturbed else "smoke_only" if seconds < 600 else "ten_minutes_or_longer"
    qualified = window == "ten_minutes_or_longer"
    return {"wall_seconds": wall, "process_cpu_seconds": cpu, "cpu_percent_one_core": 100 * cpu / wall,
            "zero_cpu_delta_at_counter_resolution": cpu == 0, "peak_sampled_rss_bytes": rss,
            "process_high_water_bytes": end[4], "high_water_below_50_decimal_MB":
            None if end[4] is None else end[4] < 50_000_000,
            "read_ops_delta": end[2] - begin[2], "write_ops_delta": end[3] - begin[3],
            "samples": samples, "max_sample_gap_seconds": gap,
            "window": window,
            "idle_cpu_this_window": ("pass" if cpu == 0 else "fail") if qualified else "inconclusive",
            "memory_this_workload": ("pass" if end[4] < 50_000_000 else "fail")
            if qualified and end[4] is not None else "inconclusive"}


def measure(args, psutil):
    binary, root, out = args.binary.resolve(strict=True), args.root.resolve(strict=True), args.out.absolute()
    parent = out.parent.resolve(strict=True)
    if (root == parent or root in parent.parents or root == out.resolve() or out.is_symlink() or out.exists()
            or not (root.is_dir() or root.is_file())):
        raise ValueError("output must be a NEW file outside a regular scan target")
    name = "measure-" + secrets.token_hex(8)
    result = {"schema": "odina-resident-lab/v1", "platform": sys.platform, "observer_psutil": psutil.__version__,
              "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
              "linux_clock_ticks_per_second": os.sysconf("SC_CLK_TCK") if sys.platform == "linux" else None,
              "event_to_invalidation_ms": None, "event_to_verdict_ms": None,
              "event_latency_status": "unmeasured: no filesystem watcher in skeleton"}
    with tempfile.TemporaryDirectory(dir=parent) as tmp:
        tmp = Path(tmp)
        def scan(tag, remote):
            f = tmp / (tag + ".json")
            extra = ["--service", name] if remote else []
            rc, _ = calls(binary, "check", root, "--out", f, *extra, allowed=(0, 2), timeout=3600)
            return signature(json.loads(f.read_text(encoding="utf-8")), rc)
        base = scan("reference", False)
        with subprocess.Popen([str(binary), "serve", str(root), "--service", name, "--cache-dir", str(tmp / "index")],
                              stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL) as child:
            def status():
                return json.loads(calls(binary, "status", "--service", name)[1])
            try:
                ready_by = time.monotonic() + 15
                while child.poll() is None:
                    try:
                        if status()["phase"] == "idle":
                            break
                    except ValueError:
                        pass
                    if time.monotonic() >= ready_by:
                        raise ValueError("resident did not become ready")
                    time.sleep(0.05)  # startup only
                if child.poll() is not None:
                    raise ValueError("resident exited before measurement")
                for tag in ("initial", "repeat"):
                    if base != scan(tag, True):
                        raise ValueError("counts, reports or exit code changed; rejecting sample")
                result["counts_and_alerts_match"] = True
                result["files"], result["bytes"] = base[0]["files"], base[0]["bytes"]
                before = status()
                if before["phase"] != "idle" or before["completed"] != 2 or before["watching"]:
                    raise ValueError("unexpected job state for this skeleton")
                result["idle"] = idle(psutil.Process(child.pid), args.idle_seconds)
                if status() != before:
                    raise ValueError("resident state changed during idle window")
                rtts = []
                for _ in range(30):
                    t = time.perf_counter_ns()
                    if status() != before:
                        raise ValueError("unexpected status")
                    rtts.append((time.perf_counter_ns() - t) / 1_000_000)
                rtts.sort()
                result["status_cli_round_trip_ms"] = {k: rtts[math.ceil(q * len(rtts)) - 1] for
                                                      k, q in (("p50", .50), ("p95", .95), ("p99", .99), ("max", 1))}
                calls(binary, "stop", "--service", name)
                if child.wait(timeout=10) != 0:
                    raise ValueError("unclean service shutdown")
            finally:
                if child.poll() is None:
                    child.kill()  # only the child this tool created
                    child.wait()
    with out.open("x", encoding="utf-8") as f:
        json.dump(result, f, indent=2, allow_nan=False)
        f.write("\n")
    print(json.dumps(result, indent=2))


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--binary", required=True, type=Path)
    p.add_argument("--root", required=True, type=Path)
    p.add_argument("--out", required=True, type=Path)
    p.add_argument("--idle-seconds", type=int, default=600)
    args = p.parse_args()
    if args.idle_seconds < 1 or sys.platform not in ("linux", "win32"):
        p.error("need Linux or Windows and at least one second")
    try:
        import psutil
    except ImportError:
        p.error("install psutil==7.2.2 in the observer's Python environment (not in the service)")
    try:
        measure(args, psutil)
    except (OSError, ValueError, KeyError, psutil.Error, subprocess.TimeoutExpired) as e:
        p.exit(1, f"measurement stopped: {e}\n")


if __name__ == "__main__":
    main()
