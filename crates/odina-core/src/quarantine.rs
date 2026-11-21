// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K4 Quarantine Store Contract.
//!
//! Enforces:
//! - Explicit move-to-quarantine record with original path, SHA-256, and reason.
//! - Capacity limit: refuses new entries when full, leaving the incident pending.
//! - Verified restore: checks SHA-256 identity & content, refuses path conflicts,
//!   and refuses restoring a known-bad item without explicit override.

use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineEntry {
    pub id: u64,
    pub original_path: String,
    pub sha256_hex: String,
    pub reason: String,
    pub known_bad: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuarantineError {
    StoreFull { capacity: usize },
    NotFound(u64),
    IdentityMismatch { expected: String, actual: String },
    PathConflict(String),
    KnownBadRestoreBlocked(u64),
}

impl fmt::Display for QuarantineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StoreFull { capacity } => write!(f, "quarantine store full (capacity={capacity})"),
            Self::NotFound(id) => write!(f, "quarantine entry {id} not found"),
            Self::IdentityMismatch { expected, actual } => {
                write!(f, "quarantine identity mismatch (expected={expected}, actual={actual})")
            }
            Self::PathConflict(path) => write!(f, "restore path already exists: {path}"),
            Self::KnownBadRestoreBlocked(id) => {
                write!(f, "refusing to restore known-bad quarantine entry {id}")
            }
        }
    }
}

impl std::error::Error for QuarantineError {}

#[derive(Debug, Clone)]
pub struct QuarantineStore {
    capacity: usize,
    next_id: u64,
    entries: BTreeMap<u64, QuarantineEntry>,
}

impl QuarantineStore {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            next_id: 1,
            entries: BTreeMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn quarantine(
        &mut self,
        original_path: impl Into<String>,
        sha256_hex: impl Into<String>,
        reason: impl Into<String>,
        known_bad: bool,
    ) -> Result<u64, QuarantineError> {
        if self.entries.len() >= self.capacity {
            return Err(QuarantineError::StoreFull {
                capacity: self.capacity,
            });
        }
        let id = self.next_id;
        self.next_id += 1;
        self.entries.insert(
            id,
            QuarantineEntry {
                id,
                original_path: original_path.into(),
                sha256_hex: sha256_hex.into(),
                reason: reason.into(),
                known_bad,
            },
        );
        Ok(id)
    }

    pub fn get(&self, id: u64) -> Option<&QuarantineEntry> {
        self.entries.get(&id)
    }

    pub fn restore_verified(
        &mut self,
        id: u64,
        actual_sha256_hex: &str,
        target_exists: bool,
        allow_known_bad: bool,
    ) -> Result<QuarantineEntry, QuarantineError> {
        let entry = self
            .entries
            .get(&id)
            .ok_or(QuarantineError::NotFound(id))?;
        if entry.known_bad && !allow_known_bad {
            return Err(QuarantineError::KnownBadRestoreBlocked(id));
        }
        if entry.sha256_hex != actual_sha256_hex {
            return Err(QuarantineError::IdentityMismatch {
                expected: entry.sha256_hex.clone(),
                actual: actual_sha256_hex.to_string(),
            });
        }
        if target_exists {
            return Err(QuarantineError::PathConflict(entry.original_path.clone()));
        }
        Ok(self.entries.remove(&id).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quarantine_and_verified_restore_succeeds() {
        let mut store = QuarantineStore::new(2);
        let id = store
            .quarantine("/tmp/sample.bin", "abc123", "suspicious", false)
            .unwrap();
        assert_eq!(store.len(), 1);
        let restored = store.restore_verified(id, "abc123", false, false).unwrap();
        assert_eq!(restored.original_path, "/tmp/sample.bin");
        assert!(store.is_empty());
    }

    #[test]
    fn restore_blocks_mismatch_conflict_and_known_bad() {
        let mut store = QuarantineStore::new(1);
        let id = store
            .quarantine("/tmp/eicar.com", "deadbeef", "known_bad", true)
            .unwrap();
        assert_eq!(
            store.quarantine("/tmp/other.bin", "111", "suspicious", false),
            Err(QuarantineError::StoreFull { capacity: 1 })
        );
        assert_eq!(
            store.restore_verified(id, "deadbeef", false, false),
            Err(QuarantineError::KnownBadRestoreBlocked(id))
        );
        assert_eq!(
            store.restore_verified(id, "wrong", false, true),
            Err(QuarantineError::IdentityMismatch {
                expected: "deadbeef".into(),
                actual: "wrong".into(),
            })
        );
        assert_eq!(
            store.restore_verified(id, "deadbeef", true, true),
            Err(QuarantineError::PathConflict("/tmp/eicar.com".into()))
        );
        assert_eq!(store.len(), 1);
    }
}
