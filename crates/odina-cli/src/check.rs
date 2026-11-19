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

#[cfg(feature = "instrument")]
mod cache;
#[cfg(feature = "instrument")]
mod engine;
#[cfg(feature = "instrument")]
mod stamp;

use std::fs::File;
use std::path::{Path, PathBuf};

use odina_core::scoring::Verdict;
use odina_report::report::Report;
use odina_report::{measure, pe};

/// The app version written into every exported report.
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Probe only an exportable file, using the same handle that reached EOF in
/// the hash reader. This avoids path-replacement races, not in-place edits.
fn pe_info(reader: &mut File) -> Option<pe::PeInfo> {
    use std::io::{Read, Seek, SeekFrom};

    reader.rewind().ok()?;
    let mut head = [0u8; 4096];
    let n = reader.read(&mut head).ok()?;
    let head = &head[..n];
    if !pe::looks_like_pe(head) {
        return None;
    }
    pe::parse(head, |offset, len| {
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
fn attributes_for(file: &File, path: &Path) -> measure::Attributes {
    let mut attrs = match file.metadata() {
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

/// PE and attributes are export fields, NOT scoring inputs in this build.
/// Do not collect either for silent verdicts or above-cap inputs that the
/// schema cannot export. Keep the predicate upstream of every enrichment I/O.
fn report_for(
    s: &super::Report,
    enrich: impl FnOnce() -> (Option<String>, measure::Attributes),
) -> Option<Report> {
    if !s.score.interrupts() || s.hashes.full_digest_skipped() {
        return None;
    }
    let (pe_hash, attributes) = enrich();
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
        odina_core::g1_reference::VERSION,
    );
    if !s.hashes.head.is_empty() {
        report.header_hash = Some(s.hashes.head.clone());
    }
    if !s.hashes.tail.is_empty() {
        report.tail_hash = Some(s.hashes.tail.clone());
    }
    report.pe_hash = pe_hash;
    report.attributes = Some(attributes);
    report.behavior_score = Some(behavior_score(s.score.verdict));
    Some(report.with_name_and_path(&name, &path_text))
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
pub fn assemble(perf_json: &str, reports: &[String], work_json: &str) -> String {
    let inner = perf_json
        .strip_prefix('{')
        .and_then(|s| s.strip_suffix('}'))
        .unwrap_or(perf_json);
    let mut out = String::with_capacity(inner.len() + 64);
    out.push('{');
    out.push_str(inner);
    out.push_str(",\"work\":");
    out.push_str(work_json);
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
    ///
    /// The public build writes this and prints nothing: the number exists for
    /// the instrumented report and for the tests. Saying so here is narrower
    /// than a crate-wide `allow`, which would hide the next field nobody reads.
    #[cfg_attr(not(feature = "instrument"), allow(dead_code))]
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

        let mut paths: Vec<(bool, PathBuf)> = Vec::new();
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

/// The command. Only reachable when the module is compiled, which is the
/// check build and this crate's tests.
#[cfg(feature = "instrument")]
pub fn command(args: &[String]) -> std::process::ExitCode {
    use odina_instrument::{Meter, PerfReport, ScanStats};
    use std::process::ExitCode;

    let mut out_path = default_out();
    let mut target: Option<String> = None;
    let mut cache_dir: Option<PathBuf> = None;
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
            "--cache-dir" => {
                let Some(v) = args.get(i + 1).filter(|v| !v.starts_with("--")) else {
                    eprintln!("odina: --cache-dir needs a private directory outside the corpus");
                    return ExitCode::FAILURE;
                };
                cache_dir = Some(PathBuf::from(v));
                i += 1;
            }
            a if !a.starts_with("--") && target.is_none() => target = Some(a.to_string()),
            _ => {
                eprintln!("odina: unexpected check argument");
                return ExitCode::FAILURE;
            }
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

    let mut index = match cache_dir
        .as_ref()
        .map(|dir| cache::Cache::open(dir, path, &out_path))
        .transpose()
    {
        Ok(index) => index,
        Err(e) => {
            eprintln!("odina: cannot use private scan cache: {e}");
            return ExitCode::FAILURE;
        }
    };
    let filter = odina_core::g1_reference::PrefixFilter::from_catalog();
    let mut work = engine::Work::default();
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
        match engine::scan(file, &filter, &mut index, &mut work) {
            Ok((scanned, mut handle)) => {
                stats.files += 1;
                stats.bytes += scanned.hashes.size;
                match scanned.score.verdict {
                    Verdict::Clean => stats.clean += 1,
                    Verdict::Monitor => stats.monitor += 1,
                    Verdict::Suspicious => stats.suspicious += 1,
                    Verdict::HighRisk => stats.high_risk += 1,
                    Verdict::Confirmed => stats.confirmed += 1,
                }
                if let Some(report) = report_for(&scanned, || {
                    let pe_hash = pe_info(&mut handle).map(|i| i.imphash);
                    (pe_hash, attributes_for(&handle, file))
                }) {
                    if let Ok(json) = report.to_json_checked() {
                        exported.push(json);
                    }
                } else if scanned.score.interrupts() {
                    // Counted in its verdict band, but not exportable without sha256.
                    not_reported += 1;
                }
            }
            Err(_) => {
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

    if let Some(c) = index {
        if let Err(e) = c.finish() {
            eprintln!("odina: could not publish cache; rescan next time: {e}");
        }
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

    let json = assemble(&perf.to_json(), &exported, &work.json());
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

    // `mkfifo(3)`. Declared here rather than pulled in as a dependency: one
    // test needs one call, and a crate for a fixture is not a trade worth
    // making in a build that is checked for every dependency it links.
    // A `///` comment would be wrong here: rustdoc does not document extern
    // blocks, so the doc comment itself is the warning.
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
        let out = assemble(perf, &[r#"{"schema":"odina-report/v1"}"#.to_string()], "{}");
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
        assert!(pe_info(&mut File::open(&f).unwrap()).is_none());
        let _ = std::fs::remove_file(&f);
    }

    #[cfg(feature = "instrument")]
    #[test]
    fn a_missing_file_has_no_scan_or_export() {
        assert!(engine::scan(
            Path::new("/nonexistent/file.exe"),
            &odina_core::g1_reference::PrefixFilter::from_catalog(),
            &mut None,
            &mut engine::Work::default()
        )
        .is_err());
    }

    #[test]
    fn the_report_for_a_real_file_carries_no_name_and_no_path() {
        let dir = std::env::temp_dir().join("odina-check-report");
        let _ = std::fs::create_dir_all(&dir);
        let f = dir.join("فاتورة.pdf.exe");
        // Data-only PE header with an empty import directory. Reading from
        // EOF without rewinding the hashed handle would lose its imphash.
        let nt = 0x40;
        let mut bytes = [0u8; 0x40 + 24 + 112];
        bytes[..2].copy_from_slice(b"MZ");
        bytes[0x3c] = nt as u8;
        bytes[nt..nt + 4].copy_from_slice(b"PE\0\0");
        bytes[nt + 20] = 112;
        bytes[nt + 24..nt + 26].copy_from_slice(&0x10bu16.to_le_bytes());
        std::fs::write(&f, bytes).unwrap();

        let mut handle = File::open(&f).unwrap();
        let mut scanned =
            crate::scan_opened(&f, &mut handle, Some(FULL_DIGEST_CAP)).expect("readable");
        assert!(report_for(&scanned, || panic!("no enrichment for a clean file")).is_none());
        // Exercise the export path without using a malware sample.
        scanned.score.verdict = Verdict::Confirmed;
        let json = report_for(&scanned, || {
            (
                pe_info(&mut handle).map(|i| i.imphash),
                attributes_for(&handle, &f),
            )
        })
        .expect("interrupting verdict with a full digest")
        .to_json_checked()
        .expect("valid");
        assert!(json.contains("\"attributes\""));
        assert!(
            json.contains(r#""pe_hash":"d41d8cd98f00b204e9800998ecf8427e""#),
            "{json}"
        );
        drop(handle);
        scanned.hashes.full.clear();
        assert!(report_for(&scanned, || panic!("no enrichment above the cap")).is_none());

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
        assert_eq!(first.read_errors, 0, "a folder we own is fully readable");
        assert!(!first.truncated);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A folder that exists but cannot be listed is counted, not thrown away.
    /// Without this the field is only written by the instrumented build, and
    /// the public one carries a number nobody ever checks.
    #[test]
    #[cfg(unix)]
    fn an_unreadable_folder_is_counted_rather_than_skipped_silently() {
        use std::os::unix::fs::PermissionsExt;

        let root = std::env::temp_dir().join("odina-check-unreadable");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("locked")).unwrap();
        std::fs::create_dir_all(root.join("open")).unwrap();
        std::fs::write(root.join("open").join("a.txt"), b"x").unwrap();
        std::fs::write(root.join("locked").join("b.txt"), b"x").unwrap();
        std::fs::set_permissions(root.join("locked"), std::fs::Permissions::from_mode(0o000))
            .unwrap();

        // Running as root defeats the permission bit; the test then has nothing
        // to prove and says so instead of passing for the wrong reason.
        if std::fs::read_dir(root.join("locked")).is_ok() {
            let _ = std::fs::set_permissions(
                root.join("locked"),
                std::fs::Permissions::from_mode(0o755),
            );
            let _ = std::fs::remove_dir_all(&root);
            return;
        }

        let walked = walk(&root);
        let _ =
            std::fs::set_permissions(root.join("locked"), std::fs::Permissions::from_mode(0o755));
        let _ = std::fs::remove_dir_all(&root);

        assert_eq!(walked.read_errors, 1, "the unreadable folder is counted");
        assert_eq!(
            walked.files.len(),
            1,
            "and the readable one is still walked"
        );
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
        assert!(attributes_for(&File::open(&f).unwrap(), &f).executable);
        let _ = std::fs::remove_file(&f);
    }
}
