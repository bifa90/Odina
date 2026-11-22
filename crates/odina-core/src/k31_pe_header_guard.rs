// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K31 Malformed/Truncated PE Header Guard Contract.

pub fn is_valid_pe_prefix(bytes: &[u8]) -> bool {
    bytes.len() >= 64 && bytes[0] == b'M' && bytes[1] == b'Z'
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_truncated_or_non_mz() {
        assert!(!is_valid_pe_prefix(b"MZ"));
        assert!(!is_valid_pe_prefix(&[0u8; 64]));
    }
}
