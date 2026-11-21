// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K39 Dependency-Free Date Format Contract.

pub fn is_valid_iso_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter().enumerate().all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_yyyy_mm_dd() {
        assert!(is_valid_iso_date("2026-11-20"));
        assert!(!is_valid_iso_date("2026/11/20"));
    }
}
