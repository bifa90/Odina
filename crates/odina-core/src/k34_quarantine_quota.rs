// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K34 Quarantine Quota Enforcement Contract.

pub fn can_accept_quarantine_item(used_bytes: u64, item_bytes: u64, quota_bytes: u64) -> bool {
    used_bytes.saturating_add(item_bytes) <= quota_bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn enforces_quarantine_quota() {
        assert!(can_accept_quarantine_item(500, 500, 1000));
        assert!(!can_accept_quarantine_item(600, 500, 1000));
    }
}
