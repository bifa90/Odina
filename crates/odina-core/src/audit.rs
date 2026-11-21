// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K12 Audit Log & Tamper-Evident Chain Contract.
//!
//! Enforces an append-only hash-chained audit log where any modification
//! or deletion breaks `verify_chain()`.

use crate::hashing::sha256_hex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEntry {
    pub seq: u64,
    pub event: String,
    pub prev_hash: String,
    pub entry_hash: String,
}

#[derive(Debug, Default, Clone)]
pub struct AuditLog {
    entries: Vec<AuditEntry>,
}

impl AuditLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(&mut self, event: &str) {
        let seq = self.entries.len() as u64;
        let prev_hash = self
            .entries
            .last()
            .map(|e| e.entry_hash.clone())
            .unwrap_or_else(|| "0".repeat(64));
        let payload = format!("{seq}:{prev_hash}:{event}");
        let entry_hash = sha256_hex(payload.as_bytes());
        self.entries.push(AuditEntry {
            seq,
            event: event.to_string(),
            prev_hash,
            entry_hash,
        });
    }

    pub fn verify_chain(&self) -> bool {
        let mut expected_prev = "0".repeat(64);
        for (idx, e) in self.entries.iter().enumerate() {
            if e.seq != idx as u64 || e.prev_hash != expected_prev {
                return false;
            }
            let payload = format!("{}:{}:{}", e.seq, e.prev_hash, e.event);
            if sha256_hex(payload.as_bytes()) != e.entry_hash {
                return false;
            }
            expected_prev = e.entry_hash.clone();
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_chain_verifies_and_detects_tampering() {
        let mut log = AuditLog::new();
        log.append("service_started");
        log.append("quarantine_added");
        assert!(log.verify_chain());

        log.entries[0].event = "tampered".to_string();
        assert!(!log.verify_chain());
    }
}
