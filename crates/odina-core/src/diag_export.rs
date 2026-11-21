// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K29 Diagnostic Bundle Export Contract.

pub fn can_export_diagnostics(operator_consented: bool, pii_redacted: bool) -> bool {
    operator_consented && pii_redacted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_consent_and_redaction() {
        assert!(can_export_diagnostics(true, true));
        assert!(!can_export_diagnostics(false, true));
        assert!(!can_export_diagnostics(true, false));
    }
}
