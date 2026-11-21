// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K41 Allow-List Ceiling Contract.

pub fn allow_list_applies(on_allow_list: bool, known_bad_hash: bool) -> bool {
    on_allow_list && !known_bad_hash
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn known_bad_beats_allow_list() {
        assert!(!allow_list_applies(true, true));
        assert!(allow_list_applies(true, false));
    }
}
