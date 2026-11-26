// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K5 Desktop UI State & Localization Contract.
//!
//! Enforces:
//! - Unified state shared across Home, Alert Center, Quarantine, Tray, Toasts, and Activity Log.
//! - Never reports "clean" when the service is stopped or disconnected.
//! - Offline RTL/LTR direction resolution for supported languages.

use crate::messages;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceHealth {
    Active,
    Suspended,
    Stopped,
    Disconnected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDirection {
    Rtl,
    Ltr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiState {
    pub health: ServiceHealth,
    pub active_alerts: usize,
    pub quarantine_count: usize,
    pub lang: String,
}

impl UiState {
    pub fn new(health: ServiceHealth, lang: impl Into<String>) -> Self {
        Self {
            health,
            active_alerts: 0,
            quarantine_count: 0,
            lang: lang.into(),
        }
    }

    pub fn direction(&self) -> TextDirection {
        match self.lang.as_str() {
            "ar" | "fa" | "ur" | "he" => TextDirection::Rtl,
            _ => TextDirection::Ltr,
        }
    }

    /// The one state line, as a key the locale files resolve.
    ///
    /// The keys were `status.*`, which no `.ftl` file defines: the tests passed
    /// because they compared a key with itself, and a screen would have shown an
    /// empty label. They are the `state.*` keys now, and `messages::UI_KEYS`
    /// holds them so a missing translation fails the build instead of the user.
    pub fn state_key(&self) -> &'static str {
        match self.health {
            ServiceHealth::Stopped => messages::STATE_UNPROTECTED,
            ServiceHealth::Disconnected => messages::STATE_UNTRUSTED,
            ServiceHealth::Suspended => messages::STATE_PAUSED,
            ServiceHealth::Active if self.active_alerts > 0 => messages::STATE_ATTENTION,
            ServiceHealth::Active => messages::STATE_PROTECTED,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::messages;
    use super::*;

    #[test]
    fn rtl_and_ltr_resolve_offline() {
        assert_eq!(
            UiState::new(ServiceHealth::Active, "ar").direction(),
            TextDirection::Rtl
        );
        assert_eq!(
            UiState::new(ServiceHealth::Active, "en").direction(),
            TextDirection::Ltr
        );
    }

    #[test]
    fn the_state_line_never_claims_protected_when_stopped_or_disconnected() {
        assert_eq!(
            UiState::new(ServiceHealth::Stopped, "ar").state_key(),
            "state.unprotected"
        );
        assert_eq!(
            UiState::new(ServiceHealth::Disconnected, "en").state_key(),
            "state.untrusted"
        );
        assert_eq!(
            UiState::new(ServiceHealth::Suspended, "en").state_key(),
            "state.paused"
        );
        let mut alert_state = UiState::new(ServiceHealth::Active, "ar");
        alert_state.active_alerts = 2;
        assert_eq!(alert_state.state_key(), "state.attention");
        assert_eq!(
            UiState::new(ServiceHealth::Active, "ar").state_key(),
            "state.protected"
        );
        // Every answer is a key the locale files actually define.
        for health in [
            ServiceHealth::Active,
            ServiceHealth::Suspended,
            ServiceHealth::Stopped,
            ServiceHealth::Disconnected,
        ] {
            let key = UiState::new(health, "ar").state_key();
            assert!(messages::UI_KEYS.contains(&key), "{key} not in UI_KEYS");
        }
    }
}
