// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K42 Scan Completeness Contract.

pub fn is_scan_complete(unreadable_files: usize, unreadable_dirs: usize) -> bool {
    unreadable_files == 0 && unreadable_dirs == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unreadable_items_break_completeness() {
        assert!(is_scan_complete(0, 0));
        assert!(!is_scan_complete(1, 0));
        assert!(!is_scan_complete(0, 1));
    }
}
