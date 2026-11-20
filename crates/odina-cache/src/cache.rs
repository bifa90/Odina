// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! Private, opt-in index of completed passes. Rows are hints; this store has
//! NO API to obtain an authoritative verdict or digest. See engine::scan.
use super::{stamp::Stamp, FULL_DIGEST_CAP};
use odina_core::{g1_reference, FileHashes, Verdict};
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    fs::{self, File, OpenOptions},
    hash::{Hash, Hasher},
    io::{self, BufWriter, Read, Write},
    path::{Path, PathBuf},
};

const LIMIT: usize = 65536;
const MAX_BYTES: u64 = 8 * 1024 * 1024;
const FILE: &str = "index-v1.tsv";

#[derive(PartialEq, Eq)]
struct Row {
    verdict: String,
    sha: String,
}
type Rows = HashMap<u64, Row>;

pub(super) struct Cache {
    dir: Option<PathBuf>,
    cwd: PathBuf,
    // Resident-only reverse lookup for invalidation; these opaque tokens never leave memory.
    paths: Option<HashMap<u64, u64>>,
    disk_dirty: bool,
    previous: Rows,
    current: Rows,
}

fn header() -> String {
    format!(
        "odina-index/v1\t{}\t{}\t{FULL_DIGEST_CAP}\tname-path-v5/1",
        env!("CARGO_PKG_VERSION"),
        g1_reference::VERSION
    )
}
fn parse(text: &str) -> Option<Rows> {
    let mut lines = text.lines();
    if lines.next()? != header() {
        return None;
    }
    let mut rows = Rows::new();
    for line in lines {
        let mut cols = line.split('\t');
        let key = u64::from_str_radix(cols.next()?, 16).ok()?;
        let (verdict, sha) = (cols.next()?, cols.next()?);
        if cols.next().is_some()
            || rows.len() >= LIMIT
            || !matches!(verdict, "clean" | "monitor" | "alert")
            || !(sha == "-" || (sha.len() == 64 && sha.bytes().all(|c| c.is_ascii_hexdigit())))
        {
            return None;
        }
        if rows
            .insert(
                key,
                Row {
                    verdict: verdict.into(),
                    sha: sha.into(),
                },
            )
            .is_some()
        {
            return None;
        }
    }
    Some(rows)
}
fn load(path: &Path) -> Option<Rows> {
    let m = fs::symlink_metadata(path).ok()?;
    if !m.is_file() || m.len() > MAX_BYTES {
        return None;
    }
    let mut text = String::new();
    File::open(path)
        .ok()?
        .take(MAX_BYTES + 1)
        .read_to_string(&mut text)
        .ok()?;
    if text.len() as u64 > MAX_BYTES {
        return None;
    }
    parse(&text)
}
fn invalid(msg: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, msg)
}

impl Cache {
    pub(super) fn memory() -> io::Result<Self> {
        Ok(Self {
            paths: None,
            disk_dirty: false,
            dir: None,
            cwd: std::env::current_dir()?,
            previous: Rows::new(),
            current: Rows::new(),
        })
    }
    pub(super) fn open(dir: &Path, target: &Path, out: Option<&Path>) -> io::Result<Self> {
        // Resolve existing parents before creating anything; never put even an
        // empty directory in the corpus, and never export the index as JSON.
        let cwd = std::env::current_dir()?;
        let named = |p: &Path| -> io::Result<PathBuf> {
            let p = cwd.join(p);
            Ok(p.parent()
                .ok_or_else(|| invalid("expected a named path"))?
                .canonicalize()?
                .join(
                    p.file_name()
                        .ok_or_else(|| invalid("expected a final name"))?,
                ))
        };
        let dir = named(dir)?;
        let dir = match fs::symlink_metadata(&dir) {
            Ok(m) if !m.is_dir() => return Err(invalid("cache cannot be a file or link")),
            Ok(m) => {
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    if m.file_attributes() & 0x400 != 0 {
                        return Err(invalid("cache cannot be a reparse point"));
                    }
                }
                let _ = m;
                dir.canonicalize()?
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => dir,
            Err(e) => return Err(e),
        };
        let target = target.canonicalize()?;
        if dir.starts_with(&target) || target.starts_with(&dir) {
            return Err(invalid("cache must be separate from the input"));
        }
        if let Some(out) = out {
            let out = match fs::symlink_metadata(out) {
                Ok(m) if !m.is_file() => {
                    return Err(invalid("--out cannot be a directory or link"))
                }
                Ok(_) => out.canonicalize()?,
                Err(e) if e.kind() == io::ErrorKind::NotFound => named(out)?,
                Err(e) => return Err(e),
            };
            if out.starts_with(&dir) || out.starts_with(&target) {
                return Err(invalid("--out must be outside cache and target"));
            }
        }
        let create = &mut fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            create.mode(0o700);
        }
        match create.create(&dir) {
            Err(e) if e.kind() != io::ErrorKind::AlreadyExists => return Err(e),
            _ => (),
        }
        let meta = fs::symlink_metadata(&dir)?;
        if !meta.is_dir() {
            return Err(invalid("cache must be a private directory, not a link"));
        }
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::fs::MetadataExt;
            // SAFETY: geteuid takes no arguments.
            if meta.uid() != unsafe { libc::geteuid() } || meta.mode() & 0o077 != 0 {
                return Err(invalid(
                    "cache directory must be owned by the caller with mode 0700",
                ));
            }
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if meta.file_attributes() & 0x400 != 0 {
                return Err(invalid("cache cannot be a reparse point"));
            }
        }
        let previous = load(&dir.join(FILE)).unwrap_or_default();
        Ok(Self {
            paths: None,
            disk_dirty: false,
            dir: Some(dir),
            cwd,
            previous,
            current: Rows::new(),
        })
    }

    pub(super) fn track(&mut self) {
        self.paths = Some(HashMap::new());
    }
    pub(super) fn begin_pass(&mut self) {
        if let Some(paths) = &mut self.paths {
            paths.clear();
        }
    }
    fn token(path: &Path) -> u64 {
        let mut h = DefaultHasher::new();
        path.hash(&mut h);
        h.finish()
    }
    pub(super) fn forget(&mut self, path: &Path) {
        if let Some(key) = self
            .paths
            .as_mut()
            .and_then(|p| p.remove(&Self::token(path)))
        {
            self.previous.remove(&key);
            self.disk_dirty = true;
        }
    }
    pub(super) fn key(&self, path: &Path, stamp: Stamp) -> u64 {
        // Lossless native path (including non-UTF8 names), actual handle ID,
        // length, mtime AND change time. Opaque lookup token, not a MAC.
        let mut h = DefaultHasher::new();
        if path.is_relative() {
            self.cwd.hash(&mut h);
        }
        path.hash(&mut h);
        stamp.hash(&mut h);
        h.finish()
    }
    pub(super) fn silent_hint(&self, key: u64) -> bool {
        self.previous
            .get(&key)
            .is_some_and(|r| r.verdict != "alert")
    }
    pub(super) fn record(
        &mut self,
        path: &Path,
        key: u64,
        verdict: Verdict,
        streamed: Option<&FileHashes>,
        delta: bool,
    ) {
        let count = if delta {
            self.previous.len()
        } else {
            self.current.len()
        };
        if count >= LIMIT {
            return;
        }
        let verdict = match verdict {
            Verdict::Clean => "clean",
            Verdict::Monitor => "monitor",
            _ => "alert",
        };
        // Keep the last observed digest on a no-read hit, for local history
        // only. It is not a measurement of this pass and never reaches V5.
        let sha = match streamed {
            Some(h) if !h.full.is_empty() => h.full.as_str(),
            Some(_) => "-",
            None => self.previous.get(&key).map_or("-", |r| r.sha.as_str()),
        };
        let row = Row {
            verdict: verdict.into(),
            sha: sha.into(),
        };
        if let Some(paths) = &mut self.paths {
            paths.insert(Self::token(path), key);
        }
        if delta {
            self.previous.insert(key, row);
            self.disk_dirty = true;
        } else {
            self.current.insert(key, row);
        }
    }
    pub(super) fn finish_pass(&mut self) -> io::Result<()> {
        // Rows describe one completed inventory. Do not reload or rebuild a
        // resident index between requests; never expose a row as a verdict.
        let published = match &self.dir {
            Some(dir) if self.disk_dirty || self.current != self.previous => self.publish(dir),
            _ => Ok(()),
        };
        if published.is_ok() {
            self.disk_dirty = false;
            std::mem::swap(&mut self.previous, &mut self.current);
        }
        self.current.clear();
        published
    }
    fn publish(&self, dir: &Path) -> io::Result<()> {
        let tmp = dir.join(format!(".index-{}.tmp", std::process::id()));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = options.open(&tmp)?;
        let publish = || -> io::Result<()> {
            let mut out = BufWriter::new(file);
            writeln!(out, "{}", header())?;
            for (key, row) in &self.current {
                writeln!(out, "{key:016x}\t{}\t{}", row.verdict, row.sha)?;
            }
            out.flush()?;
            out.get_ref().sync_all()?;
            drop(out);
            fs::rename(&tmp, dir.join(FILE))
        };
        let result = publish();
        if result.is_err() {
            let _ = fs::remove_file(&tmp);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn delta_keeps_unrelated_hints_and_removal_drops_the_changed_identity() {
        let mut c = Cache::memory().unwrap();
        c.track();
        let (a, b) = (Path::new("a"), Path::new("b"));
        c.begin_pass();
        c.record(a, 1, Verdict::Clean, None, false);
        c.record(b, 2, Verdict::Clean, None, false);
        c.finish_pass().unwrap();
        assert_eq!(c.previous.len(), 2);
        c.forget(a);
        c.record(a, 3, Verdict::Confirmed, None, true);
        assert!(!c.previous.contains_key(&1));
        assert!(c.previous.contains_key(&3));
        assert!(c.silent_hint(2));
        c.forget(a);
        assert_eq!(c.previous.len(), 1);
        assert!(c.silent_hint(2));
        assert!(c.current.is_empty());
    }
    #[test]
    fn unknown_version_partial_duplicate_or_oversized_field_is_a_cold_miss() {
        for bad in [
            "old\n0\tclean\t-".into(),
            format!("{}\n0\tclean", header()),
            format!("{}\n0\talert\tzzz", header()),
            format!("{}\n0\tclean\t-\n0\tclean\t-", header()),
        ] {
            assert!(parse(&bad).is_none());
        }
    }
}
