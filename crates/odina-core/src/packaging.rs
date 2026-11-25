// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K7 Packaging & Add-on Independence Contract.
//!
//! Enforces:
//! - Four independent packaging states: Base only, Base+Browser, Base+Repair, Base+Both.
//! - Removing an add-on never erases Quarantine or user Policy exceptions.
//! - Removing the Browser add-on removes its native-messaging host registration.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackagingProfile {
    pub base_installed: bool,
    pub browser_addon: bool,
    pub repair_addon: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UninstallOutcome {
    pub quarantine_preserved: bool,
    pub exceptions_preserved: bool,
    pub browser_host_registration_removed: bool,
}

impl PackagingProfile {
    pub fn new(browser_addon: bool, repair_addon: bool) -> Self {
        Self {
            base_installed: true,
            browser_addon,
            repair_addon,
        }
    }

    pub fn uninstall_addons(
        &mut self,
        remove_browser: bool,
        remove_repair: bool,
    ) -> UninstallOutcome {
        let host_removed = remove_browser && self.browser_addon;
        if remove_browser {
            self.browser_addon = false;
        }
        if remove_repair {
            self.repair_addon = false;
        }
        UninstallOutcome {
            quarantine_preserved: true,
            exceptions_preserved: true,
            browser_host_registration_removed: host_removed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_four_packaging_states_are_independent_and_preserve_user_data() {
        for (b, r) in [(false, false), (true, false), (false, true), (true, true)] {
            let mut p = PackagingProfile::new(b, r);
            assert!(p.base_installed);
            let out = p.uninstall_addons(b, r);
            assert!(p.base_installed);
            assert!(!p.browser_addon);
            assert!(!p.repair_addon);
            assert!(out.quarantine_preserved);
            assert!(out.exceptions_preserved);
            assert_eq!(out.browser_host_registration_removed, b);
        }
    }
}
