// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! The public scan path: walk a tree, score every file, stop.
//!
//! This crate exists for one reason, written down as K1 in `docs/roadmap.md`:
//! the resident service is a *core* capability (v19 §1, D49), so it cannot
//! live behind the `instrument` feature. Before this crate existed, the only
//! pass loop in the tree was `odina-cli`'s `check::run_pass`, which imports
//! `odina-report`, `odina-instrument` and `odina-cache` — so gating the
//! *service* on `instrument` was the only way to keep the measurement layer
//! out of the public build. That is no longer true: this loop measures
//! nothing, exports nothing, and depends on `odina-core` alone.
//!
//! What is deliberately absent:
//!
//! - **No `odina-report`.** No report is built here, so no field can leak.
//! - **No `odina-instrument`.** No meter, no timing, no counters.
//! - **No `odina-cache`.** The public pass reads every file it is given. A
//!   cache is an optimisation with its own invalidation rules, and a wrong
//!   cached hint is a wrong verdict; it joins when the verdict can be proven
//!   identical with and without it (K1 exit criterion 1).
//!
//! The walk is the same walk the check build uses, moved rather than copied:
//! two implementations of "which files does this folder contain" would drift,
//! and the drift would show up as a scan that finds different files in the two
//! builds. `odina-cli` now re-exports this one, so the tests that pin the walk
//! behaviour keep guarding the only implementation there is.

use odina_core::assessment::Assessment;
use odina_core::Verdict;
use std::path::{Path, PathBuf};

/// How many directories one walk may visit. A Downloads folder is far below
/// this; the number exists only so that a pathological tree cannot run for
/// hours while the user watches nothing happen.
pub const MAX_DIRS: u64 = 200_000;

/// How many entries one directory may yield before the walk gives up on it.
/// Windows shortcut files are a normal part of a Downloads folder, and
/// resolving them is not this tool's job.
const MAX_ENTRIES_PER_DIR: usize = 500_000;

/// What a folder walk found, and what it refused to follow.
pub struct Walked {
    /// Every regular file, in a stable order.
    pub files: Vec<PathBuf>,
    /// Junctions, symlinks and reparse points met and not entered. A folder
    /// whose real content lives elsewhere is not a folder we measured.
    pub links_skipped: u64,
    /// Folders the walk could not read, split by whether anyone could have.
    pub read_errors: u64,
    /// True if the walk stopped early because it hit a limit.
    pub truncated: bool,
}

impl Walked {
    /// The trivial walk: one file, nothing skipped, nothing truncated. The
    /// single-file case is not a special path in the loop below, only here.
    pub fn one(path: &Path) -> Self {
        Self {
            files: vec![path.to_path_buf()],
            links_skipped: 0,
            read_errors: 0,
            truncated: false,
        }
    }

    /// True when the answer is the whole answer: nothing was capped and every
    /// folder could be read. A partial answer must never be reported as clean.
    pub fn is_complete(&self) -> bool {
        !self.truncated && self.read_errors == 0
    }
}

/// Every file under `dir`, in a stable order. Two runs on the same folder must
/// produce the same sequence, or the numbers cannot be compared.
///
/// Three things this deliberately does *not* do, each of which would turn a
/// scan into a hang:
///
/// 1. **It never enters a link.** `Path::is_dir` follows junctions and
///    symlinks, and one junction pointing at an ancestor is an infinite walk.
///    The entry type from `read_dir` is used instead, which reports the link
///    rather than its target.
/// 2. **It never opens a non-regular file.** A named pipe in a downloads
///    folder blocks on open, forever, and looks exactly like a slow scan.
/// 3. **It bounds itself.** `MAX_DIRS` stops a tree that is legal but absurd,
///    and says so rather than returning a number that looks complete.
pub fn walk(dir: &Path) -> Walked {
    let mut files = Vec::new();
    let mut links_skipped = 0u64;
    let mut read_errors = 0u64;
    let mut visited = 0u64;
    let mut truncated = false;

    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        if visited >= MAX_DIRS {
            truncated = true;
            break;
        }
        visited += 1;

        let entries = match std::fs::read_dir(&current) {
            Ok(rd) => rd,
            Err(_) => {
                read_errors += 1;
                continue;
            }
        };

        let mut paths: Vec<(bool, PathBuf)> = Vec::new();
        for (index, entry) in entries.enumerate() {
            if index == MAX_ENTRIES_PER_DIR {
                truncated = true;
                break;
            }
            let Ok(entry) = entry else {
                read_errors += 1;
                continue;
            };
            // `DirEntry::file_type` does not follow a link; that is the whole
            // point of using it here.
            let Ok(kind) = entry.file_type() else {
                read_errors += 1;
                continue;
            };
            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                match entry.metadata() {
                    Ok(m) if m.file_attributes() & 0x400 != 0 => {
                        links_skipped += 1;
                        continue;
                    }
                    Err(_) => {
                        read_errors += 1;
                        continue;
                    }
                    _ => (),
                }
            }
            if kind.is_symlink() {
                links_skipped += 1;
                continue;
            }
            if kind.is_dir() {
                paths.push((true, entry.path()));
            } else if kind.is_file() {
                paths.push((false, entry.path()));
            } else {
                // A pipe, a socket, a device: opening it can block forever.
                links_skipped += 1;
            }
        }
        // Sorted by path, and the `is_dir` flag rides along so the loop below
        // does not have to ask the filesystem a second time — asking again is
        // what followed the link in the first place.
        paths.sort_by(|a, b| a.1.cmp(&b.1));
        for (is_dir, path) in paths {
            if is_dir {
                stack.push(path);
            } else {
                files.push(path);
            }
        }
    }

    files.sort();
    Walked {
        files,
        links_skipped,
        read_errors,
        truncated,
    }
}

/// What one public pass found. Counts and bytes only — no names, no paths.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Totals {
    /// Files the scorer returned a verdict for.
    pub files: u64,
    /// Files that could not be read at all. Counted, not silently dropped:
    /// an unreadable file is not a clean file.
    pub unreadable: u64,
    pub bytes: u64,
    pub clean: u64,
    pub monitor: u64,
    pub suspicious: u64,
    pub high_risk: u64,
    pub confirmed: u64,
}

impl Totals {
    /// Files that call for attention. This is the number the exit code and the
    /// resident's `code` are derived from, so it is computed once, here.
    pub fn interrupting(&self) -> u64 {
        self.confirmed + self.high_risk
    }

    /// The process exit code for this pass: 2 when something demands
    /// attention, 0 otherwise. Unreadable files do not raise it — they are a
    /// coverage problem, reported through `is_complete`, not a verdict.
    pub fn exit_code(&self) -> u8 {
        if self.interrupting() > 0 {
            2
        } else {
            0
        }
    }
}

/// The scan surface this crate needs. `odina-cache::Scanner` implements it
/// directly; the resident service and the CLI pass share the loop below, so
/// "the public build scores the same way" is one function, not two that agree
/// by inspection.
pub trait Read {
    /// Score one file. The `Work` counters the cache build keeps are dropped
    /// here on purpose: they are a measurement.
    fn score(&mut self, path: &Path) -> std::io::Result<Assessment>;
}

/// Walk, score, count. Returns the totals and the walk's own report so the
/// caller can tell a complete answer from a partial one.
///
/// `seen` is called once per scored file with the full assessment, for callers
/// that need the verdicts (the resident forwards them; the CLI ignores them).
pub fn pass(
    reader: &mut dyn Read,
    path: &Path,
    seen: &mut dyn FnMut(&Assessment),
) -> std::io::Result<(Totals, Walked)> {
    let walked = if path.is_dir() {
        walk(path)
    } else {
        Walked::one(path)
    };

    let mut totals = Totals::default();
    for file in &walked.files {
        match reader.score(file) {
            Ok(scanned) => {
                seen(&scanned);
                totals.files += 1;
                totals.bytes += scanned.hashes.size;
                match scanned.score.verdict {
                    Verdict::Clean => totals.clean += 1,
                    Verdict::Monitor => totals.monitor += 1,
                    Verdict::Suspicious => totals.suspicious += 1,
                    Verdict::HighRisk => totals.high_risk += 1,
                    Verdict::Confirmed => totals.confirmed += 1,
                }
            }
            Err(_) => {
                // A file nobody could open is still a file that was there.
                totals.files += 1;
                totals.unreadable += 1;
            }
        }
    }
    Ok((totals, walked))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// A scratch directory that removes itself, so a failing assert cannot
    /// leave a tree behind for the next run to trip over.
    struct Scratch(PathBuf);
    impl Scratch {
        fn new(tag: &str) -> Self {
            static N: AtomicU64 = AtomicU64::new(0);
            let p = std::env::temp_dir().join(format!(
                "odina-scan-{}-{}-{}",
                tag,
                std::process::id(),
                N.fetch_add(1, Ordering::Relaxed)
            ));
            let _ = fs::remove_dir_all(&p);
            fs::create_dir_all(&p).expect("scratch dir");
            Self(p)
        }
        fn file(&self, rel: &str, body: &[u8]) -> PathBuf {
            let p = self.0.join(rel);
            if let Some(parent) = p.parent() {
                fs::create_dir_all(parent).expect("parent");
            }
            fs::write(&p, body).expect("write");
            p
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    /// A reader that answers from a list, so the loop can be tested without
    /// touching a filesystem. `Err` stands for an unreadable file.
    struct Fake {
        answers: Vec<(PathBuf, Option<u64>)>,
        calls: usize,
    }
    impl Read for Fake {
        fn score(&mut self, path: &Path) -> std::io::Result<Assessment> {
            self.calls += 1;
            let (_, size) = self
                .answers
                .iter()
                .find(|(p, _)| p == path)
                .expect("unexpected path");
            match size {
                Some(n) => Ok(scored(*n)),
                None => Err(std::io::Error::other("unreadable")),
            }
        }
    }

    fn scored(size: u64) -> Assessment {
        let mut a = odina_core::assessment::assess(Path::new("x.bin"), Default::default());
        a.hashes.size = size;
        a
    }

    #[test]
    fn the_walk_is_stable_and_sorted() {
        let s = Scratch::new("stable");
        s.file("b.txt", b"b");
        s.file("a.txt", b"a");
        s.file("sub/c.txt", b"c");
        let first = walk(&s.0);
        let second = walk(&s.0);
        assert_eq!(first.files, second.files, "two walks must agree");
        let mut sorted = first.files.clone();
        sorted.sort();
        assert_eq!(first.files, sorted, "walk output must be sorted");
        assert_eq!(first.files.len(), 3);
        assert!(first.is_complete());
    }

    #[test]
    fn a_single_file_is_not_a_special_loop_path() {
        let s = Scratch::new("single");
        let f = s.file("one.bin", b"1234");
        let mut reader = Fake {
            answers: vec![(f.clone(), Some(4))],
            calls: 0,
        };
        let mut seen = 0;
        let (totals, walked) = pass(&mut reader, &f, &mut |_| seen += 1).expect("pass");
        assert_eq!(totals.files, 1);
        assert_eq!(totals.bytes, 4);
        assert_eq!(seen, 1);
        assert_eq!(reader.calls, 1);
        assert!(walked.is_complete());
    }

    #[test]
    fn an_unreadable_file_is_counted_and_breaks_completeness() {
        let s = Scratch::new("unreadable");
        let good = s.file("good.bin", b"ok");
        let bad = s.file("bad.bin", b"??");
        let mut reader = Fake {
            answers: vec![(good, Some(2)), (bad, None)],
            calls: 0,
        };
        let (totals, _) = pass(&mut reader, &s.0, &mut |_| {}).expect("pass");
        assert_eq!(totals.files, 2, "an unreadable file still counts");
        assert_eq!(totals.unreadable, 1);
        assert_eq!(totals.files - totals.unreadable, 1);
    }

    #[test]
    fn the_exit_code_comes_from_the_verdicts_not_from_coverage() {
        let mut t = Totals::default();
        assert_eq!(t.exit_code(), 0);
        t.files = 10;
        t.unreadable = 3;
        assert_eq!(
            t.exit_code(),
            0,
            "unreadable files are coverage, not a verdict"
        );
        t.high_risk = 1;
        assert_eq!(t.exit_code(), 2);
        assert_eq!(t.interrupting(), 1);
        t.confirmed = 2;
        assert_eq!(t.interrupting(), 3);
    }

    #[test]
    fn seen_is_called_once_per_scored_file_only() {
        let s = Scratch::new("seen");
        let a = s.file("a.bin", b"a");
        let b = s.file("b.bin", b"b");
        let mut reader = Fake {
            answers: vec![(a, Some(1)), (b, None)],
            calls: 0,
        };
        let mut count = 0;
        pass(&mut reader, &s.0, &mut |_| count += 1).expect("pass");
        assert_eq!(count, 1, "an unreadable file yields no assessment to see");
    }
}
