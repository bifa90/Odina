# SPDX-License-Identifier: MIT
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
import importlib.util
from pathlib import Path
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("resident_lab", Path(__file__).with_name("verify-resident.py"))
lab = importlib.util.module_from_spec(spec)
spec.loader.exec_module(lab)


class Gates(unittest.TestCase):
    def test_thresholds_include_exactly_ten_ms_and_timeouts_not_a_zero_latency(self):
        r = {"t0_ns": 5, "received_ns": 6, "invalidated_ns": 7, "published_ns": 10_000_005}
        s = lab.event_summary([r, {"published_ns": None}])
        self.assertEqual((s['at_or_above_10_ms'], s['unwitnessed_percent']), (1, 50))
        self.assertEqual(s['mutation_to_publication_ms']['max'], 10)
        self.assertIsNone(lab.event_summary([{"published_ns": None}])['mutation_to_publication_ms']['p50'])

    def test_no_stale_generation_or_inventory_witness(self):
        r = dict(origin='event', state='complete', bytes=65, verdict='clean', generation=5, received_ns=101,
                 invalidated_ns=102, published_ns=103)
        with patch.object(lab, 'clock', return_value=200):
            for invalid in ({'origin': 'inventory'}, {'received_ns': 90}, {'generation': 4}, {'bytes': 64}, {'state': 'unknown'}):
                self.assertIsNone(lab.witnessed(lambda: {**r, **invalid}, 100, 'complete', 65, 4, 100))
            self.assertEqual(lab.witnessed(lambda: r, 100, 'complete', 65, 4, 100), r)

    def test_short_idle_incomplete_classes_or_reinventory_cannot_pass(self):
        w = dict(idle_cpu_this_window='pass', memory_this_workload='pass')
        g = dict(attempted=1000, at_or_above_10_ms=0, missed_by_deadline=0)
        r = dict(idle_windows=[w] * 3, events=dict(create=g, modify=g, delete=g), isolated_event_window=True,
                 process_high_water_after_workload_bytes=49_999_999, corpus_matches_expected=True)
        self.assertEqual(lab.gates(r)['workload'], 'pass')
        self.assertEqual(lab.gates({**r, 'idle_windows': [w]})['workload'], 'inconclusive')
        self.assertEqual(lab.gates({**r, 'isolated_event_window': False})['strict_response'], 'fail')
        self.assertEqual(lab.gates({**r, 'process_high_water_after_workload_bytes': 50_000_000})['workload'], 'fail')
        g['attempted'] = 999
        self.assertEqual(lab.gates(r)['strict_response'], 'inconclusive')
