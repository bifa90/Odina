// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K20 Certificate & Publisher Revocation Contract.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublisherCert {
    pub chain_valid: bool,
    pub revoked: bool,
    pub expired: bool,
}

impl PublisherCert {
    pub fn qualifies_for_signed_cap(&self) -> bool {
        self.chain_valid && !self.revoked && !self.expired
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revoked_or_expired_certs_never_qualify() {
        assert!(PublisherCert {
            chain_valid: true,
            revoked: false,
            expired: false,
        }
        .qualifies_for_signed_cap());

        assert!(!PublisherCert {
            chain_valid: true,
            revoked: true,
            expired: false,
        }
        .qualifies_for_signed_cap());

        assert!(!PublisherCert {
            chain_valid: true,
            revoked: false,
            expired: true,
        }
        .qualifies_for_signed_cap());
    }
}
