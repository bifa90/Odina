#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
"""Turn a failed clippy run into annotations.

The clippy step writes `clippy.log`. Job logs are not always retrievable (they
live behind a signed URL that can be refused), so this reads the file the step
created and emits the compiler's own diagnostics as workflow errors — one per
`error`, with the line it points at. A single cycle is then enough to see what
broke instead of a run that only says "exit code 1".
"""

import pathlib
import re
import sys

ANSI = re.compile(r"\x1b\[[0-9;]*m")
LOG = pathlib.Path("clippy.log")


def main() -> int:
    if not LOG.exists():
        print("::error::clippy.log is missing — the clippy step did not write it")
        return 0

    lines = [ANSI.sub("", line).rstrip() for line in LOG.read_text(errors="replace").splitlines()]
    exits = [line for line in lines if re.match(r"^(WORKSPACE|INSTRUMENT)_EXIT=", line)]
    print("::error::" + " ".join(exits or ["no exit codes recorded"])[:400])

    # One diagnostic, in order: the `error:` line and the `-->` that follows it.
    # Pairing the two lists by index instead reads as a mismatch and sends the
    # reader to the wrong line.
    reported = 0
    for index, line in enumerate(lines):
        if not line.startswith("error") or reported >= 8:
            continue
        location = next(
            (later.strip() for later in lines[index + 1 : index + 6] if later.strip().startswith("-->")),
            "",
        )
        print(f"::error::{line.strip()[:300]} {location[:200]}".rstrip())
        reported += 1

    if not reported:
        print("::error::no 'error' lines — last: " + (lines[-1] if lines else "empty")[:400])
    return 0


if __name__ == "__main__":
    sys.exit(main())
