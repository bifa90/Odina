// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K36 Service Crash-Loop Backoff Contract.

pub fn next_backoff_secs(consecutive_failures: u32) -> u64 {
    let shift = consecutive_failures.min(6);
    1u64 << shift
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn caps_exponential_backoff() {
        assert_eq!(next_backoff_secs(0), 1);
        assert_eq!(next_backoff_secs(3), 8);
        assert_eq!(next_backoff_secs(20), 64);
    }
}
