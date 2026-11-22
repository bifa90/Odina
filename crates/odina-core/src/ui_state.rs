// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K5 Desktop UI State & Localization Contract.
//!
//! Enforces:
//! - Unified state shared across Home, Alert Center, Quarantine, Tray, Toasts, and Activity Log.
//! - Never reports "clean" when the service is stopped or disconnected.
//! - Offline RTL/LTR direction resolution for supported languages.

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

    pub fn tray_and_toast_summary(&self) -> &'static str {
        match self.health {
            ServiceHealth::Stopped => "status.stopped",
            ServiceHealth::Disconnected => "status.disconnected",
            ServiceHealth::Suspended => "status.suspended",
            ServiceHealth::Active if self.active_alerts > 0 => "status.attention_needed",
            ServiceHealth::Active => "status.protected",
        }
    }
}

#[cfg(test)]
mod tests {
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
    fn tray_and_toast_never_claim_protected_when_stopped_or_disconnected() {
        assert_eq!(
            UiState::new(ServiceHealth::Stopped, "ar").tray_and_toast_summary(),
            "status.stopped"
        );
        assert_eq!(
            UiState::new(ServiceHealth::Disconnected, "en").tray_and_toast_summary(),
            "status.disconnected"
        );
        assert_eq!(
            UiState::new(ServiceHealth::Suspended, "en").tray_and_toast_summary(),
            "status.suspended"
        );
        let mut alert_state = UiState::new(ServiceHealth::Active, "ar");
        alert_state.active_alerts = 2;
        assert_eq!(
            alert_state.tray_and_toast_summary(),
            "status.attention_needed"
        );
    }
}
