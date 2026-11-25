// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K37 CLI Exit Code Determinism Contract.

pub fn map_scan_exit_code(threats_found: usize, scan_error: bool) -> i32 {
    if scan_error {
        2
    } else if threats_found > 0 {
        1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn maps_clean_threat_and_error_codes() {
        assert_eq!(map_scan_exit_code(0, false), 0);
        assert_eq!(map_scan_exit_code(1, false), 1);
        assert_eq!(map_scan_exit_code(0, true), 2);
    }
}
