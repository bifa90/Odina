#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
"""External Linux/Windows observer. Run with a release instrumented binary, psutil, and a stable test corpus.
Owns its child. No IPC during idle. Only --probe-changes writes a disposable benign file in a new private test directory inside the root.
"""
import argparse
from contextlib import contextmanager
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
            "idle_cpu_this_window": ("pass" if 100 * cpu / wall < 0.1 else "fail") if qualified else "inconclusive",
            "memory_this_workload": ("pass" if end[4] < 50_000_000 else "fail")
            if qualified and end[4] is not None else "inconclusive"}


def percentiles(values):
    values = sorted(values)
    return {k: values[math.ceil(q * len(values)) - 1] if values else None
            for k, q in (("p50", .5), ("p95", .95), ("p99", .99), ("max", 1))}


@contextmanager
def probe_dir(root, enabled):
    if not enabled:
        yield None
        return
    if not root.is_dir():
        raise ValueError("--probe-changes needs a writable directory, not a file")
    # Install the empty directory before subscription; never pick/modify an owner's file.
    d = Path(tempfile.mkdtemp(prefix="odina-probe-", dir=root))
    try:
        yield d
    finally:
        d.rmdir()  # never recursively remove unexpected entries


def wait_ready(child, status, timeout=3600):
    by = time.monotonic() + timeout
    while child.poll() is None:
        try:
            s = status()
        except ValueError:  # binding has not finished yet
            s = None
        if s and s["watching"] and s["phase"] == "idle":
            return s
        if s and s["monitor"] in ("offline", "incomplete_inventory"):
            raise ValueError(f"resident coverage unknown: {s['monitor']}")
        if time.monotonic() >= by:
            raise ValueError("no current, quiescent inventory before deadline")
        time.sleep(.05)
    raise ValueError("owned resident exited before measurement")


# Python 3.11+: QPC on Windows, CLOCK_MONOTONIC on Linux, shared with the native publisher.
# In CPython 3.11/3.12 on Windows, monotonic_ns uses GetTickCount64 (NOT QPC).
# perf_counter_ns is the system-wide QPC on supported Windows Python versions.
CLOCK_NAME = "perf_counter" if sys.platform == "win32" else "monotonic"
clock = time.perf_counter_ns if sys.platform == "win32" else time.monotonic_ns


def witnessed(get, start, state, size, prior_gen, timeout_ns):
    deadline = start + timeout_ns
    while True:
        r = get()
        now = clock()
        if (r.get("origin") == "event" and r["state"] == state and r["bytes"] == size
                and r["verdict"] == ("clean" if state == "complete" else None)
                and r["generation"] > prior_gen and r["received_ns"] >= start):
            a, b, c = (r[k] for k in ("received_ns", "invalidated_ns", "published_ns"))
            if not start <= a <= b <= c <= now:
                raise ValueError("timestamp order/clock domain mismatch; no latency claim")
            if c <= deadline:
                return r
        if now >= deadline:
            return None
        time.sleep(.001)  # retrieval only; not included in the publisher's timestamp


def event_summary(rows):
    seen = [r for r in rows if r["published_ns"] is not None]
    def span(end, begin):
        return percentiles([(r[end] - r[begin]) / 1e6 for r in seen])
    return {"attempted": len(rows), "witnessed": len(seen), "missed_by_deadline": len(rows) - len(seen),
            "unwitnessed_percent": 100 * (len(rows) - len(seen)) / len(rows),
            "at_or_above_10_ms": sum(r["published_ns"] - r["t0_ns"] >= 10_000_000 for r in seen),
            "mutation_to_publication_ms": span("published_ns", "t0_ns"),
            "mutation_to_invalidation_ms": span("invalidated_ns", "t0_ns"),
            "mutation_to_receipt_ms": span("received_ns", "t0_ns"),
            "receipt_to_invalidation_ms": span("invalidated_ns", "received_ns"),
            "invalidation_to_publication_ms": span("published_ns", "invalidated_ns")}


def events(binary, name, root, d, count, timeout):
    # One logical mutation at a time. Distinct lengths + generation + T1>=T0 + expected
    # verdict prevent a late receipt, an intermediate empty CREATE, or old row from passing.
    p = d / "probe.dat"
    key = p.relative_to(root)
    def get():
        return json.loads(calls(binary, "verdict", key, "--service", name)[1])
    rows = {k: [] for k in ("create", "modify", "delete")}
    gen = 0
    owned = False
    try:
        for _ in range(count):
            for kind, size in (("create", 64), ("modify", 65), ("delete", 0)):
                if kind == "create":
                    start = clock()  # conservative: immediately BEFORE the mutating syscall
                    with p.open("xb", buffering=0) as handle:
                        owned = True
                        if handle.write(b"A" * 64) != 64:
                            raise OSError("short probe write")
                elif kind == "modify":
                    with p.open("ab", buffering=0) as handle:
                        start = clock()
                        if handle.write(b"B") != 1:
                            raise OSError("short append")
                else:
                    start = clock()
                    p.unlink()
                    owned = False
                returned = clock()  # retain syscall/close interval; never start timer after publication
                r = witnessed(get, start, "deleted" if kind == "delete" else "complete", size, gen, int(timeout * 1e9))
                fields = ("epoch", "generation", "received_ns", "invalidated_ns", "published_ns")
                rows[kind].append({"t0_ns": start, "mutation_return_ns": returned,
                                   **{k: r[k] if r else None for k in fields}})
                if r:
                    gen = r["generation"]
    finally:
        if owned:
            p.unlink(missing_ok=True)
    return {k: event_summary(v) for k, v in rows.items()}, rows


def gates(result):
    idle_windows = result["idle_windows"]
    tags = [w[k] for w in idle_windows for k in ("idle_cpu_this_window", "memory_this_workload")]
    ig = "fail" if "fail" in tags else "pass" if len(idle_windows) >= 3 and all(t == "pass" for t in tags) else "inconclusive"
    eg = "not_measured"
    if result.get("events"):
        groups = result["events"].values()
        if (not result["isolated_event_window"] or any(v["missed_by_deadline"] or v["at_or_above_10_ms"] for v in groups)):
            eg = "fail"
        elif min(v["attempted"] for v in groups) >= 1000:
            eg = "pass"
        else:
            eg = "inconclusive"
    peak = result["process_high_water_after_workload_bytes"]
    total = "fail" if "fail" in (eg, ig) or (peak is not None and peak >= 50_000_000) else (
        "pass" if eg == ig == "pass" and peak is not None else "inconclusive")
    if not result["corpus_matches_expected"]:
        total = "inconclusive"
    return {"idle_three_windows": ig, "strict_response": eg, "workload": total}


def measure(args, psutil):
    binary, root, out = args.binary.resolve(strict=True), args.root.resolve(strict=True), args.out.absolute()
    parent = out.parent.resolve(strict=True)
    if (root == parent or root in parent.parents or root == out.resolve() or out.is_symlink() or out.exists()
            or not (root.is_dir() or root.is_file())):
        raise ValueError("output must be a NEW file outside a regular scan target")
    name = "measure-" + secrets.token_hex(8)
    result = {"schema": "odina-resident-lab/v2", "label": args.label, "platform": sys.platform,
              "observer_psutil": psutil.__version__, "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
              "linux_clock_ticks_per_second": os.sysconf("SC_CLK_TCK") if sys.platform == "linux" else None,
              "clock": time.get_clock_info(CLOCK_NAME).implementation, "event_deadline_seconds": args.event_timeout,
              "expected_files": args.expect_files, "expected_bytes": args.expect_bytes, "idle_windows": []}
    with tempfile.TemporaryDirectory(dir=parent) as tmp, probe_dir(root, args.probe_changes) as d:
        tmp = Path(tmp)
        def scan(tag, remote):
            f = tmp / (tag + ".json")
            extra = ["--service", name] if remote else []
            rc, _ = calls(binary, "check", root, "--out", f, *extra, allowed=(0, 2), timeout=3600)
            return signature(json.loads(f.read_text(encoding="utf-8")), rc)
        base = scan("reference", False)
        with subprocess.Popen([str(binary), "serve", str(root), "--service", name, "--cache-dir", str(tmp / "index")],
                              stdout=subprocess.DEVNULL) as child:
            def status():
                return json.loads(calls(binary, "status", "--service", name)[1])
            try:
                wait_ready(child, status)
                for tag in ("first", "repeat"):
                    if base != scan(tag, True):
                        raise ValueError("counts, reports or exit code changed; rejecting sample")
                    wait_ready(child, status)
                result["counts_and_alerts_match"] = True
                result["files"], result["bytes"] = base[0]["files"], base[0]["bytes"]
                result["corpus_matches_expected"] = (args.expect_files in (None, result["files"]) and
                                                     args.expect_bytes in (None, result["bytes"]))
                proc = psutil.Process(child.pid)
                for _ in range(args.windows):
                    before = wait_ready(child, status)
                    w = idle(proc, args.idle_seconds)  # zero IPC inside this interval
                    if status() != before:
                        w.update(window="activity_during_idle", idle_cpu_this_window="inconclusive", memory_this_workload="inconclusive")
                    result["idle_windows"].append(w)
                before = wait_ready(child, status)
                rtts = []
                for _ in range(30):
                    t = clock()
                    status()
                    rtts.append((clock() - t) / 1e6)
                result["status_cli_round_trip_ms"] = percentiles(rtts)  # diagnostic, never a response gate
                if d:
                    before = wait_ready(child, status)
                    groups, raw = events(binary, name, root, d, args.events_per_class, args.event_timeout)
                    after = status()
                    drain_by = time.monotonic() + args.event_timeout
                    while after["phase"] != "idle" and time.monotonic() < drain_by:
                        time.sleep(.01)
                        after = status()
                    delivered = after["events"] - before["events"]
                    result.update(events=groups, samples=raw, before_events=before, after_events=after,
                                  isolated_event_window=(after["watching"] and after["phase"] == "idle" and
                                                         0 < delivered <= after["consecutive_same_path_events"] and
                                                         before["full_scans"] == after["full_scans"] and
                                                         before["loss_incidents"] == after["loss_incidents"]))
                result["process_high_water_after_workload_bytes"] = high_water(child.pid, proc.memory_info())
                calls(binary, "stop", "--service", name)
                if child.wait(timeout=10) != 0:
                    raise ValueError("unclean service shutdown")
            finally:
                if child.poll() is None:
                    child.kill()  # only a child we created, never an existing instance
                    child.wait()
    result["gates"] = gates(result)
    with out.open("x", encoding="utf-8") as f:
        json.dump(result, f, indent=2, allow_nan=False)
        f.write("\n")
    print(json.dumps({"out": str(out), "files": result["files"], "events": result.get("events"), "gates": result["gates"]}, indent=2))
    return result


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--binary", required=True, type=Path)
    p.add_argument("--root", required=True, type=Path)
    p.add_argument("--out", required=True, type=Path)
    p.add_argument("--label", default="local")
    p.add_argument("--expect-files", type=int)
    p.add_argument("--expect-bytes", type=int)
    p.add_argument("--idle-seconds", type=int, default=600)
    p.add_argument("--windows", type=int, default=3)
    p.add_argument("--probe-changes", action="store_true", help="write only our disposable benign probe inside root")
    p.add_argument("--events-per-class", type=int, default=1000)
    p.add_argument("--event-timeout", type=float, default=2, help="deadline per logical mutation (seconds)")
    args = p.parse_args()
    if (min(args.idle_seconds, args.windows, args.events_per_class) < 1 or not 0 < args.event_timeout <= 60 or
            sys.platform not in ("linux", "win32") or sys.version_info < (3, 11)):
        p.error("need Python 3.11+, Linux/Windows, positive counts and a deadline <= 60 seconds")
    try:
        import psutil
    except ImportError:
        p.error("install psutil==7.2.2 in the observer's Python environment (not in the service)")
    try:
        measure(args, psutil)  # 0 = measurement delivered (including failed gates), not acceptance
    except (OSError, ValueError, KeyError, psutil.Error, subprocess.TimeoutExpired) as e:
        p.exit(1, f"measurement stopped: {e}\n")


if __name__ == "__main__":
    main()
