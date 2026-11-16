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
use odina_report::report::{Report, REPORT_SCHEMA};
use odina_report::{measure, pe};

/// The app version written into every exported report.
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

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
        REPORT_SCHEMA, // no threat database ships yet; the field says so
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

/// Every file under `dir`, in a stable order. Two runs on the same folder must
/// produce the same sequence, or the numbers cannot be compared.
pub fn walk(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let mut entries: Vec<PathBuf> = match std::fs::read_dir(&current) {
            Ok(rd) => rd.filter_map(|e| e.ok()).map(|e| e.path()).collect(),
            Err(_) => continue,
        };
        entries.sort();
        for p in entries {
            if p.is_dir() {
                stack.push(p);
            } else {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

/// Score one file the same way `scan` does. Kept here so `check` cannot invent
/// a second opinion about the same file.
pub fn score_file(path: &Path) -> Option<Scanned> {
    let file = std::fs::File::open(path).ok()?;
    // No cap on the digest, same as `scan`: the user pointed at a folder and
    // asked about it, and a silently skipped digest would be a different
    // answer, not a faster one.
    let hashes = hash_reader(file, None).ok()?;
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

    let files = if path.is_dir() {
        walk(path)
    } else {
        vec![path.to_path_buf()]
    };

    for file in &files {
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
                    if let Ok(json) = report_for(&scanned).to_json_checked() {
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
        assert_eq!(first, second);
        assert!(first.len() >= 4);
        let _ = std::fs::remove_dir_all(&dir);
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
