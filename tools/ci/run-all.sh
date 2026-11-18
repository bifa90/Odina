#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
#
# Run every gate locally, in the same order and with the same exit codes as
# CI. Use this before pushing: if it passes here, the workflow will pass.
#
#   ./tools/ci/run-all.sh            # all gates
#   ./tools/ci/run-all.sh --fast     # skip the font hash pass (682 files)

set -uo pipefail

cd "$(dirname "$0")/../.." || exit 2

FAST=0
[[ "${1:-}" == "--fast" ]] && FAST=1

pass=0
fail=0
failed_names=()

run() {
  local name="$1"; shift
  printf '%-10s ' "$name"
  if "$@" >/tmp/gate-out 2>&1; then
    printf 'PASS\n'
    pass=$((pass + 1))
  else
    printf 'FAIL\n'
    sed 's/^/           /' /tmp/gate-out | head -20
    fail=$((fail + 1))
    failed_names+=("$name")
  fi
}

run license  python3 tools/ci/check-license.py --quiet
run icons    python3 tools/ci/check-no-emoji.py --quiet
run contrast python3 tools/ci/check-contrast.py --quiet
run i18n     python3 tools/i18n-lint.py --quiet
run pseudo   python3 tools/pseudo-localize.py --quiet --check
if [[ $FAST -eq 0 ]]; then
  run fonts  python3 tools/ci/check-fonts.py --quiet
fi
run docs     python3 tools/ci/check-docs.py --quiet
run exc-ui   python3 tools/ci/check-exception-ui.py --quiet
run ui-law   python3 tools/ci/check-ui-constitution.py --quiet
run loc-js   python3 tools/gen-locale-js.py --check --quiet
# Validate the allowlisted output, not just the source pages. The browser
# half of this gate is run separately in CI (see docs/ci.md).
run site     python3 tools/deploy-site.py --dry-run
run repair-pkg python3 tools/ci/test-repair-package.py
run rust-syn bash -c "python3 tools/check-rust-syntax.py > /dev/null"
run public   python3 tools/ci/check-public-build.py --quiet

echo
if [[ $fail -eq 0 ]]; then
  echo "all gates passed ($pass)"
  exit 0
fi
echo "gates passed: $pass · failed: $fail (${failed_names[*]})"
exit 1
