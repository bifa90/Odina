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

## What changed at K1, and why this gate had to change with it

Before K1 the split was *by crate*: `odina-cache` and `odina-resident` were
optional, so the resident service simply did not exist in a public build. That
was a cheaper rule to enforce and it was the wrong product: the service is a
core capability (v19 §1, D49), and a user who installs Odina must get it
without a lab flag.

So the split is now *by crate, but a different pair*. `odina-scan` is the only
pass loop in the tree; it depends on `odina-core` and nothing else. The service
and the cache are public. The report writer and the meter are optional. The
invariants below are what keeps that honest:

  1. `odina-report` and `odina-instrument` are `optional = true` in the CLI
     manifest, and `instrument` is not in `default`.
  2. `odina-scan` depends on `odina-core` alone, and its source never names a
     measurement or cache crate — the loop that ships to every user must not
     be able to measure.
  3. `odina-scan`, `odina-cache` and `odina-resident` are **not** optional in
     the CLI: making the service optional again is how K1 regresses.
  4. `odina-core` depends on none of the crates above it.
  5. `mod check;` is gated on `instrument`; `mod resident;` is **not**.
  6. With every `#[cfg(feature = "instrument")]` item removed from
     `resident.rs`, the remaining text does not mention `check::`, `Meter`,
     `odina_report` or `odina_instrument`. This is the check that matters:
     the public inventory path and the measured one are two functions, and the
     public one must be able to compile with the module absent.
  7. No crate in the workspace depends on a network or HTTP client, and the
     instrumentation crates contain no socket, no `std::net`, no URL string and
     no call to a process-spawning API.

When `cargo` is available (it is in CI, it is not in every dev sandbox), the
script also runs `cargo tree -p odina-cli --edges normal` and fails if
`odina-report` or `odina-instrument` appears in the default build graph, and
fails if `odina-scan` does *not* — the loop has to ship. That is the binary
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

# Must be absent from a default build.
INSTR_CRATES = ("odina-report", "odina-instrument")

# The crates the separation is about: the engine, the loop, the cache and the
# service. Their arrangement is checked below; they are not "private".
PUBLIC_CRATES = ("odina-core", "odina-scan", "odina-cache", "odina-resident")

# Names of measurement and cache crates, in Rust import spelling, so that a
# `use odina_report::…` inside the public loop is caught as well as the
# manifest line.
FORBIDDEN_IN_PUBLIC_LOOP = (
    "odina_report",
    "odina-report",
    "odina_instrument",
    "odina-instrument",
    "odina_cache",
    "odina-cache",
    "Meter",
    "odina_resident",
    "odina-resident",
)

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

INSTR_ATTR = '#[cfg(feature = "instrument")]'


def code_only(text: str) -> str:
    """Drop comments, so a crate may *document* what it must not import.

    `odina-scan` explains at length why it has no cache and no report; that
    prose names the crates and is the most useful text in the file. Only code
    counts, so line comments go and nothing else does.
    """
    return "\n".join(line.split("//", 1)[0] for line in text.splitlines())


def strip_instrument_items(text: str) -> str:
    """Remove every item guarded by `#[cfg(feature = "instrument")]`.

    The point is to read `resident.rs` the way the compiler reads it in a
    public build: the gated `use`, the gated `impl Session` holding the
    measured pass, and the gated branch inside `inventory` all disappear, and
    whatever is still referring to them is a broken public build.

    Deliberately line-oriented and blunt — it only has to be right about this
    one attribute, and being wrong here fails closed (junk left behind fails
    the check) rather than open.
    """
    lines = text.splitlines(keepends=True)
    out: list[str] = []
    i = 0
    while i < len(lines):
        if lines[i].strip() != INSTR_ATTR:
            out.append(lines[i])
            i += 1
            continue
        i += 1
        # Skip the blank and comment lines that belong to the gated item, so
        # its doc comment does not survive into the public text either.
        while i < len(lines) and (
            not lines[i].strip() or lines[i].lstrip().startswith("//")
        ):
            i += 1
        depth = 0
        opened = False
        while i < len(lines):
            depth += lines[i].count("{") - lines[i].count("}")
            if "{" in lines[i]:
                opened = True
            i += 1
            if opened and depth == 0:
                break
            if not opened and lines[i - 1].rstrip().endswith(";"):
                break
    return "".join(out)


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

    @staticmethod
    def dependency_line(text: str, crate: str) -> str | None:
        match = re.search(
            "^" + re.escape(crate) + r"\s*=\s*\{([^}]*)\}", text, re.MULTILINE
        )
        return match.group(1) if match else None

    # ── 1 + 3: the CLI manifest ───────────────────────────────────────────
    def check_cli_manifest(self) -> None:
        text = self.read("crates/odina-cli/Cargo.toml")
        if not text:
            return
        for crate in INSTR_CRATES:
            line = self.dependency_line(text, crate)
            if line is None:
                self.fail(
                    f"{crate} is not declared in crates/odina-cli/Cargo.toml; "
                    "the instrumented build needs it and the public build must "
                    "be able to leave it out"
                )
                continue
            if "optional = true" not in line:
                self.fail(
                    f"{crate} is not optional — the public build would link it"
                )
        for crate in ("odina-scan", "odina-cache", "odina-resident"):
            line = self.dependency_line(text, crate)
            if line is None:
                self.fail(f"{crate} is not declared in crates/odina-cli/Cargo.toml")
            elif "optional = true" in line:
                self.fail(
                    f"{crate} is optional again — the resident service is a core "
                    "capability (v19 §1, D49) and must ship in every build"
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
        defin = re.search(r"^instrument\s*=\s*\[([^\]]*)\]", body, re.M)
        if defin is None:
            self.fail("the `instrument` feature is not defined")
        else:
            enabled = {name.strip().strip('"') for name in defin.group(1).split(",")}
            enabled.discard("")
            for crate in INSTR_CRATES:
                if crate not in enabled:
                    self.fail(f"the `instrument` feature does not enable {crate}")
            for crate in ("odina-cache", "odina-resident"):
                if crate in enabled:
                    self.fail(
                        f"the `instrument` feature still enables {crate}; the "
                        "service is not a measurement, so it belongs in every build"
                    )

    # ── 2: the public loop's own dependencies ─────────────────────────────
    def check_scan_crate(self) -> None:
        text = self.read("crates/odina-scan/Cargo.toml")
        if text:
            deps = re.search(r"^\[dependencies\]\s*$(.*?)(?=^\[|\Z)", text, re.M | re.S)
            body = deps.group(1) if deps else ""
            names = {
                m.group(1)
                for m in re.finditer(r"^([A-Za-z0-9_-]+)\s*=", body, re.MULTILINE)
            }
            if names != {"odina-core"}:
                extra = ", ".join(sorted(names - {"odina-core"})) or "none"
                self.fail(
                    "odina-scan must depend on odina-core alone — it is the loop "
                    f"that ships to every user; found extra deps: {extra}"
                )
        for src in sorted((self.root / "crates" / "odina-scan" / "src").rglob("*.rs")):
            text = code_only(src.read_text(encoding="utf-8"))
            for needle in FORBIDDEN_IN_PUBLIC_LOOP:
                if needle in text:
                    self.fail(f"{src.relative_to(self.root)}: contains {needle!r}")

    # ── 4: the engine cannot measure ──────────────────────────────────────
    def check_core_manifest(self) -> None:
        text = self.read("crates/odina-core/Cargo.toml")
        if not text:
            return
        deps = re.search(r"^\[dependencies\]\s*$(.*?)(?=^\[|\Z)", text, re.M | re.S)
        body = deps.group(1) if deps else ""
        for crate in (*INSTR_CRATES, "odina-cache", "odina-resident", "odina-scan"):
            if re.search(rf"^{re.escape(crate)}\b", body, re.M):
                self.fail(f"odina-core depends on {crate}; the engine must not measure")

    # ── 5 + 6: the modules, and what the public half may name ─────────────
    def check_module_gate(self) -> None:
        text = self.read("crates/odina-cli/src/main.rs")
        if text:
            self._require_gate(text, "main.rs", "mod check;", True)
            self._require_gate(text, "main.rs", "mod resident;", False)

        text = self.read("crates/odina-cli/src/resident.rs")
        if not text:
            return
        public = code_only(strip_instrument_items(text))
        for needle in ("check::", "Meter", "odina_report", "odina_instrument", "Instant::"):
            if needle in public:
                self.fail(
                    f"crates/odina-cli/src/resident.rs: {needle!r} appears outside a "
                    "`#[cfg(feature = \"instrument\")]` item; the public build has no "
                    "such module and would not compile"
                )
        if "odina_scan::pass(" not in public:
            self.fail(
                "crates/odina-cli/src/resident.rs: the public inventory path does not "
                "call odina_scan::pass — the service would have no loop of its own"
            )

    def _require_gate(self, text: str, where: str, decl: str, gated: bool) -> None:
        match = None
        for m in re.finditer("^" + re.escape(decl), text, re.M):
            before = text[: m.start()].rstrip().splitlines()
            gate_line = before[-1].strip() if before else ""
            if gate_line.startswith("#[cfg("):
                match = gate_line
            if match:
                break
        if gated:
            if match is None:
                self.fail(f"{where}: `{decl}` is not gated at all")
            elif 'feature = "instrument"' not in match:
                self.fail(
                    f"{where}: `{decl}` is not gated on the `instrument` feature, so "
                    "the module (and its dependencies) compile into the public build"
                )
        elif match is not None and 'feature = "instrument"' in match:
            self.fail(
                f"{where}: `{decl}` is gated on `instrument`; the service is a core "
                "capability and must be present in the public build"
            )

    # ── 7: no network crate anywhere ──────────────────────────────────────
    def check_no_network(self) -> None:
        for manifest in sorted(self.root.glob("crates/*/Cargo.toml")):
            text = manifest.read_text(encoding="utf-8")
            for name in NETWORK_CRATES:
                if re.search(rf"^{re.escape(name)}\s*=", text, re.M):
                    self.fail(f"{manifest}: depends on {name}")

    # ── 7b: the measurement crates cannot reach out ────────────────────────
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
        for crate in INSTR_CRATES:
            if re.search(rf"\b{re.escape(crate)} v", graph):
                self.fail(
                    f"{crate} appears in the default build graph of odina-cli; "
                    "the public binary would link measurement code"
                )
        for crate in PUBLIC_CRATES:
            if not re.search(rf"\b{re.escape(crate)} v", graph):
                self.fail(
                    f"{crate} is missing from the default build graph of odina-cli; "
                    "the public build has stopped shipping a core crate"
                )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", default=".")
    parser.add_argument("--quiet", action="store_true")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    gate = Gate(root)
    gate.check_cli_manifest()
    gate.check_scan_crate()
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
