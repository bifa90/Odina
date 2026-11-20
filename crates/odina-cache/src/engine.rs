// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
use super::{
    cache::Cache,
    stamp::{stamp, state},
    FULL_DIGEST_CAP,
};
use odina_core::assessment::{assess, Assessment};
use odina_core::{g1_reference::PrefixFilter, FileHashes};
use std::{
    fs::{File, OpenOptions},
    io::{self, Read},
    path::Path,
};

/// Completed decision-stream work, including completed passes retried due to
/// concurrent writes. No names, identities, actual disk-I/O or CPU estimates.
#[derive(Debug, Default)]
pub struct Work {
    pub cache_hits: u64,
    pub bloom_negatives: u64,
    pub sha_files: u64,
    pub streamed_bytes: u64,
}

fn open_regular(path: &Path) -> io::Result<File> {
    let mut opts = OpenOptions::new();
    opts.read(true);
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        opts.custom_flags(0x0020_0000);
    } // OPEN_REPARSE_POINT
    let file = opts.open(path)?;
    let m = file.metadata()?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if m.file_attributes() & 0x400 != 0 {
            return Err(io::Error::other("reparse point"));
        }
    }
    if !m.is_file() {
        return Err(io::Error::other("not a regular data file"));
    }
    Ok(file)
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum ReadMode {
    Inventory,
    Delta,
    Explicit,
}
pub(super) fn scan(
    path: &Path,
    filter: &PrefixFilter,
    cache: &mut Option<Cache>,
    work: &mut Work,
    mode: ReadMode,
) -> io::Result<(Assessment, File)> {
    let delta = mode != ReadMode::Inventory;
    for _ in 0..2 {
        let mut file = open_regular(path)?; // even hits must still be openable
        let proof = if delta {
            Some(state(&file).ok_or_else(|| io::Error::other("no stable file identity"))?)
        } else {
            None
        };
        if mode == ReadMode::Explicit && proof.is_some_and(|s| s.len > FULL_DIGEST_CAP) {
            return Err(io::Error::other("explicit digest cap"));
        }
        let initial = cache.as_ref().and_then(|_| stamp(&file));
        let key = initial.and_then(|s| cache.as_ref().map(|c| c.key(path, s)));
        let mut preliminary = assess(path, FileHashes::default());
        // This is permitted ONLY while all decision evidence other than V5 is
        // freshly evaluated by assess(). Length exclusion, not the stored row,
        // proves V5 impossible. Candidate sizes always re-probe real bytes.
        if let (Some(s), Some(k), Some(c)) = (initial, key, cache.as_mut()) {
            if !delta
                && !preliminary.score.interrupts()
                && filter.excludes_length(s.len)
                && c.silent_hint(k)
            {
                if stamp(&file) != Some(s) {
                    continue;
                }
                preliminary.hashes.size = s.len;
                c.record(path, k, preliminary.score.verdict, None, false);
                work.cache_hits += 1;
                return Ok((preliminary, file));
            }
        }
        let (hashes, bytes) = if mode == ReadMode::Explicit {
            // Also bound a source that grows after fstat. A capped/partial digest
            // must never turn a consented full verification into a negative.
            let result = filter.check_reader(
                file.by_ref().take(FULL_DIGEST_CAP + 1),
                FULL_DIGEST_CAP,
                true,
            )?;
            if result
                .0
                .as_ref()
                .map_or(true, FileHashes::full_digest_skipped)
            {
                return Err(io::Error::other("incomplete explicit digest"));
            }
            result
        } else {
            filter.check_reader(&mut file, FULL_DIGEST_CAP, preliminary.score.interrupts())?
        };
        work.streamed_bytes += bytes;
        let report = if let Some(hashes) = hashes {
            work.sha_files += 1;
            assess(path, hashes)
        } else {
            work.bloom_negatives += 1;
            preliminary.hashes.size = bytes;
            preliminary
        };
        if let Some(id) = proof {
            if bytes != id.len
                || state(&file) != Some(id)
                || state(&open_regular(path)?) != Some(id)
            {
                continue;
            }
        }
        if let (Some(s), Some(k), Some(c)) = (initial, key, cache.as_mut()) {
            if stamp(&file) != Some(s) || bytes != s.len {
                continue;
            }
            c.record(path, k, report.score.verdict, Some(&report.hashes), delta);
        }
        return Ok((report, file));
    }
    Err(io::Error::other("file did not remain stable"))
}
#[cfg(test)]
mod explicit_tests {
    use super::*;
    #[test]
    fn explicit_negative_still_hashes_the_whole_file_and_never_uses_a_hint() {
        let path = std::env::temp_dir().join(format!("odina-verify-one-{}", std::process::id()));
        let f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        drop(f);
        let data = b"ordinary prefix, not a catalog candidate";
        std::fs::write(&path, data).unwrap();
        let mut w = Work::default();
        let (a, file) = scan(
            &path,
            &PrefixFilter::from_catalog(),
            &mut None,
            &mut w,
            ReadMode::Explicit,
        )
        .unwrap();
        assert_eq!(a.hashes, odina_core::hashing::hash_bytes(data));
        assert_eq!(
            (
                w.sha_files,
                w.bloom_negatives,
                w.cache_hits,
                w.streamed_bytes
            ),
            (1, 0, 0, data.len() as u64)
        );
        drop(file);
        std::fs::remove_file(path).unwrap();
    }
}
