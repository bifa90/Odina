#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
"""Guard what the manual benchmark is allowed to call a result. No network."""
import contextlib
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("verify_b9", Path(__file__).with_name("verify-b9.py"))
b9 = importlib.util.module_from_spec(spec)
spec.loader.exec_module(b9)


def row(seconds=2, pct=2.99, status="pass"):
    return {"seconds": seconds, "cpu_percent_one_core": pct, "B9": status,
            "cpu_seconds_approx": seconds * pct / 100, "clean_sha_skip_lower_bound_percent": 95,
            "work": {"sha_files": 1, "streamed_bytes": 100, "cache_hits": 0}}


class Comparison(unittest.TestCase):
    def test_b9_is_a_strict_active_core_percent_not_cpu_seconds_or_speedup(self):
        cases = [([row()] * 3, False, "pass"), ([row(pct=3, status="fail")] * 3, False, "fail"),
                 ([row()] * 3, True, "inconclusive"), ([row(seconds=0.5)] * 3, False, "inconclusive"),
                 ([row(), row(), row(seconds=3)], False, "inconclusive"),
                 ([row(status="unmeasured")] * 3, False, "inconclusive")]
        for after, quick, expected in cases:
            with self.subTest(after=after, quick=quick), contextlib.redirect_stdout(io.StringIO()):
                result = b9.compare([row(10, 90, "fail")] * 3, after, quick)
                self.assertEqual(expected, result["B9_this_workload"])
                if expected == "fail":
                    self.assertEqual("faster", result["speed_observed"])
                    self.assertGreater(result["cpu_seconds_reduction_percent"], 90)

    def test_count_or_alert_mismatch_aborts_before_summarising(self):
        with tempfile.TemporaryDirectory() as tmp, contextlib.redirect_stdout(io.StringIO()), \
                patch.object(b9, "scan", side_effect=[("first", row()), ("changed", row())]), \
                patch.object(b9, "compare") as compare:
            with self.assertRaises(ValueError):
                b9.measure(1, Path(tmp), {"before": "old", "after": "new"}, Path(tmp), True)
            compare.assert_not_called()

    def test_parity_signature_ignores_timing_but_includes_the_report_and_exit(self):
        with tempfile.TemporaryDirectory() as tmp, patch.object(b9.subprocess, "run") as run:
            p = Path(tmp) / "report.json"
            run.return_value.returncode = 2
            doc = {"schema": "odina-perf/v1", "reports": [{"sha256": "a" * 64}],
                   "scan": {"files": 11, "bytes": 168, "clean": 10, "confirmed": 1, "elapsed_ms": 2000,
                            "truncated": False, "unreadable": 0, "read_errors": 0},
                   "metrics": [{"gate": "cpu_scan_avg_percent", "status": "fail", "value": 90},
                               {"gate": "throughput_files_per_sec", "value": 5.5}],
                   "work": {"cache_hits": 0, "bloom_negatives": 10, "sha_files": 1, "streamed_bytes": 168}}

            def scan():
                p.write_text(json.dumps(doc), encoding="utf-8")
                return b9.scan("binary", tmp, p, Path(tmp) / "index")

            sig, measured = scan()
            self.assertEqual(1.8, measured["cpu_seconds_approx"])
            self.assertEqual(90, measured["clean_sha_skip_lower_bound_percent"])
            doc["scan"]["elapsed_ms"] = 999
            self.assertEqual(sig, scan()[0])
            doc["reports"][0]["sha256"] = "b" * 64
            self.assertNotEqual(sig, scan()[0])
            doc["reports"][0]["sha256"] = "a" * 64
            run.return_value.returncode = 0
            self.assertNotEqual(sig, scan()[0])
            doc["work"]["sha_files"] += 1  # retried pass, not comparable
            with self.assertRaises(ValueError):
                scan()
            doc["scan"]["unreadable"] = 1
            doc["work"]["sha_files"] -= 1
            with self.assertRaises(ValueError):
                scan()


if __name__ == "__main__":
    unittest.main()
