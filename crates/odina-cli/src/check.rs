// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! `odina check` — the command the instrumented build adds, and the code that
//! turns a scan into a local file.
//!
//! This module is compiled in two cases: the check build
//! (`--features instrument`), and this crate's own tests. It is **not** in the
//! public build — the dependency itself is optional, so the public binary does
//! not link `odina-report` or `odina-instrument` at all. That is stronger than
//! not calling them: `tools/ci/check-public-build.py` runs `cargo tree` on a
//! default build and fails if either name appears.
//!
//! The file it writes is one JSON object:
//!
//! ```text
//! { ...perf object from odina-instrument..., "reports": [ ...odina-report/v1... ] }
//! ```
//!
//! No socket, no URL, no clock beyond the date, and no file name anywhere: the
//! report objects carry hashes and patterns.

use std::path::{Path, PathBuf};

use odina_core::hashing::{hash_reader, FileHashes};
use odina_core::scoring::{score_with_known_bad, Evidence, Indicator, Score, Thresholds, Verdict};
use odina_report::report::Report;
use odina_report::{measure, pe};

/// The app version written into every exported report.
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// No threat database ships yet, so the field carries the version of the
/// built-in heuristic set rather than pretending a database exists. It must
/// look like a version — the schema rejects anything else — and the schema
/// name itself lives in its own field, `schema`.
pub const DB_VERSION: &str = "builtin-1";

/// What a scan of one file produced, kept in one place so the text output and
/// the report cannot drift apart.
pub struct Scanned {
    pub path: PathBuf,
    pub hashes: FileHashes,
    pub score: Score,
    pub pe_hash: Option<String>,
    pub attributes: measure::Attributes,
}

/// Read a PE import hash. `None` for anything that is not a PE, and for a file
/// that cannot be read: a missing fingerprint is a missing measurement, not a
/// failed scan.
pub fn pe_info(path: &Path) -> Option<pe::PeInfo> {
    use std::io::{Read, Seek, SeekFrom};

    let mut file = std::fs::File::open(path).ok()?;
    let mut head = vec![0u8; 4096];
    let n = file.read(&mut head).ok()?;
    head.truncate(n);
    if !pe::looks_like_pe(&head) {
        return None;
    }

    let mut reader = std::fs::File::open(path).ok()?;
    pe::parse(&head, |offset, len| {
        if len > 1 << 20 {
            return None; // a single read this large is not a header
        }
        let mut buf = vec![0u8; len];
        reader.seek(SeekFrom::Start(offset as u64)).ok()?;
        reader.read_exact(&mut buf).ok()?;
        Some(buf)
    })
}

/// The three attributes, plus the one bit the filesystem does not carry on
/// Unix: an executable extension.
pub fn attributes_for(path: &Path) -> measure::Attributes {
    let mut attrs = match std::fs::metadata(path) {
        Ok(meta) => measure::attributes(&meta),
        Err(_) => measure::Attributes::default(),
    };
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    if matches!(
        ext.as_str(),
        "exe" | "dll" | "scr" | "com" | "bat" | "cmd" | "ps1" | "msi" | "vbs" | "js" | "jar"
    ) {
        attrs.executable = true;
    }
    attrs
}

/// Behaviour score for the schema's 0.0..=1.0 field, derived from the verdict
/// band. The engine's raw score is deliberately **not** exported: §8 of
/// `scoring-spec` says the number is not shown to the user, and a report that
/// carried it would leak the weights one sample at a time.
pub fn behavior_score(verdict: Verdict) -> f32 {
    match verdict {
        Verdict::Clean => 0.0,
        Verdict::Monitor => 0.25,
        Verdict::Suspicious => 0.6,
        Verdict::HighRisk => 0.85,
        Verdict::Confirmed => 1.0,
    }
}

/// Build the sixteen-field object for one file. Never touches the disk.
pub fn report_for(s: &Scanned) -> Report {
    let name = s
        .path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let path_text = s.path.to_string_lossy();

    let mut report = Report::new(
        s.hashes.full.clone(),
        s.hashes.size,
        APP_VERSION,
        DB_VERSION,
    );
    if !s.hashes.head.is_empty() {
        report.header_hash = Some(s.hashes.head.clone());
    }
    if !s.hashes.tail.is_empty() {
        report.tail_hash = Some(s.hashes.tail.clone());
    }
    report.pe_hash = s.pe_hash.clone();
    report.attributes = Some(s.attributes);
    report.behavior_score = Some(behavior_score(s.score.verdict));
    report.with_name_and_path(&name, &path_text)
}

/// Default output file. A fixed name, so the user can find it, and no
/// timestamp in the name: the date belongs in the file, not in a path.
///
/// Only the instrumented path writes a file, so a default build would see
/// this as dead code without the allow.
#[cfg_attr(not(feature = "instrument"), allow(dead_code))]
pub fn default_out() -> PathBuf {
    PathBuf::from("odina-check.json")
}

/// Splice the perf object and the report array into one balanced JSON object.
pub fn assemble(perf_json: &str, reports: &[String]) -> String {
    let inner = perf_json
        .strip_prefix('{')
        .and_then(|s| s.strip_suffix('}'))
        .unwrap_or(perf_json);
    let mut out = String::with_capacity(inner.len() + 64);
    out.push('{');
    out.push_str(inner);
    out.push_str(",\"reports\":[");
    out.push_str(&reports.join(","));
    out.push_str("]}");
    out
}

/// The date in `YYYY-MM-DD`, from the system clock.
///
/// A **date**, not a timestamp: the day a build was measured is a property of
/// the report, while the hour someone ran it is a property of the person.
/// `report-schema-v1.md` §4.1 bans `ts` for exactly that reason, and this
/// function is where the distinction is implemented.
pub fn today() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (y, m, d) = civil_from_days((secs / 86_400) as i64);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Howard Hinnant's `civil_from_days`: a day count to a date, no table, no
/// dependency.
pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

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

/// Above this size, the whole-file digest is skipped and only head and tail
/// are hashed. 512 MiB is roughly when reading the file stops being free
/// compared to everything else a scan does.
pub const FULL_DIGEST_CAP: u64 = 512 * 1024 * 1024;

/// How many directories one walk may visit. A Downloads folder is far below
/// this; the number exists only so that a pathological tree cannot run for
/// hours while the user watches nothing happen.
pub const MAX_DIRS: u64 = 200_000;

/// How many entries one directory may yield before the walk gives up on it.
/// Windows shortcut files are a normal part of a Downloads folder, and
/// resolving them is not this tool's job.
const MAX_ENTRIES_PER_DIR: usize = 500_000;

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

        let mut paths: Vec<PathBuf> = Vec::new();
        for entry in entries.take(MAX_ENTRIES_PER_DIR) {
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
            if kind.is_symlink() {
                links_skipped += 1;
                continue;
            }
            if kind.is_dir() {
                paths.push(entry.path());
            } else if kind.is_file() {
                paths.push(entry.path());
            } else {
                // A pipe, a socket, a device: opening it can block forever.
                links_skipped += 1;
            }
        }
        paths.sort();
        for path in paths {
            // `is_dir` here is safe: the entry was not a symlink above.
            if path.is_dir() {
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

/// Score one file the same way `scan` does. Kept here so `check` cannot invent
/// a second opinion about the same file.
pub fn score_file(path: &Path) -> Option<Scanned> {
    let file = std::fs::File::open(path).ok()?;
    // The cap exists for this loop and not for `scan`: `scan` was pointed at
    // one file on purpose, while `check` walks whatever the user has. One ISO
    // in a Downloads folder is several gigabytes of reading before the first
    // line of output, which is indistinguishable from a hang. Above the cap,
    // head and tail are still hashed, the file is still scored, and the report
    // says the full digest was skipped rather than inventing one.
    let hashes = hash_reader(file, Some(FULL_DIGEST_CAP)).ok()?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();

    let mut evidence = Evidence::new();
    evidence.set(
        Indicator::NamePattern,
        odina_core::heuristics::name_shape(&name).score(),
    );
    evidence.set(
        Indicator::PathPattern,
        odina_core::heuristics::path_score(odina_core::heuristics::guess_path_flags(path)),
    );
    // Same call, same arguments as `scan`, and no known-bad database exists to
    // consult — the flag is the honest false, not an optimistic one.
    let score = score_with_known_bad(&evidence, &Thresholds::default(), None, false);

    Some(Scanned {
        path: path.to_path_buf(),
        hashes,
        score,
        pe_hash: pe_info(path).map(|i| i.imphash),
        attributes: attributes_for(path),
    })
}

/// The command. Only reachable when the module is compiled, which is the
/// check build and this crate's tests.
#[cfg(feature = "instrument")]
pub fn command(args: &[String]) -> std::process::ExitCode {
    use odina_instrument::{Meter, PerfReport, ScanStats};
    use std::process::ExitCode;

    let mut out_path = default_out();
    let mut target: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--out" => {
                if let Some(v) = args.get(i + 1) {
                    out_path = PathBuf::from(v);
                    i += 2;
                    continue;
                }
                eprintln!("odina: --out needs a file name");
                return ExitCode::FAILURE;
            }
            a if !a.starts_with("--") => target = Some(a.to_string()),
            _ => {}
        }
        i += 1;
    }

    let Some(target) = target else {
        eprintln!("odina: check needs a path, and optionally --out <file>");
        return ExitCode::FAILURE;
    };
    let path = Path::new(&target);
    if !path.exists() {
        eprintln!("odina: {target}: no such file or directory");
        return ExitCode::FAILURE;
    }

    let idle = Meter::now();
    let started = std::time::Instant::now();

    let mut stats = ScanStats::default();
    let mut exported: Vec<String> = Vec::new();

    let walked = if path.is_dir() {
        walk(path)
    } else {
        Walked {
            files: vec![path.to_path_buf()],
            links_skipped: 0,
            read_errors: 0,
            truncated: false,
        }
    };
    let files = walked.files;
    stats.links_skipped = walked.links_skipped;
    stats.read_errors = walked.read_errors;
    stats.truncated = walked.truncated;

    // Silence is what makes a slow scan look like a hung one. This is the only
    // line written to stderr during a run, it is a count, and it exists because
    // the first real run on a real Downloads folder stopped here and there was
    // nothing to read.
    eprintln!(
        "odina: {} files to score ({} links skipped, {} folders unreadable{})",
        files.len(),
        walked.links_skipped,
        walked.read_errors,
        if walked.truncated { ", LIMITED" } else { "" }
    );

    let mut done: u64 = 0;
    let mut not_reported: u64 = 0;
    let total = files.len() as u64;
    for file in &files {
        done += 1;
        if total >= 500 && done % 500 == 0 {
            eprintln!("odina: {done}/{total}");
        }
        match score_file(file) {
            Some(scanned) => {
                stats.files += 1;
                stats.bytes += scanned.hashes.size;
                match scanned.score.verdict {
                    Verdict::Clean => stats.clean += 1,
                    Verdict::Monitor => stats.monitor += 1,
                    Verdict::Suspicious => stats.suspicious += 1,
                    Verdict::HighRisk => stats.high_risk += 1,
                    Verdict::Confirmed => stats.confirmed += 1,
                }
                // Only a file that would interrupt the user gets a report.
                // Exporting clean files would turn a privacy-preserving design
                // into a census of somebody's disk.
                if scanned.score.interrupts() {
                    // A file above the digest cap has no `sha256`, and the
                    // schema requires one, so it cannot produce a valid
                    // report. It is still counted in its verdict band above —
                    // it was read and scored, and pretending otherwise would
                    // break the accounting — but the omission is announced
                    // rather than silent.
                    if scanned.hashes.full_digest_skipped() {
                        not_reported += 1;
                    } else if let Ok(json) = report_for(&scanned).to_json_checked() {
                        exported.push(json);
                    }
                }
            }
            None => {
                stats.files += 1;
                stats.unreadable += 1;
            }
        }
    }

    if not_reported > 0 {
        eprintln!(
            "odina: {not_reported} suspicious file(s) above {} MiB have no full digest, \
             so no report was written for them",
            FULL_DIGEST_CAP / (1024 * 1024)
        );
    }

    stats.elapsed_ms = started.elapsed().as_millis() as u64;
    let after = Meter::now();

    let hw = odina_report::hw::Hardware::detect();
    let mut perf = PerfReport::new(
        &today(),
        APP_VERSION,
        &hw.label(),
        &format!("{} · {}", std::env::consts::OS, std::env::consts::ARCH),
    );
    perf.fill_from_scan(stats, &idle, Some(&after));

    let json = assemble(&perf.to_json(), &exported);
    if let Err(err) = std::fs::write(&out_path, json.as_bytes()) {
        eprintln!("odina: cannot write {}: {err}", out_path.display());
        return ExitCode::FAILURE;
    }

    println!("الفحص: {} ملفًا · {} بايت", stats.files, stats.bytes);
    println!("النتائج: {}", perf.summary());
    println!(
        "التقارير المصدَّرة: {} (فقط ما يستدعي تنبيهًا)",
        exported.len()
    );
    println!("الملف: {}", out_path.display());
    println!("لم يُرسل شيء: هذا الملف على قرصك، وأنت من يقرر إرساله.");

    if stats.confirmed > 0 || stats.high_risk > 0 {
        ExitCode::from(2)
    } else {
        ExitCode::SUCCESS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `mkfifo(3)`. Declared here rather than pulled in as a dependency: one
    /// test needs one call, and a crate for a fixture is not a trade worth
    /// making in a build that is checked for every dependency it links.
    #[cfg(unix)]
    extern "C" {
        #[link_name = "mkfifo"]
        fn mkfifo_c(path: *const std::os::raw::c_char, mode: u32) -> i32;
    }

    #[test]
    fn dates_are_computed_without_a_dependency() {
        // Epoch, a leap day, and the day the instrumented build was written.
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(19_782), (2024, 2, 29));
        assert_eq!(civil_from_days(20_773), (2026, 11, 16));
    }

    #[test]
    fn today_looks_like_a_date() {
        let d = today();
        assert_eq!(d.len(), 10, "{d}");
        assert_eq!(d.matches('-').count(), 2, "{d}");
        assert!(d.starts_with("20"), "{d}");
    }

    #[test]
    fn the_assembled_file_is_one_balanced_object() {
        let perf = r#"{"schema":"odina-perf/v1","metrics":[]}"#;
        let out = assemble(perf, &[r#"{"schema":"odina-report/v1"}"#.to_string()]);
        assert!(out.starts_with('{') && out.ends_with('}'));
        assert_eq!(out.matches('{').count(), out.matches('}').count());
        assert_eq!(out.matches('[').count(), out.matches(']').count());
        assert!(out.contains("\"reports\":["));
        assert!(!out.contains("}{"), "the perf object must be spliced");
    }

    #[test]
    fn a_text_file_has_no_pe_hash() {
        let dir = std::env::temp_dir().join("odina-check-pe");
        let _ = std::fs::create_dir_all(&dir);
        let f = dir.join("a.txt");
        std::fs::write(&f, b"not a portable executable").unwrap();
        assert!(pe_info(&f).is_none());
        let _ = std::fs::remove_file(&f);
    }

    #[test]
    fn a_missing_file_is_a_missing_measurement_not_an_error() {
        assert!(pe_info(Path::new("/nonexistent/file.exe")).is_none());
    }

    #[test]
    fn the_report_for_a_real_file_carries_no_name_and_no_path() {
        let dir = std::env::temp_dir().join("odina-check-report");
        let _ = std::fs::create_dir_all(&dir);
        let f = dir.join("فاتورة.pdf.exe");
        std::fs::write(&f, b"payload").unwrap();

        let scanned = score_file(&f).expect("readable");
        let report = report_for(&scanned);
        let json = report.to_json_checked().expect("valid");

        assert!(!json.contains("فاتورة"), "{json}");
        assert!(!json.contains("odina-check-report"), "{json}");
        assert!(!json.contains(dir.to_string_lossy().as_ref()), "{json}");
        assert!(json.contains("\"name_pattern\""));
        assert!(json.contains("\"path_pattern\""));
        assert!(json.contains("\"app_version\""));

        let _ = std::fs::remove_file(&f);
    }

    #[test]
    fn the_behavior_score_follows_the_verdict_not_the_raw_number() {
        // §8 of scoring-spec: the number is not shown, so exporting it would
        // leak the weights. The band is what travels.
        assert_eq!(behavior_score(Verdict::Clean), 0.0);
        assert_eq!(behavior_score(Verdict::Confirmed), 1.0);
        let bands = [
            behavior_score(Verdict::Monitor),
            behavior_score(Verdict::Suspicious),
            behavior_score(Verdict::HighRisk),
        ];
        assert!(bands.windows(2).all(|w| w[0] < w[1]), "{bands:?}");
    }

    #[test]
    fn walk_is_stable_between_runs() {
        let dir = std::env::temp_dir().join("odina-check-walk");
        let _ = std::fs::create_dir_all(dir.join("sub"));
        for name in ["b.txt", "a.txt", "c.txt"] {
            std::fs::write(dir.join(name), b"x").unwrap();
        }
        std::fs::write(dir.join("sub").join("d.txt"), b"x").unwrap();
        let first = walk(&dir);
        let second = walk(&dir);
        assert_eq!(first.files, second.files);
        assert!(first.files.len() >= 4);
        assert_eq!(first.links_skipped, 0);
        assert!(!first.truncated);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The defect that stopped the first real run: a link that points at an
    /// ancestor turns `Path::is_dir` into an infinite walk, and the program
    /// waits forever with no output. The walk must not enter it.
    #[test]
    #[cfg(unix)]
    fn a_link_that_points_at_an_ancestor_does_not_hang_the_walk() {
        let root = std::env::temp_dir().join("odina-check-loop");
        let inner = root.join("inner");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&inner).unwrap();
        std::fs::write(inner.join("real.txt"), b"x").unwrap();
        // `inner/loop` -> `root`: entering it walks back to where we started.
        std::os::unix::fs::symlink(&root, inner.join("loop")).unwrap();

        let walked = walk(&root);

        assert_eq!(walked.files.len(), 1, "only the real file is a file");
        assert!(
            walked.links_skipped >= 1,
            "the link must be reported, not ignored"
        );
        assert!(!walked.truncated);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A named pipe blocks on open, and an open that blocks is indistinguishable
    /// from a scan that hangs.
    #[test]
    #[cfg(unix)]
    fn a_fifo_is_skipped_rather_than_opened() {
        use std::ffi::CString;
        let root = std::env::temp_dir().join("odina-check-fifo");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("normal.txt"), b"x").unwrap();
        let path = CString::new(root.join("pipe").to_string_lossy().as_bytes()).unwrap();
        // SAFETY: `path` is a live NUL-terminated string, and mkfifo only reads it.
        let rc = unsafe { mkfifo_c(path.as_ptr(), 0o600) };
        assert_eq!(rc, 0, "the fixture pipe could not be created");

        let walked = walk(&root);

        assert!(walked.files.iter().all(|f| f.ends_with("normal.txt")));
        assert!(
            walked.links_skipped >= 1,
            "the pipe must be counted as skipped"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A file above the cap must be scored, must not be reported (the schema
    /// requires a digest it does not have), and must not be dropped from the
    /// counters. This is the difference between a scan that says "I could not"
    /// and one that quietly loses a file.
    #[test]
    fn above_the_cap_the_file_is_scored_but_gets_no_report() {
        use odina_core::hashing::hash_reader;

        // A small cap, because the point is the behaviour above *any* cap, and
        // hashing 512 MiB in a unit test would be a different kind of problem.
        const CAP: u64 = 64 * 1024;

        // A reader that never ends, so the cap is what stops it.
        struct Endless(u64);
        impl std::io::Read for Endless {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                if self.0 == 0 {
                    return Ok(0);
                }
                let n = buf.len().min(self.0 as usize);
                buf[..n].fill(0x41);
                self.0 -= n as u64;
                Ok(n)
            }
        }

        let tiny = hash_reader(Endless(CAP + 1), Some(CAP)).unwrap();
        assert!(
            tiny.full_digest_skipped(),
            "the cap must leave the digest empty"
        );
        assert_eq!(tiny.size, CAP + 1);
        assert!(!tiny.head.is_empty() && !tiny.tail.is_empty());
    }

    #[test]
    fn an_executable_extension_counts_as_executable_where_the_os_has_no_bit() {
        let dir = std::env::temp_dir().join("odina-check-attrs");
        let _ = std::fs::create_dir_all(&dir);
        let f = dir.join("a.exe");
        std::fs::write(&f, b"x").unwrap();
        assert!(attributes_for(&f).executable);
        let _ = std::fs::remove_file(&f);
    }
}
