// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K14 Threat DB Freshness & Pinning Contract.
//!
//! Enforces that stale or unpinned threat databases are flagged
//! and never silently report full protection.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreatDbStatus {
    pub db_version: u64,
    pub age_hours: u64,
    pub max_fresh_hours: u64,
    pub pinned_digest_valid: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbHealth {
    Fresh,
    Stale,
    UnpinnedOrCorrupt,
}

impl ThreatDbStatus {
    pub fn evaluate(&self) -> DbHealth {
        if !self.pinned_digest_valid || self.db_version == 0 {
            return DbHealth::UnpinnedOrCorrupt;
        }
        if self.age_hours > self.max_fresh_hours {
            return DbHealth::Stale;
        }
        DbHealth::Fresh
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_stale_and_unpinned_databases() {
        let fresh = ThreatDbStatus {
            db_version: 42,
            age_hours: 12,
            max_fresh_hours: 72,
            pinned_digest_valid: true,
        };
        assert_eq!(fresh.evaluate(), DbHealth::Fresh);

        let stale = ThreatDbStatus {
            age_hours: 96,
            ..fresh
        };
        assert_eq!(stale.evaluate(), DbHealth::Stale);

        let unpinned = ThreatDbStatus {
            pinned_digest_valid: false,
            ..fresh
        };
        assert_eq!(unpinned.evaluate(), DbHealth::UnpinnedOrCorrupt);
    }
}
