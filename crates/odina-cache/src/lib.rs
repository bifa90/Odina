// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! Session-owned catalog and cache. No command parsing, exporters or sockets.
mod cache;
mod engine;
mod stamp;
use cache::Cache;
use engine::ReadMode;
pub use engine::Work;
use odina_core::{assessment::Assessment, g1_reference::PrefixFilter};
use std::{fs::File, io, path::Path};

pub const FULL_DIGEST_CAP: u64 = 512 * 1024 * 1024;

pub struct Scanner {
    filter: PrefixFilter,
    index: Option<Cache>,
}
impl Scanner {
    pub fn one_shot(target: &Path, out: &Path, cache: Option<&Path>) -> io::Result<Self> {
        Ok(Self {
            filter: PrefixFilter::from_catalog(),
            index: cache
                .map(|dir| Cache::open(dir, target, Some(out)))
                .transpose()?,
        })
    }
    pub fn resident(target: &Path, cache: Option<&Path>) -> io::Result<Self> {
        let mut index = match cache {
            Some(dir) => Cache::open(dir, target, None)?,
            None => Cache::memory()?,
        };
        index.track();
        Ok(Self {
            filter: PrefixFilter::from_catalog(),
            index: Some(index),
        })
    }
    pub fn scan_file(&mut self, path: &Path, work: &mut Work) -> io::Result<(Assessment, File)> {
        engine::scan(
            path,
            &self.filter,
            &mut self.index,
            work,
            ReadMode::Inventory,
        )
    }
    pub fn begin_pass(&mut self) {
        if let Some(c) = &mut self.index {
            c.begin_pass();
        }
    }
    pub fn forget(&mut self, path: &Path) {
        if let Some(c) = &mut self.index {
            c.forget(path);
        }
    }
    /// A native notice forces a fresh stream, with handle/name identity checks even on a cache miss.
    /// No full-pass rotation or disk index write for a delta.
    pub fn scan_changed(&mut self, path: &Path) -> io::Result<Assessment> {
        self.forget(path);
        engine::scan(
            path,
            &self.filter,
            &mut self.index,
            &mut Work::default(),
            ReadMode::Delta,
        )
        .map(|(a, _)| a)
    }
    /// Opt-in read of one stable regular file. Force Bloom + full SHA; do not read
    /// cached hints, update inventory, export a hash or run the file.
    pub fn verify_file(&self, path: &Path) -> io::Result<Assessment> {
        engine::scan(
            path,
            &self.filter,
            &mut None,
            &mut Work::default(),
            ReadMode::Explicit,
        )
        .map(|(a, _)| a)
    }
    /// Seal only a full inventory.
    pub fn finish_pass(&mut self) -> io::Result<()> {
        self.index.as_mut().map_or(Ok(()), Cache::finish_pass)
    }
}

/// The public pass loop scores through this trait (`odina_scan::Read`). The
/// cache's per-file counters are a measurement, so they are created and
/// dropped here rather than leaking into the loop that must stay unmeasured.
impl odina_scan::Read for Scanner {
    fn score(&mut self, path: &Path) -> io::Result<Assessment> {
        self.scan_file(path, &mut Work::default())
            .map(|(assessment, _handle)| assessment)
    }
}
