#!/usr/bin/env python3
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
    errors = [line for line in lines if line.startswith("error")]
    where = [line.strip().lstrip("--> ") for line in lines if line.strip().startswith("-->")]

    print("::error::" + " ".join(exits or ["no exit codes recorded"])[:400])
    for index, error in enumerate(errors[1:9], start=0):
        location = where[index] if index < len(where) else ""
        print(f"::error::{error.strip()[:300]} {location[:200]}".rstrip())

    if not errors:
        print("::error::no 'error' lines — last: " + (lines[-1] if lines else "empty")[:400])
    return 0


if __name__ == "__main__":
    sys.exit(main())
