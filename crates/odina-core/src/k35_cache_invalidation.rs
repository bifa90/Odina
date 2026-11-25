// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K35 Cache Invalidation on DB Version Bump Contract.

pub fn is_cache_entry_valid(entry_db_version: u64, current_db_version: u64) -> bool {
    entry_db_version == current_db_version && current_db_version > 0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalidates_on_db_bump() {
        assert!(is_cache_entry_valid(5, 5));
        assert!(!is_cache_entry_valid(5, 6));
    }
}
