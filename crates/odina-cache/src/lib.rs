// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! Session-owned catalog and cache. No command parsing, exporters or sockets.
mod cache;
mod engine;
mod stamp;
use cache::Cache;
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
        let index = match cache {
            Some(dir) => Cache::open(dir, target, None)?,
            None => Cache::memory()?,
        };
        Ok(Self {
            filter: PrefixFilter::from_catalog(),
            index: Some(index),
        })
    }
    pub fn scan_file(&mut self, path: &Path, work: &mut Work) -> io::Result<(Assessment, File)> {
        engine::scan(path, &self.filter, &mut self.index, work)
    }
    /// Seal a full inventory, not a single watcher event. A future delta API
    /// must preserve unrelated rows and invalidate removed/renamed identities.
    pub fn finish_pass(&mut self) -> io::Result<()> {
        self.index.as_mut().map_or(Ok(()), Cache::finish_pass)
    }
}
