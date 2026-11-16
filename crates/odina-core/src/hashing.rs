// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! The five hashes the scoring model needs, computed from the bytes only.
//!
//! `docs/spec/scoring-spec-v1.md` §2 asks for a whole-file digest plus a head
//! and tail digest (indicators 5 and 6, 23% of the weight together). This
//! module produces them without reading the file twice and without allocating
//! the whole file: a 4 GB ISO costs 8 KB of memory here.

use sha2::{Digest, Sha256};

/// Bytes taken from the start of the file for `header_hash`.
pub const HEAD_BYTES: usize = 4096;
/// Bytes taken from the end of the file for `tail_hash`.
pub const TAIL_BYTES: usize = 4096;

/// The hash set for one file. Every field is a lowercase hex SHA-256.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileHashes {
    /// Digest of the entire file. Empty for files above the size limit.
    pub full: String,
    /// Digest of the first [`HEAD_BYTES`] bytes.
    pub head: String,
    /// Digest of the last [`TAIL_BYTES`] bytes.
    pub tail: String,
    /// Size in bytes, as counted while reading — never trusted from metadata.
    pub size: u64,
}

impl FileHashes {
    /// True when a head and tail were produced but the full digest was skipped
    /// because the file exceeded the caller's limit.
    pub fn full_digest_skipped(&self) -> bool {
        self.full.is_empty()
    }
}

fn hex(bytes: &[u8]) -> String {
    // Written by hand instead of pulling `hex` in: one dependency less to
    // audit, and the output format is fixed by the spec.
    const TABLE: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(TABLE[(b >> 4) as usize] as char);
        out.push(TABLE[(b & 0x0f) as usize] as char);
    }
    out
}

/// Hash bytes already in memory. Used for tests and for small artefacts.
pub fn hash_bytes(data: &[u8]) -> FileHashes {
    let head_len = data.len().min(HEAD_BYTES);
    let tail_start = data.len().saturating_sub(TAIL_BYTES);
    FileHashes {
        full: hex(&Sha256::digest(data)),
        head: hex(&Sha256::digest(&data[..head_len])),
        tail: hex(&Sha256::digest(&data[tail_start..])),
        size: data.len() as u64,
    }
}

/// Hash a reader without knowing its length in advance.
///
/// `max_full_digest` caps how much is fed into the whole-file digest. Above
/// it, `full` is left empty and only head/tail are returned — the caller is
/// expected to fall back to the head/tail indicators, which is exactly what
/// the spec's cap S1 assumes for very large files.
pub fn hash_reader<R: std::io::Read>(
    mut reader: R,
    max_full_digest: Option<u64>,
) -> std::io::Result<FileHashes> {
    let mut full = Sha256::new();
    let mut head = Sha256::new();
    let mut head_filled = 0usize;

    // A sliding window keeps only the last TAIL_BYTES seen so far, so the
    // whole file never has to be buffered.
    let mut tail_buf: Vec<u8> = Vec::with_capacity(TAIL_BYTES);
    let mut size: u64 = 0;

    let mut chunk = [0u8; 64 * 1024];
    let limit = max_full_digest.unwrap_or(u64::MAX);
    let mut full_enabled = true;

    loop {
        let n = reader.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        let slice = &chunk[..n];
        size += n as u64;

        if head_filled < HEAD_BYTES {
            let take = (HEAD_BYTES - head_filled).min(n);
            head.update(&slice[..take]);
            head_filled += take;
        }

        if full_enabled {
            if size > limit {
                full_enabled = false;
            } else {
                full.update(slice);
            }
        }

        if slice.len() >= TAIL_BYTES {
            tail_buf.clear();
            tail_buf.extend_from_slice(&slice[slice.len() - TAIL_BYTES..]);
        } else {
            let overflow = (tail_buf.len() + slice.len()).saturating_sub(TAIL_BYTES);
            if overflow > 0 {
                tail_buf.drain(..overflow);
            }
            tail_buf.extend_from_slice(slice);
        }
    }

    Ok(FileHashes {
        full: if full_enabled {
            hex(&full.finalize())
        } else {
            String::new()
        },
        head: hex(&head.finalize()),
        tail: hex(&Sha256::digest(&tail_buf)),
        size,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_is_stable() {
        let h = hash_bytes(b"");
        assert_eq!(h.size, 0);
        assert_eq!(
            h.head, h.tail,
            "empty file: head and tail are the same bytes"
        );
        assert_eq!(
            h.head,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn known_vector() {
        // SHA-256("abc")
        let h = hash_bytes(b"abc");
        assert_eq!(
            h.full,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(h.size, 3);
    }

    #[test]
    fn head_and_tail_match_manual_slice_for_big_file() {
        let data: Vec<u8> = (0..(HEAD_BYTES * 3)).map(|i| (i % 251) as u8).collect();
        let h = hash_bytes(&data);
        let head_manual = hex(&Sha256::digest(&data[..HEAD_BYTES]));
        let tail_manual = hex(&Sha256::digest(&data[data.len() - TAIL_BYTES..]));
        assert_eq!(h.head, head_manual);
        assert_eq!(h.tail, tail_manual);
    }

    #[test]
    fn reader_matches_in_memory() {
        let data: Vec<u8> = (0..100_000u32).map(|i| (i % 256) as u8).collect();
        let from_mem = hash_bytes(&data);
        let from_reader = hash_reader(&data[..], None).unwrap();
        assert_eq!(from_mem, from_reader);
    }

    #[test]
    fn reader_can_skip_the_full_digest() {
        let data = vec![7u8; 10_000];
        let h = hash_reader(&data[..], Some(1_000)).unwrap();
        assert!(h.full_digest_skipped(), "above the cap, full must be empty");
        assert_eq!(h.size, 10_000);
        assert!(!h.head.is_empty() && !h.tail.is_empty());
    }

    #[test]
    fn tail_window_survives_odd_chunking() {
        // The sliding window must not depend on read chunk boundaries.
        let data: Vec<u8> = (0..20_000u32).map(|i| (i % 253) as u8).collect();
        let expected = hex(&Sha256::digest(&data[data.len() - TAIL_BYTES..]));
        struct Trickle<'a>(&'a [u8]);
        impl std::io::Read for Trickle<'_> {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                let n = buf.len().min(7).min(self.0.len());
                buf[..n].copy_from_slice(&self.0[..n]);
                self.0 = &self.0[n..];
                Ok(n)
            }
        }
        let h = hash_reader(Trickle(&data), None).unwrap();
        assert_eq!(h.tail, expected);
        assert_eq!(h.size, 20_000);
    }
}
