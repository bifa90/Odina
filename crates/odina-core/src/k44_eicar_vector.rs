// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K44 EICAR Standard Vector Contract.

pub const EICAR_LEN: usize = 68;

pub fn is_eicar_length(len: usize) -> bool {
    len == EICAR_LEN
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn eicar_is_68_bytes() {
        assert!(is_eicar_length(68));
        assert!(!is_eicar_length(69));
    }
}
