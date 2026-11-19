// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! One local test signature, NOT a production threat database or a repair recipe.
//! Lookup consumes the full digest already obtained by the streaming reader.

use crate::FileHashes;

pub const VERSION: &str = "g1-eicar-only-1";
/// SHA-256 of the canonical 68-byte, uncompressed EICAR anti-malware test file.
pub const EICAR_SHA256: &str = "275a021bbfb6489e54d471899f7db9d1663fc695ec2fe2a2c4538aabf651fd0f";

/// No match for a missing/capped digest. A filename, a matching size or a
/// head/tail fingerprint can never substitute for byte identity in veto V5.
pub fn matches(hashes: &FileHashes) -> bool {
    hashes.full == EICAR_SHA256
}
