// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
use super::{cache::Cache, stamp::stamp, FULL_DIGEST_CAP};
use odina_core::assessment::{assess, Assessment};
use odina_core::{g1_reference::PrefixFilter, FileHashes};
use std::{
    fs::{File, OpenOptions},
    io,
    path::Path,
};

/// Completed decision-stream work, including completed passes retried due to
/// concurrent writes. No names, identities, actual disk-I/O or CPU estimates.
#[derive(Default)]
pub struct Work {
    hits: u64,
    negatives: u64,
    sha: u64,
    bytes: u64,
}
impl Work {
    pub fn json(&self) -> String {
        format!(
            "{{\"cache_hits\":{},\"bloom_negatives\":{},\"sha_files\":{},\"streamed_bytes\":{}}}",
            self.hits, self.negatives, self.sha, self.bytes
        )
    }
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

pub(super) fn scan(
    path: &Path,
    filter: &PrefixFilter,
    cache: &mut Option<Cache>,
    work: &mut Work,
) -> io::Result<(Assessment, File)> {
    for _ in 0..2 {
        let mut file = open_regular(path)?; // even hits must still be openable
        let initial = cache.as_ref().and_then(|_| stamp(&file));
        let key = initial.and_then(|s| cache.as_ref().map(|c| c.key(path, s)));
        let mut preliminary = assess(path, FileHashes::default());
        // This is permitted ONLY while all decision evidence other than V5 is
        // freshly evaluated by assess(). Length exclusion, not the stored row,
        // proves V5 impossible. Candidate sizes always re-probe real bytes.
        if let (Some(s), Some(k), Some(c)) = (initial, key, cache.as_mut()) {
            if !preliminary.score.interrupts() && filter.excludes_length(s.len) && c.silent_hint(k)
            {
                if stamp(&file) != Some(s) {
                    continue;
                }
                preliminary.hashes.size = s.len;
                c.record(k, preliminary.score.verdict, None);
                work.hits += 1;
                return Ok((preliminary, file));
            }
        }
        let (hashes, bytes) =
            filter.check_reader(&mut file, FULL_DIGEST_CAP, preliminary.score.interrupts())?;
        work.bytes += bytes;
        let report = if let Some(hashes) = hashes {
            work.sha += 1;
            assess(path, hashes)
        } else {
            work.negatives += 1;
            preliminary.hashes.size = bytes;
            preliminary
        };
        if let (Some(s), Some(k), Some(c)) = (initial, key, cache.as_mut()) {
            if stamp(&file) != Some(s) || bytes != s.len {
                continue;
            }
            c.record(k, report.score.verdict, Some(&report.hashes));
        }
        return Ok((report, file));
    }
    Err(io::Error::other("file did not remain stable"))
}
