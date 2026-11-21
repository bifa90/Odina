// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K23 Symlink & Junction Loop Guard Contract.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkDecision {
    Follow,
    SkipLoopOrJunction,
}

pub fn evaluate_link(is_symlink_or_junction: bool, points_to_ancestor: bool) -> LinkDecision {
    if is_symlink_or_junction && points_to_ancestor {
        LinkDecision::SkipLoopOrJunction
    } else {
        LinkDecision::Follow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_ancestor_loops() {
        assert_eq!(evaluate_link(true, true), LinkDecision::SkipLoopOrJunction);
        assert_eq!(evaluate_link(false, false), LinkDecision::Follow);
    }
}
