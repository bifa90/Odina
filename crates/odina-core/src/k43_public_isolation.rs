// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K43 Public Build Isolation Contract.

pub fn public_build_exposes_internal_check(is_public_build: bool) -> bool {
    !is_public_build
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn public_build_hides_internal_check() {
        assert!(!public_build_exposes_internal_check(true));
        assert!(public_build_exposes_internal_check(false));
    }
}
