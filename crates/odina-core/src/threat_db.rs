// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K1 Threat Database & Update Contract.
//!
//! Enforces:
//! - Trusted source, license, signature verification, and SHA-256 payload integrity.
//! - Last-known-good fallback: a failed update never replaces or corrupts the active DB.
//! - Monotonic version tracking so the Smart Cache invalidates when the DB version changes.

use crate::hashing::sha256_hex;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreatDbMeta {
    pub version: u64,
    pub source: String,
    pub license: String,
    pub digest_sha256: [u8; 32],
    pub signature_verified: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThreatDbError {
    UntrustedSource,
    InvalidLicense,
    UnverifiedSignature,
    DigestMismatch,
    VersionRollback { current: u64, candidate: u64 },
    MalformedPayload(usize),
}

impl fmt::Display for ThreatDbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UntrustedSource => write!(f, "untrusted threat database source"),
            Self::InvalidLicense => write!(f, "invalid threat database license"),
            Self::UnverifiedSignature => write!(f, "unverified threat database signature"),
            Self::DigestMismatch => write!(f, "threat database payload digest mismatch"),
            Self::VersionRollback { current, candidate } => write!(
                f,
                "threat database version rollback rejected (current={current}, candidate={candidate})"
            ),
            Self::MalformedPayload(line) => {
                write!(f, "malformed threat database entry at line {line}")
            }
        }
    }
}

impl std::error::Error for ThreatDbError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreatDb {
    meta: ThreatDbMeta,
    hashes: Vec<[u8; 32]>,
}

impl ThreatDb {
    pub fn load_verified(
        meta: ThreatDbMeta,
        payload: &[u8],
        expected_source: &str,
        expected_license: &str,
    ) -> Result<Self, ThreatDbError> {
        if meta.source != expected_source {
            return Err(ThreatDbError::UntrustedSource);
        }
        if meta.license != expected_license {
            return Err(ThreatDbError::InvalidLicense);
        }
        if !meta.signature_verified {
            return Err(ThreatDbError::UnverifiedSignature);
        }
        if parse_hex_32(&sha256_hex(payload)) != Some(meta.digest_sha256) {
            return Err(ThreatDbError::DigestMismatch);
        }

        let text = std::str::from_utf8(payload).map_err(|_| ThreatDbError::MalformedPayload(1))?;
        let mut hashes = Vec::new();
        for (idx, raw_line) in text.lines().enumerate() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let digest = parse_hex_32(line).ok_or(ThreatDbError::MalformedPayload(idx + 1))?;
            hashes.push(digest);
        }
        hashes.sort_unstable();
        hashes.dedup();

        Ok(Self { meta, hashes })
    }

    pub fn version(&self) -> u64 {
        self.meta.version
    }

    pub fn meta(&self) -> &ThreatDbMeta {
        &self.meta
    }

    pub fn contains_sha256(&self, digest: &[u8; 32]) -> bool {
        self.hashes.binary_search(digest).is_ok()
    }

    pub fn len(&self) -> usize {
        self.hashes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.hashes.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct ThreatDbStore {
    expected_source: String,
    expected_license: String,
    active: ThreatDb,
    last_known_good: Option<ThreatDb>,
}

impl ThreatDbStore {
    pub fn new(
        expected_source: impl Into<String>,
        expected_license: impl Into<String>,
        initial: ThreatDb,
    ) -> Self {
        Self {
            expected_source: expected_source.into(),
            expected_license: expected_license.into(),
            active: initial,
            last_known_good: None,
        }
    }

    pub fn active(&self) -> &ThreatDb {
        &self.active
    }

    pub fn active_version(&self) -> u64 {
        self.active.version()
    }

    pub fn try_update(&mut self, meta: ThreatDbMeta, payload: &[u8]) -> Result<u64, ThreatDbError> {
        if meta.version <= self.active.version() {
            return Err(ThreatDbError::VersionRollback {
                current: self.active.version(),
                candidate: meta.version,
            });
        }
        let next = ThreatDb::load_verified(
            meta,
            payload,
            &self.expected_source,
            &self.expected_license,
        )?;
        let prev = std::mem::replace(&mut self.active, next);
        self.last_known_good = Some(prev);
        Ok(self.active.version())
    }

    pub fn rollback(&mut self) -> Option<u64> {
        let prev = self.last_known_good.take()?;
        self.active = prev;
        Some(self.active.version())
    }
}

fn parse_hex_32(s: &str) -> Option<[u8; 32]> {
    if s.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    let bytes = s.as_bytes();
    for i in 0..32 {
        let hi = hex_nibble(bytes[i * 2])?;
        let lo = hex_nibble(bytes[i * 2 + 1])?;
        out[i] = (hi << 4) | lo;
    }
    Some(out)
}

fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = "odina-official";
    const LICENSE: &str = "MIT";
    const EICAR_HEX: &str = "275a021bbfb6489e54d471899f7db9d1663fc695ec2fe2a2c4538aabf651fd0f";
    const OTHER_HEX: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

    fn make_meta(version: u64, payload: &[u8]) -> ThreatDbMeta {
        ThreatDbMeta {
            version,
            source: SOURCE.to_string(),
            license: LICENSE.to_string(),
            digest_sha256: parse_hex_32(&sha256_hex(payload)).unwrap(),
            signature_verified: true,
        }
    }

    #[test]
    fn verified_db_loads_and_matches_known_digest() {
        let payload = format!("# v1\n{EICAR_HEX}\n").into_bytes();
        let db = ThreatDb::load_verified(make_meta(1, &payload), &payload, SOURCE, LICENSE).unwrap();
        assert_eq!(db.version(), 1);
        assert_eq!(db.len(), 1);
        let eicar = parse_hex_32(EICAR_HEX).unwrap();
        let other = parse_hex_32(OTHER_HEX).unwrap();
        assert!(db.contains_sha256(&eicar));
        assert!(!db.contains_sha256(&other));
    }

    #[test]
    fn failed_update_keeps_last_known_good_active() {
        let p1 = format!("{EICAR_HEX}\n").into_bytes();
        let db1 = ThreatDb::load_verified(make_meta(1, &p1), &p1, SOURCE, LICENSE).unwrap();
        let mut store = ThreatDbStore::new(SOURCE, LICENSE, db1);

        // 1. Tampered payload -> DigestMismatch
        let p2 = format!("{OTHER_HEX}\n").into_bytes();
        let mut bad_meta = make_meta(2, &p2);
        bad_meta.digest_sha256[0] ^= 0xff;
        assert_eq!(
            store.try_update(bad_meta, &p2),
            Err(ThreatDbError::DigestMismatch)
        );
        assert_eq!(store.active_version(), 1);

        // 2. Unverified signature -> UnverifiedSignature
        let mut unsig = make_meta(2, &p2);
        unsig.signature_verified = false;
        assert_eq!(
            store.try_update(unsig, &p2),
            Err(ThreatDbError::UnverifiedSignature)
        );
        assert_eq!(store.active_version(), 1);

        // 3. Valid update to v2 succeeds, and rollback restores v1
        let v2 = store.try_update(make_meta(2, &p2), &p2).unwrap();
        assert_eq!(v2, 2);
        assert!(store.active().contains_sha256(&parse_hex_32(OTHER_HEX).unwrap()));
        assert_eq!(store.rollback(), Some(1));
        assert!(store.active().contains_sha256(&parse_hex_32(EICAR_HEX).unwrap()));
    }

    #[test]
    fn version_rollback_in_update_is_rejected() {
        let p1 = format!("{EICAR_HEX}\n").into_bytes();
        let db1 = ThreatDb::load_verified(make_meta(5, &p1), &p1, SOURCE, LICENSE).unwrap();
        let mut store = ThreatDbStore::new(SOURCE, LICENSE, db1);

        let p_old = format!("{OTHER_HEX}\n").into_bytes();
        assert_eq!(
            store.try_update(make_meta(5, &p_old), &p_old),
            Err(ThreatDbError::VersionRollback {
                current: 5,
                candidate: 5
            })
        );
        assert_eq!(store.active_version(), 5);
    }
}
