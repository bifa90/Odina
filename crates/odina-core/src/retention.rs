// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K24 Quarantine Retention & Expiry Contract.

pub const DEFAULT_RETENTION_DAYS: u32 = 30;

pub fn is_expired_for_purge(age_days: u32, max_days: u32) -> bool {
    age_days > max_days
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn purges_only_after_retention_window() {
        assert!(!is_expired_for_purge(30, DEFAULT_RETENTION_DAYS));
        assert!(is_expired_for_purge(31, DEFAULT_RETENTION_DAYS));
    }
}
