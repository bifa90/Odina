// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K11 Scheduled Scan & Idle-Budget Contract.
//!
//! Enforces:
//! - No overlapping scheduled scans.
//! - Respects suspended state and battery-saver/non-idle budget.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScanBudget {
    pub service_suspended: bool,
    pub on_battery_saver: bool,
    pub scan_in_progress: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleDecision {
    Start,
    Defer(&'static str),
}

pub fn evaluate_scheduled_scan(budget: ScanBudget) -> ScheduleDecision {
    if budget.service_suspended {
        return ScheduleDecision::Defer("service_suspended");
    }
    if budget.scan_in_progress {
        return ScheduleDecision::Defer("scan_already_in_progress");
    }
    if budget.on_battery_saver {
        return ScheduleDecision::Defer("battery_saver_active");
    }
    ScheduleDecision::Start
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defers_when_suspended_busy_or_on_battery_saver() {
        assert_eq!(
            evaluate_scheduled_scan(ScanBudget {
                service_suspended: true,
                on_battery_saver: false,
                scan_in_progress: false,
            }),
            ScheduleDecision::Defer("service_suspended")
        );
        assert_eq!(
            evaluate_scheduled_scan(ScanBudget {
                service_suspended: false,
                on_battery_saver: false,
                scan_in_progress: true,
            }),
            ScheduleDecision::Defer("scan_already_in_progress")
        );
        assert_eq!(
            evaluate_scheduled_scan(ScanBudget {
                service_suspended: false,
                on_battery_saver: true,
                scan_in_progress: false,
            }),
            ScheduleDecision::Defer("battery_saver_active")
        );
        assert_eq!(
            evaluate_scheduled_scan(ScanBudget {
                service_suspended: false,
                on_battery_saver: false,
                scan_in_progress: false,
            }),
            ScheduleDecision::Start
        );
    }
}
