#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
"""Exercise both sides of the release boundary with the real ZIP writer."""

import hashlib
import runpy
import tempfile
import unittest
from pathlib import Path
from zipfile import ZipFile

pack = runpy.run_path(str(Path(__file__).resolve().parents[1] / 'package-windows.py'))


class PackageTest(unittest.TestCase):
    def test_public_bundle_uses_only_the_public_binary_and_the_data(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            stage = root / 'stage'
            stage.mkdir()
            for name in ['odina.exe', 'odina-check.exe', 'READ-ME-FIRST.txt']:
                (stage / name).write_bytes(b'fixture')
            zipfile = root / 'public.zip'
            self.assertTrue(pack['package'](stage, 'odina.exe', zipfile))
            with ZipFile(zipfile) as z:
                self.assertEqual(set(z.namelist()), {'odina.exe', pack['CATALOG'].as_posix(), 'README-REPAIR.txt', 'LICENSE', 'NOTICE'})

    def test_layout_and_exact_threshold(self):
        self.assertEqual(pack['LIMIT'], 100_000_000)
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            stage = root / 'stage'
            stage.mkdir()
            exe = 'odina-check.exe'
            (stage / exe).write_bytes(b'harmless packaging fixture')
            (stage / 'READ-ME-FIRST.txt').write_text('CHECK BUILD')
            zipfile = root / 'check.zip'
            key = pack['CATALOG'].as_posix()
            data = (pack['ROOT'] / key).read_bytes()
            self.assertTrue(pack['package'](stage, exe, zipfile))
            full_size = zipfile.stat().st_size
            for limit, expected in [(full_size, True), (full_size - 1, False)]:
                self.assertEqual(pack['package'](stage, exe, zipfile, limit=limit), expected)
                with ZipFile(zipfile) as z:
                    self.assertEqual(key in z.namelist(), expected)
                    self.assertIn(pack['STANDALONE'], z.read('README-REPAIR.txt').decode())
                    self.assertIn('READ-ME-FIRST.txt', z.namelist())
                    self.assertIn(exe, z.namelist())
                    if expected:
                        self.assertEqual(z.read(key), data)
                separate = root / pack['STANDALONE']
                with ZipFile(separate) as z:
                    self.assertEqual(set(z.namelist()), {key, 'components/repair/LICENSE', 'components/repair/NOTICE'})
                    self.assertEqual(z.read(key), data)
                for path in [zipfile, separate]:
                    digest = hashlib.sha256(path.read_bytes()).hexdigest()
                    self.assertEqual(Path(str(path) + '.sha256').read_text().split(), [digest, path.name])
            # Neither a missing binary nor damaged data may produce a new build.
            with self.assertRaises(ValueError):
                pack['package'](stage, 'odina.exe', zipfile)
            (root / key).parent.mkdir(parents=True)
            (root / key).write_bytes(data + b'!')
            (root / pack['PIN']).parent.mkdir(parents=True)
            (root / pack['PIN']).write_text(hashlib.sha256(data).hexdigest())
            with self.assertRaisesRegex(ValueError, 'pin'):
                pack['package'](stage, exe, zipfile, root=root)
            # The shipped seed is checked too: cap, header, and CC0 rows only.
            (root / key).write_bytes(data)
            (root / pack['PIN']).write_text(hashlib.sha256(data).hexdigest())
            for name in ('LICENSE', 'NOTICE'):
                (root / name).write_bytes((pack['ROOT'] / name).read_bytes())
            seed = root / pack['SEED']
            seed.parent.mkdir(parents=True, exist_ok=True)
            seed.write_text('odina-seed/v1\t1\n' + 'aa' * 32 + '\t68\tfam\tcc0-1.0\tsrc\n', encoding='utf-8')
            self.assertTrue(pack['package'](stage, exe, zipfile, root=root))
            self.assertEqual(pack['SEED_CAP'], 2 * 1024 * 1024)
            for bad, message in [
                ('odina-seed/v2\t1\n', 'header'),
                ('odina-seed/v1\t1\n' + 'aa' * 32 + '\t68\tfam\tgpl-2.0\tsrc\n', 'CC0'),
                ('odina-seed/v1\t1\n' + 'aa' * 32 + '\t68\tfam\n', 'five columns'),
                ('# ' + 'x' * pack['SEED_CAP'] + '\n', 'cap'),
            ]:
                seed.write_text(bad, encoding='utf-8')
                with self.assertRaisesRegex(ValueError, message):
                    pack['package'](stage, exe, zipfile, root=root)


if __name__ == '__main__':
    unittest.main()
