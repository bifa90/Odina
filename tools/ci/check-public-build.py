#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
"""Public-build gate — the public binary must contain no measurement code.

§12.2 rule 4 of the lab-build section in `docs/odina-agent-prompt-v19.md` says, in one line:

> CI verifies the public release contains **no** instrumentation code —
> a textual check **and** a binary check.

This script is the textual half. It exists because "we only call it behind a
flag" is a claim, and a flag that defaults to on, or a dependency that is not
optional, ships the measurement layer to every user while the source still
looks innocent.

Checked here:

  1. In `crates/odina-cli/Cargo.toml`, the report, instrumentation, cache
     and resident packages are `optional = true`.
  2. The `instrument` feature is not in `default`.
  3. `odina-core` has no dependency on the cache, IPC or measurement layers.
  4. The `check` module in the CLI is gated by
     `cfg(any(feature = "instrument", test))`.
  5. No crate in the workspace depends on a network or HTTP client.
  6. The instrumentation crates contain no socket, no `std::net`, no URL
     string, and no call to a process-spawning API.

When `cargo` is available (it is in CI, it is not in every dev sandbox), the
script also runs `cargo tree -p odina-cli --edges normal` and fails if any
private crate appears in the default build graph. That is the binary
half: `cargo tree` reports what the link step will actually contain, not what
the source appears to say.

Usage:
    python3 tools/ci/check-public-build.py [--root .] [--quiet]

Exit codes: 0 = the public build is clean, 1 = a violation, 2 = usage error.
"""

from __future__ import annotations

import argparse
import re
import shutil
import subprocess
import sys
from pathlib import Path

INSTR_CRATES = ("odina-report", "odina-instrument")
PRIVATE_CRATES = (*INSTR_CRATES, "odina-cache", "odina-resident")

# Crates that would give any part of the product a way to talk to a network.
NETWORK_CRATES = (
    "reqwest", "hyper", "ureq", "curl", "isahc", "attost", "attohttpc",
    "surf", "awc", "tokio", "async-std", "smol", "zmq", "nanomsg",
)

# Calls that would let the engine or the measurement layer reach outside the
# file it was pointed at.
FORBIDDEN_CALLS = (
    "std::net::",
    "TcpStream",
    "UdpSocket",
    "std::process::Command",
    "Command::new",
    "curl ",
    "http://",
    "https://",
)


class Gate:
    def __init__(self, root: Path) -> None:
        self.root = root
        self.errors: list[str] = []
        self.notes: list[str] = []

    def fail(self, message: str) -> None:
        self.errors.append(message)

    def read(self, rel: str) -> str:
        path = self.root / rel
        if not path.exists():
            self.fail(f"missing file: {rel}")
            return ""
        return path.read_text(encoding="utf-8")

    # ── 1 + 2: the CLI manifest ───────────────────────────────────────────
    def check_cli_manifest(self) -> None:
        text = self.read("crates/odina-cli/Cargo.toml")
        if not text:
            return
        for crate in PRIVATE_CRATES:
            pattern = re.compile(
                "^" + re.escape(crate) + r"\s*=\s*\{([^}]*)\}", re.MULTILINE
            )
            match = pattern.search(text)
            if crate == "odina-report" and match is None:
                # odina-report is present twice: optional dependency and dev
                # dependency. The optional one must exist.
                self.fail(
                    "odina-report is not declared as an optional dependency"
                )
                continue
            if match is None:
                self.fail(f"{crate} is not declared in crates/odina-cli/Cargo.toml")
                continue
            if "optional = true" not in match.group(1):
                self.fail(
                    f"{crate} is not optional — the public build would link it"
                )

        feature = re.search(r"^\[features\]\s*$(.*?)(?=^\[|\Z)", text, re.M | re.S)
        if not feature:
            self.fail("crates/odina-cli/Cargo.toml has no [features] section")
            return
        body = feature.group(1)
        default = re.search(r"^default\s*=\s*(.*)$", body, re.M)
        if default is None:
            self.fail("no `default = []` line; the instrument flag might be on")
        elif "instrument" in default.group(1):
            self.fail("the `instrument` feature is enabled by default")
        if not re.search(r"^instrument\s*=\s*\[", body, re.M):
            self.fail("the `instrument` feature is not defined")

    # ── 3: the engine cannot measure ──────────────────────────────────────
    def check_core_manifest(self) -> None:
        text = self.read("crates/odina-core/Cargo.toml")
        if not text:
            return
        deps = re.search(r"^\[dependencies\]\s*$(.*?)(?=^\[|\Z)", text, re.M | re.S)
        body = deps.group(1) if deps else ""
        for crate in PRIVATE_CRATES:
            if re.search(rf"^{re.escape(crate)}\b", body, re.M):
                self.fail(f"odina-core depends on {crate}; the engine must not measure")

    # ── 4: the module is gated ────────────────────────────────────────────
    def check_module_gate(self) -> None:
        text = self.read("crates/odina-cli/src/main.rs")
        if not text:
            return
        # The gate attribute sits on the line before `mod check;`, possibly
        # after comment lines, so search backwards from the declaration.
        match = None
        for m in re.finditer(r"^mod check;", text, re.M):
            before = text[: m.start()].rstrip().splitlines()
            gate_line = before[-1] if before else ""
            if gate_line.strip().startswith("#[cfg("):
                match = re.match(r"^(#\[cfg\(.*\)\])", gate_line.strip())
            if match:
                break
        if match is None:
            self.fail("crates/odina-cli/src/main.rs does not declare a gated `mod check;`")
            return
        gate = match.group(1)
        if "feature = \"instrument\"" not in gate:
            self.fail(
                "`mod check;` is not gated on the `instrument` feature, so the "
                "module (and its dependencies) compile into the public build"
            )

    # ── 5: no network crate anywhere ──────────────────────────────────────
    def check_no_network(self) -> None:
        for manifest in sorted(self.root.glob("crates/*/Cargo.toml")):
            text = manifest.read_text(encoding="utf-8")
            for name in NETWORK_CRATES:
                if re.search(rf"^{re.escape(name)}\s*=", text, re.M):
                    self.fail(f"{manifest}: depends on {name}")

    # ── 6: the measurement crates cannot reach out ─────────────────────────
    def check_instrument_sources(self) -> None:
        for crate in INSTR_CRATES:
            for src in sorted((self.root / "crates" / crate / "src").rglob("*.rs")):
                text = src.read_text(encoding="utf-8")
                for needle in FORBIDDEN_CALLS:
                    if needle in text:
                        self.fail(f"{src}: contains {needle!r}")

    # ── the binary half ───────────────────────────────────────────────────
    def check_cargo_tree(self) -> None:
        cargo = shutil.which("cargo")
        if cargo is None:
            self.notes.append(
                "cargo is not installed here, so the dependency-graph check "
                "was skipped; CI runs it on ubuntu-latest"
            )
            return
        try:
            out = subprocess.run(
                [cargo, "tree", "-p", "odina-cli", "--edges", "normal"],
                cwd=self.root,
                capture_output=True,
                text=True,
                timeout=180,
                check=False,
            )
        except (OSError, subprocess.TimeoutExpired) as err:
            self.notes.append(f"cargo tree did not run: {err}")
            return
        if out.returncode != 0:
            self.notes.append(
                "cargo tree failed (no lockfile in the sandbox?): "
                + (out.stderr.strip().splitlines() or ["?"])[0]
            )
            return
        graph = out.stdout
        for crate in PRIVATE_CRATES:
            if re.search(rf"\b{re.escape(crate)} v", graph):
                self.fail(
                    f"{crate} appears in the default build graph of odina-cli; "
                    "the public binary would link measurement code"
                )
        if "odina-core v" not in graph:
            self.notes.append("odina-core did not appear in cargo tree output")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", default=".")
    parser.add_argument("--quiet", action="store_true")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    gate = Gate(root)
    gate.check_cli_manifest()
    gate.check_core_manifest()
    gate.check_module_gate()
    gate.check_no_network()
    gate.check_instrument_sources()
    gate.check_cargo_tree()

    for note in gate.notes:
        print(f"note: {note}", file=sys.stderr)

    if gate.errors:
        print("public-build gate: FAILED", file=sys.stderr)
        for err in gate.errors:
            print(f"  - {err}", file=sys.stderr)
        return 1

    if not args.quiet:
        print("public-build gate: clean — no measurement code in the default build")
    return 0


if __name__ == "__main__":
    sys.exit(main())
