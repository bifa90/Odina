// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K3 Behavior Wall & Incident Lifecycle.
//!
//! Enforces:
//! - Explicit start, scope, and end/cancel for every behavior-wall reason.
//! - Overlapping reasons: cancelling or ending one leaves others active;
//!   zero lingering activity once the last reason ends.
//! - Clear separation between silencing an incident and disabling protection
//!   (which requires showing both warnings first).

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncidentReason {
    DirectFileScan,
    DeviceInfection,
    RiskEntry,
    QuickHotspotScan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncidentRecord {
    pub id: u64,
    pub reason: IncidentReason,
    pub scope: String,
    pub silenced: bool,
}

#[derive(Debug, Default)]
pub struct BehaviorWall {
    next_id: u64,
    active: BTreeMap<u64, IncidentRecord>,
    protection_disabled: bool,
}

impl BehaviorWall {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            active: BTreeMap::new(),
            protection_disabled: false,
        }
    }

    pub fn start(&mut self, reason: IncidentReason, scope: impl Into<String>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.active.insert(
            id,
            IncidentRecord {
                id,
                reason,
                scope: scope.into(),
                silenced: false,
            },
        );
        id
    }

    pub fn end(&mut self, id: u64) -> bool {
        self.active.remove(&id).is_some()
    }

    pub fn cancel(&mut self, id: u64) -> bool {
        self.active.remove(&id).is_some()
    }

    pub fn is_active(&self) -> bool {
        !self.active.is_empty() && !self.protection_disabled
    }

    pub fn active_count(&self) -> usize {
        self.active.len()
    }

    pub fn silence(&mut self, id: u64) -> bool {
        if let Some(rec) = self.active.get_mut(&id) {
            rec.silenced = true;
            true
        } else {
            false
        }
    }

    pub fn is_protection_disabled(&self) -> bool {
        self.protection_disabled
    }

    pub fn disable_protection_with_warnings(
        &mut self,
        warnings_shown: u8,
    ) -> Result<(), &'static str> {
        if warnings_shown < 2 {
            return Err("both warnings must be shown before disabling protection");
        }
        self.protection_disabled = true;
        Ok(())
    }

    pub fn enable_protection(&mut self) {
        self.protection_disabled = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlapping_reasons_and_cancel_leave_zero_activity_after_last_ends() {
        let mut wall = BehaviorWall::new();
        assert!(!wall.is_active());

        let a = wall.start(IncidentReason::DirectFileScan, "/tmp/a.exe");
        let b = wall.start(IncidentReason::QuickHotspotScan, "/tmp/hot");
        assert!(wall.is_active());
        assert_eq!(wall.active_count(), 2);

        assert!(wall.cancel(a));
        assert!(wall.is_active());
        assert_eq!(wall.active_count(), 1);

        assert!(wall.end(b));
        assert!(!wall.is_active());
        assert_eq!(wall.active_count(), 0);
    }

    #[test]
    fn silence_keeps_protection_enabled_and_disable_requires_two_warnings() {
        let mut wall = BehaviorWall::new();
        let id = wall.start(IncidentReason::RiskEntry, "/tmp/risk.bin");
        assert!(wall.silence(id));
        assert!(!wall.is_protection_disabled());
        assert!(wall.is_active());

        assert!(wall.disable_protection_with_warnings(1).is_err());
        assert!(!wall.is_protection_disabled());

        assert!(wall.disable_protection_with_warnings(2).is_ok());
        assert!(wall.is_protection_disabled());
        assert!(!wall.is_active());

        wall.enable_protection();
        assert!(!wall.is_protection_disabled());
        assert!(wall.is_active());
    }
}
