// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K27 AMSI / Script-Host Inspection Contract.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptVerdict {
    Allow,
    MonitorTimeoutFallback,
    Block,
}

pub fn evaluate_script_inspection(timed_out: bool, malicious: bool) -> ScriptVerdict {
    if malicious {
        ScriptVerdict::Block
    } else if timed_out {
        ScriptVerdict::MonitorTimeoutFallback
    } else {
        ScriptVerdict::Allow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeout_never_silently_allows_without_monitor_fallback() {
        assert_eq!(evaluate_script_inspection(true, false), ScriptVerdict::MonitorTimeoutFallback);
        assert_eq!(evaluate_script_inspection(false, true), ScriptVerdict::Block);
        assert_eq!(evaluate_script_inspection(false, false), ScriptVerdict::Allow);
    }
}
