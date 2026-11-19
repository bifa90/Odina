// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! One exact test signature, not a production threat database. Both the cheap
//! rejection index and V5 are derived from the SAME catalog. The Bloom key is
//! observed content, not a path, extension, or an as-yet uncomputed SHA.

use crate::FileHashes;
use std::io::{self, Read};

pub const VERSION: &str = "g1-eicar-only-1";
pub const EICAR_SHA256: &str = "275a021bbfb6489e54d471899f7db9d1663fc695ec2fe2a2c4538aabf651fd0f";
const WIDTH: usize = 8;

struct Signature {
    sha256: &'static str,
    prefix: Option<[u8; WIDTH]>,
    len: Option<u64>,
}

const CATALOG: &[Signature] = &[Signature {
    sha256: EICAR_SHA256,
    prefix: Some(*b"X5O!P%@A"),
    len: Some(68),
}];

pub fn matches(hashes: &FileHashes) -> bool {
    !hashes.full.is_empty() && CATALOG.iter().any(|s| hashes.full == s.sha256)
}

pub struct PrefixFilter {
    words: Vec<u64>,
    complete: bool,
    lengths: Option<Vec<u64>>,
}

impl PrefixFilter {
    pub fn from_catalog() -> Self {
        Self::build(CATALOG)
    }

    fn build(entries: &[Signature]) -> Self {
        // Missing even one length disables metadata rejection. Otherwise the
        // per-file guard is logarithmic in distinct lengths, not a DB scan.
        let mut lengths: Option<Vec<_>> = entries.iter().map(|e| e.len).collect();
        if let Some(v) = &mut lengths {
            v.sort_unstable();
            v.dedup();
        }
        let mut index = Self {
            words: vec![0; entries.len().div_ceil(4).max(1)],
            complete: true,
            lengths,
        };
        for entry in entries {
            if let Some(key) = entry.prefix {
                for bit in index.positions(key) {
                    index.words[bit / 64] |= 1 << (bit % 64);
                }
            } else {
                // A digest-only update must never create false negatives.
                index.complete = false;
            }
        }
        index
    }

    fn positions(&self, key: [u8; WIDTH]) -> [usize; 4] {
        // SplitMix64 finalizer on the observed eight bytes; double hashing.
        let mut h = u64::from_le_bytes(key);
        h = (h ^ (h >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        h = (h ^ (h >> 27)).wrapping_mul(0x94d049bb133111eb);
        h ^= h >> 31;
        let step = (h >> 32) | 1;
        let bits = self.words.len() * 64;
        std::array::from_fn(|i| h.wrapping_add((i as u64).wrapping_mul(step)) as usize % bits)
    }

    /// A matching cache stamp, even if stale, cannot hide a possible V5 length.
    pub fn excludes_length(&self, len: u64) -> bool {
        self.lengths
            .as_ref()
            .is_some_and(|lengths| lengths.binary_search(&len).is_err())
    }

    fn maybe_matches(&self, bytes: &[u8]) -> bool {
        !self.complete
            || bytes.try_into().is_ok_and(|key| {
                self.positions(key)
                    .iter()
                    .all(|&bit| self.words[bit / 64] & (1 << (bit % 64)) != 0)
            })
    }

    /// `None` means no content digests were needed, NOT that the file is safe.
    /// The caller still runs all non-content indicators. Non-cached passes
    /// read to EOF even on negatives: do not turn a late I/O error into Clean
    /// or trust metadata as a byte count for virtual/remote files.
    pub fn check_reader<R: Read>(
        &self,
        mut input: R,
        cap: u64,
        needs_report_hashes: bool,
    ) -> io::Result<(Option<FileHashes>, u64)> {
        let mut prefix = [0; WIDTH];
        let mut n = 0;
        while n < WIDTH {
            match input.read(&mut prefix[n..]) {
                Ok(0) => break,
                Ok(count) => n += count,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            }
        }
        let maybe = self.maybe_matches(&prefix[..n]);
        if !maybe && !needs_report_hashes {
            return Ok((None, n as u64 + io::copy(&mut input, &mut io::sink())?));
        }
        // Replay precisely the bytes probed, without reopening or rewinding.
        let hashes = crate::hashing::hash_reader(prefix[..n].chain(input), Some(cap))?;
        let size = hashes.size;
        Ok((Some(hashes), size))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hashing::hash_bytes;

    #[test]
    fn a_possible_prefix_requires_the_exact_digest_and_even_a_bloom_collision_is_not_evidence() {
        let index = PrefixFilter::from_catalog();
        let input = b"X5O!P%@A harmless";
        let (some, n) = index.check_reader(&input[..], 512, false).unwrap();
        let hashes = some.unwrap();
        assert_eq!(n, input.len() as u64);
        assert_eq!(hashes, hash_bytes(input));
        assert!(!matches(&hashes));
        assert!(!index.excludes_length(68));
        let saturated = PrefixFilter {
            words: vec![u64::MAX],
            complete: true,
            lengths: None,
        };
        let (some, _) = saturated
            .check_reader(&b"12345678"[..], 512, false)
            .unwrap();
        assert!(!matches(&some.unwrap()));
    }

    #[test]
    fn missing_catalog_metadata_or_a_required_report_falls_back() {
        let unknown = PrefixFilter::build(&[Signature {
            prefix: None,
            len: None,
            sha256: EICAR_SHA256,
        }]);
        let index = PrefixFilter::from_catalog();
        assert!(!unknown.excludes_length(512));
        for s in [&b""[..], b"abc", b"ordinary contents"] {
            assert_eq!(
                index.check_reader(s, 100, false).unwrap(),
                (None, s.len() as u64)
            );
            assert_eq!(
                index.check_reader(s, 100, true).unwrap().0.unwrap(),
                hash_bytes(s)
            );
            assert_eq!(
                unknown.check_reader(s, 100, false).unwrap().0.unwrap(),
                hash_bytes(s)
            );
        }
        assert!(index
            .check_reader(&b"X5O!P%@A"[..], 3, false)
            .unwrap()
            .0
            .unwrap()
            .full_digest_skipped());
    }

    #[test]
    fn short_reads_do_not_truncate_the_probe_and_late_read_errors_propagate() {
        struct OneByte<'a>(&'a [u8]);
        impl Read for OneByte<'_> {
            fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
                self.0.read(&mut out[..1])
            }
        }
        let index = PrefixFilter::from_catalog();
        let data = b"X5O!P%@A suffix";
        assert_eq!(
            index
                .check_reader(OneByte(data), 512, false)
                .unwrap()
                .0
                .unwrap(),
            hash_bytes(data)
        );
        struct Broken;
        impl Read for Broken {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::other("unreadable sector"))
            }
        }
        assert!(index
            .check_reader(b"negative prefix".as_slice().chain(Broken), 512, false)
            .is_err());
    }
}
