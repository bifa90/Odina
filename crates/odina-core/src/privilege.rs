// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K16 Multi-User & Privilege Boundary Contract.
//!
//! Enforces that unprivileged users cannot disable protection,
//! purge quarantine, or grant permanent whole-rule exceptions.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    StandardUser,
    Administrator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivilegedAction {
    DisableRealtimeProtection,
    PurgeQuarantine,
    GrantPermanentWholeRuleException,
    RunUserScan,
}

pub fn is_authorized(role: Role, action: PrivilegedAction) -> bool {
    match action {
        PrivilegedAction::RunUserScan => true,
        PrivilegedAction::DisableRealtimeProtection
        | PrivilegedAction::PurgeQuarantine
        | PrivilegedAction::GrantPermanentWholeRuleException => role == Role::Administrator,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_user_cannot_perform_admin_actions() {
        assert!(is_authorized(
            Role::StandardUser,
            PrivilegedAction::RunUserScan
        ));
        assert!(!is_authorized(
            Role::StandardUser,
            PrivilegedAction::DisableRealtimeProtection
        ));
        assert!(!is_authorized(
            Role::StandardUser,
            PrivilegedAction::PurgeQuarantine
        ));
        assert!(!is_authorized(
            Role::StandardUser,
            PrivilegedAction::GrantPermanentWholeRuleException
        ));
        assert!(is_authorized(
            Role::Administrator,
            PrivilegedAction::DisableRealtimeProtection
        ));
    }
}
