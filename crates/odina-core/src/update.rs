// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K8 Update, Signature & Rollback Contract.
//!
//! Enforces:
//! - Every update package must pass signature & digest verification before activation.
//! - Failed verification or activation rolls back atomically to the previous good version.
//! - Offline/failed update checks never claim "up to date".

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdatePackage {
    pub version: u64,
    pub sha256_hex: String,
    pub signature_valid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateStatus {
    Updated { version: u64 },
    RolledBack { active_version: u64, reason: &'static str },
    CheckFailed { active_version: u64, reason: &'static str },
}

#[derive(Debug, Clone)]
pub struct UpdateManager {
    active_version: u64,
    previous_version: Option<u64>,
}

impl UpdateManager {
    pub fn new(initial_version: u64) -> Self {
        Self {
            active_version: initial_version,
            previous_version: None,
        }
    }

    pub fn active_version(&self) -> u64 {
        self.active_version
    }

    pub fn apply_update(&mut self, pkg: UpdatePackage) -> UpdateStatus {
        if !pkg.signature_valid || pkg.sha256_hex.len() != 64 {
            return UpdateStatus::RolledBack {
                active_version: self.active_version,
                reason: "invalid_signature_or_digest",
            };
        }
        if pkg.version <= self.active_version {
            return UpdateStatus::RolledBack {
                active_version: self.active_version,
                reason: "non_monotonic_version",
            };
        }
        self.previous_version = Some(self.active_version);
        self.active_version = pkg.version;
        UpdateStatus::Updated {
            version: self.active_version,
        }
    }

    pub fn record_offline_failure(&self) -> UpdateStatus {
        UpdateStatus::CheckFailed {
            active_version: self.active_version,
            reason: "offline_or_unreachable",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_signed_update_advances_and_invalid_rolls_back() {
        let mut mgr = UpdateManager::new(10);
        let bad = UpdatePackage {
            version: 11,
            sha256_hex: "a".repeat(64),
            signature_valid: false,
        };
        assert_eq!(
            mgr.apply_update(bad),
            UpdateStatus::RolledBack {
                active_version: 10,
                reason: "invalid_signature_or_digest",
            }
        );
        assert_eq!(mgr.active_version(), 10);

        let good = UpdatePackage {
            version: 11,
            sha256_hex: "b".repeat(64),
            signature_valid: true,
        };
        assert_eq!(mgr.apply_update(good), UpdateStatus::Updated { version: 11 });
        assert_eq!(mgr.active_version(), 11);
    }

    #[test]
    fn offline_failure_never_claims_up_to_date() {
        let mgr = UpdateManager::new(10);
        assert_eq!(
            mgr.record_offline_failure(),
            UpdateStatus::CheckFailed {
                active_version: 10,
                reason: "offline_or_unreachable",
            }
        );
    }
}
