// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K9 Release Gate & Acceptance Summary Contract.
//!
//! Enforces that a release candidate is only marked Ready when all
//! required milestone contracts (K1..K8) and CI/public gates pass.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReleaseReadiness {
    pub ci_gates_passed: bool,
    pub instrument_tests_passed: bool,
    pub public_build_isolated: bool,
    pub k1_to_k8_verified: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReleaseVerdict {
    Ready,
    Blocked(Vec<&'static str>),
}

impl ReleaseReadiness {
    pub fn evaluate(&self) -> ReleaseVerdict {
        let mut blockers = Vec::new();
        if !self.ci_gates_passed {
            blockers.push("ci_gates_failed");
        }
        if !self.instrument_tests_passed {
            blockers.push("instrument_tests_failed");
        }
        if !self.public_build_isolated {
            blockers.push("public_build_not_isolated");
        }
        if !self.k1_to_k8_verified {
            blockers.push("k1_to_k8_incomplete");
        }
        if blockers.is_empty() {
            ReleaseVerdict::Ready
        } else {
            ReleaseVerdict::Blocked(blockers)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_is_ready_only_when_all_gates_pass() {
        let all_good = ReleaseReadiness {
            ci_gates_passed: true,
            instrument_tests_passed: true,
            public_build_isolated: true,
            k1_to_k8_verified: true,
        };
        assert_eq!(all_good.evaluate(), ReleaseVerdict::Ready);

        let missing_ci = ReleaseReadiness {
            ci_gates_passed: false,
            ..all_good
        };
        assert_eq!(
            missing_ci.evaluate(),
            ReleaseVerdict::Blocked(vec!["ci_gates_failed"])
        );
    }
}
