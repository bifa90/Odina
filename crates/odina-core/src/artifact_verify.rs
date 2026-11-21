// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K15 Remediated Artifact Verification Contract.
//!
//! Enforces that a repaired artifact is only released after both
//! post-repair digest verification and a clean final rescan.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactCheck {
    pub expected_sha256: String,
    pub actual_sha256: String,
    pub final_rescan_clean: bool,
}

pub fn can_release_repaired_artifact(check: &ArtifactCheck) -> bool {
    check.final_rescan_clean
        && check.expected_sha256.len() == 64
        && check.expected_sha256 == check.actual_sha256
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_exact_digest_and_clean_rescan() {
        let ok = ArtifactCheck {
            expected_sha256: "a".repeat(64),
            actual_sha256: "a".repeat(64),
            final_rescan_clean: true,
        };
        assert!(can_release_repaired_artifact(&ok));

        let dirty = ArtifactCheck {
            final_rescan_clean: false,
            ..ok.clone()
        };
        assert!(!can_release_repaired_artifact(&dirty));

        let mismatch = ArtifactCheck {
            actual_sha256: "b".repeat(64),
            ..ok
        };
        assert!(!can_release_repaired_artifact(&mismatch));
    }
}
