// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K28 Offline Installer & Integrity Contract.

pub fn verify_offline_bundle(
    signature_ok: bool,
    manifest_hash_ok: bool,
    network_required: bool,
) -> bool {
    signature_ok && manifest_hash_ok && !network_required
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offline_bundle_requires_no_network() {
        assert!(verify_offline_bundle(true, true, false));
        assert!(!verify_offline_bundle(true, true, true));
        assert!(!verify_offline_bundle(false, true, false));
    }
}
