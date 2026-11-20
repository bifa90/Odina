#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
"""Native overflow regression. Only runs against our own child and a new temporary tree.
No test-only control opcode, root parameter, kernel tuning or simulated overflow flag.
"""
import argparse
import json
from pathlib import Path
import secrets
import subprocess
import sys
import tempfile
import time

import psutil


def run(binary, root):
    name = "gap-" + secrets.token_hex(8)
    a, b, survivor = (root / n for n in ("first.dat", "renamed.dat", "stable.dat"))
    a.write_bytes(b"a")
    survivor.write_bytes(b"old")
    def query(op, *args):
        r = subprocess.run([binary, op, *args, "--service", name], capture_output=True, timeout=5)
        return json.loads(r.stdout) if r.returncode == 0 else {}
    def until(predicate, child):
        end = time.monotonic() + 30
        while child.poll() is None:
            s = query("status")
            if predicate(s):
                return s
            if time.monotonic() > end:
                raise ValueError(f"coverage condition timed out: {s}")
            time.sleep(.01)
        raise ValueError("owned service exited")
    with subprocess.Popen([binary, "serve", root, "--service", name], stdout=subprocess.DEVNULL) as child:
        try:
            before = until(lambda s: s.get("watching"), child)
            assert query("verdict", a.name)["bytes"] == 1
            count = 2500  # far larger than the Windows 64 KiB change buffer
            if sys.platform == "linux":
                # Each rename queues MOVED_FROM + MOVED_TO, not identical coalescible writes.
                limit = int(Path("/proc/sys/fs/inotify/max_queued_events").read_text())
                if limit > 250_000:
                    raise ValueError("unusually large inotify queue; refusing unbounded test")
                count = limit // 2 + 100
            proc = psutil.Process(child.pid)
            proc.suspend()
            try:
                for _ in range(count):
                    a.rename(b)
                    a, b = b, a
                a.unlink()  # loss of these last notifications must not leave the old inventory!
                survivor.write_bytes(b"reconciled")
            finally:
                proc.resume()
            after = until(lambda s: s.get("watching") and s["epoch"] > before["epoch"], child)
            assert after["full_scans"] > before["full_scans"] and after["loss_incidents"] > 0, after
            assert after["queue_overflows"] == 0, "expected kernel loss, not overflow of our 2 path keys"
            for old in (a, b):
                assert query("verdict", old.name)["state"] in ("unknown", "deleted")
            v = query("verdict", survivor.name)
            assert (v["state"], v["verdict"], v["bytes"]) == ("complete", "clean", 10), v
            query("stop")
            assert child.wait(timeout=10) == 0
            print(json.dumps({"native_gap_recovered": True, "renames_while_suspended": count,
                              "before": before, "after": after}))
        finally:
            if child.poll() is None:
                child.kill()
                child.wait()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    binary = parser.parse_args().binary.resolve(strict=True)
    with tempfile.TemporaryDirectory() as d:
        run(binary, Path(d))
