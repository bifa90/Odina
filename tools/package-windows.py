#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
"""Build the actual Windows downloads, not the website or an installer mock."""

import argparse
import hashlib
from pathlib import Path
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo

ROOT = Path(__file__).resolve().parents[1]
CATALOG = Path('components/repair/odina-repair-v1.tsv')
PIN = Path('crates/odina-core/src/repair-v1.sha256')
STANDALONE = 'odina-repair-v1-r1.zip'
LIMIT = 100_000_000  # decimal MB of the complete downloadable ZIP, inclusive
NOTE = f'''Repair data (schema v1, revision 1; training families only)
If {CATALOG.as_posix()} is missing, extract {STANDALONE} from the same
Odina release beside this executable, preserving directories. Run the executable
with repair-db to check the data against its built-in pin. No network required.
A missing or untrusted database is never permission to run an infected file.
'''


def sidecar(path):
    digest = hashlib.sha256()
    with path.open('rb') as src:
        for chunk in iter(lambda: src.read(1 << 20), b''):
            digest.update(chunk)
    Path(str(path) + '.sha256').write_text(f'{digest.hexdigest()}  {path.name}\n', encoding='ascii')


def package(stage, exe, output, *, limit=LIMIT, root=ROOT):
    entries = [exe] if exe == 'odina.exe' else [exe, 'READ-ME-FIRST.txt']
    if exe not in ('odina.exe', 'odina-check.exe') or not all((stage / name).is_file() for name in entries):
        raise ValueError('missing or invalid staged build')
    data = (root / CATALOG).read_bytes()
    pin = (root / PIN).read_text(encoding='ascii').strip()
    if hashlib.sha256(data).hexdigest() != pin:
        raise ValueError('repair catalog does not match the program pin')
    notices = {name: (root / name).read_bytes() for name in ('LICENSE', 'NOTICE')}
    separate = output.with_name(STANDALONE)
    with ZipFile(separate, 'w', ZIP_DEFLATED) as z:
        # Identical revision, identical standalone ZIP in either matrix job.
        files = {CATALOG.name: data, **notices}
        for name, content in files.items():
            entry = ZipInfo((CATALOG.parent / name).as_posix())
            entry.create_system = 3
            entry.external_attr = 0o100644 << 16
            z.writestr(entry, content, compress_type=ZIP_DEFLATED)
    sidecar(separate)

    def archive(bundle):
        with ZipFile(output, 'w', ZIP_DEFLATED) as z:
            for name in entries:
                z.write(stage / name, name)
            for name, content in notices.items():
                z.writestr(name, content)
            z.writestr('README-REPAIR.txt', NOTE)
            if bundle:
                z.writestr(CATALOG.as_posix(), data)

    archive(True)
    bundled = output.stat().st_size <= limit
    if not bundled:
        archive(False)
    sidecar(output)
    return bundled


if __name__ == '__main__':
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('stage', type=Path)
    ap.add_argument('exe', choices=('odina.exe', 'odina-check.exe'))
    ap.add_argument('output', type=Path)
    args = ap.parse_args()
    print('Repair data:', 'bundled' if package(args.stage, args.exe, args.output) else STANDALONE)
