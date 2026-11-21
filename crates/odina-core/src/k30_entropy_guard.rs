// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K30 High-Entropy Payload Guard Contract.

pub fn is_high_entropy_packed(shannon_bits_per_byte: f32) -> bool {
    shannon_bits_per_byte >= 7.2
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn flags_high_entropy_only() {
        assert!(is_high_entropy_packed(7.5));
        assert!(!is_high_entropy_packed(5.0));
    }
}
