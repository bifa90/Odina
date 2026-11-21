// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K18 Network Share & UNC Isolation Contract.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkScopeDecision {
    RejectedFromRealtime,
    OnDemandWithConsent,
}

pub fn evaluate_network_path(path: &str, operator_consent: bool) -> Option<NetworkScopeDecision> {
    let is_unc = path.starts_with("\\\\") || path.starts_with("//");
    if !is_unc {
        return None;
    }
    if operator_consent {
        Some(NetworkScopeDecision::OnDemandWithConsent)
    } else {
        Some(NetworkScopeDecision::RejectedFromRealtime)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unc_paths_require_consent_and_stay_out_of_realtime() {
        assert_eq!(
            evaluate_network_path("\\\\server\\share\\a.exe", false),
            Some(NetworkScopeDecision::RejectedFromRealtime)
        );
        assert_eq!(
            evaluate_network_path("\\\\server\\share\\a.exe", true),
            Some(NetworkScopeDecision::OnDemandWithConsent)
        );
        assert_eq!(evaluate_network_path("C:\\local\\a.exe", false), None);
    }
}
