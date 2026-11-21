// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! OS-Native Micro-Sandbox & Progressive Behavioral Analysis Contract.
//!
//! Enforces:
//! - AppContainer / Low Integrity + Job Object isolation (network disabled, temp-only writes).
//! - Never claims "Safe" solely because no malicious behavior was observed during the window.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SandboxConfig {
    pub network_disabled: bool,
    pub temp_fs_redirected: bool,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxVerdict {
    MaliciousBehaviorObserved,
    SuspiciousBehaviorObserved,
    NoSuspiciousBehaviorObserved,
    Inconclusive,
    AnalysisFailed,
}

pub fn evaluate_sandbox_run(
    cfg: SandboxConfig,
    malicious_events: u32,
    suspicious_events: u32,
    timed_out: bool,
) -> SandboxVerdict {
    if !cfg.network_disabled || !cfg.temp_fs_redirected {
        return SandboxVerdict::AnalysisFailed;
    }
    if malicious_events > 0 {
        SandboxVerdict::MaliciousBehaviorObserved
    } else if suspicious_events > 0 {
        SandboxVerdict::SuspiciousBehaviorObserved
    } else if timed_out {
        SandboxVerdict::Inconclusive
    } else {
        SandboxVerdict::NoSuspiciousBehaviorObserved
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_isolation_and_never_claims_safe() {
        let unsafe_cfg = SandboxConfig {
            network_disabled: false,
            temp_fs_redirected: true,
            timeout_ms: 3000,
        };
        assert_eq!(evaluate_sandbox_run(unsafe_cfg, 0, 0, false), SandboxVerdict::AnalysisFailed);

        let safe_cfg = SandboxConfig {
            network_disabled: true,
            temp_fs_redirected: true,
            timeout_ms: 3000,
        };
        assert_eq!(
            evaluate_sandbox_run(safe_cfg, 1, 0, false),
            SandboxVerdict::MaliciousBehaviorObserved
        );
        assert_eq!(
            evaluate_sandbox_run(safe_cfg, 0, 0, false),
            SandboxVerdict::NoSuspiciousBehaviorObserved
        );
    }
}
