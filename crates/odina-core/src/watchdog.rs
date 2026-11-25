// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K21 Watchdog & Heartbeat Liveness Contract.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerLiveness {
    Alive,
    Stalled,
}

pub fn check_heartbeat(elapsed_ms: u64, timeout_ms: u64) -> WorkerLiveness {
    if elapsed_ms > timeout_ms {
        WorkerLiveness::Stalled
    } else {
        WorkerLiveness::Alive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_stalled_worker_after_timeout() {
        assert_eq!(check_heartbeat(500, 1000), WorkerLiveness::Alive);
        assert_eq!(check_heartbeat(1001, 1000), WorkerLiveness::Stalled);
    }
}
