// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K22 Memory & Resource Cap Contract.

pub const MAX_SCAN_MEMORY_MB: u64 = 512;
pub const MAX_WORKERS: u8 = 4;

pub fn within_resource_caps(memory_mb: u64, workers: u8) -> bool {
    memory_mb <= MAX_SCAN_MEMORY_MB && (1..=MAX_WORKERS).contains(&workers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enforces_memory_and_worker_caps() {
        assert!(within_resource_caps(256, 2));
        assert!(!within_resource_caps(1024, 2));
        assert!(!within_resource_caps(256, 8));
    }
}
