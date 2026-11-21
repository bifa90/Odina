// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K17 USB & Removable Media Policy Contract.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaAction {
    AllowMounted,
    ScanBeforeAutorun,
    BlockAutorun,
}

pub fn evaluate_removable_mount(has_autorun: bool, scanned_clean: bool) -> MediaAction {
    if has_autorun && !scanned_clean {
        MediaAction::BlockAutorun
    } else if !scanned_clean {
        MediaAction::ScanBeforeAutorun
    } else {
        MediaAction::AllowMounted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_unscanned_autorun_and_requires_scan() {
        assert_eq!(evaluate_removable_mount(true, false), MediaAction::BlockAutorun);
        assert_eq!(evaluate_removable_mount(false, false), MediaAction::ScanBeforeAutorun);
        assert_eq!(evaluate_removable_mount(false, true), MediaAction::AllowMounted);
    }
}
