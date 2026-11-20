// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! `odina scan <file>` — the smallest program that can be wrong in public.
//!
//! This is gate **G1** from `docs/roadmap.md`: one command classifies a real
//! file. It exists before the app, before Tauri, before any installer, for one
//! reason — to make the engine answerable to a file the user can point at.
//!
//! What it does honestly today:
//!
//!   * hashes the file (full, first 4 KB, last 4 KB) and prints the hashes;
//!   * measures name shape and path flags;
//!   * looks up the full digest in the local EICAR-only reference (V5);
//!   * scores the other evidence with the spec's weights;
//!   * prints the verdict, the reason, and every cap that applied.
//!
//! What it does **not** do, and says so in its own output:
//!
//!   * no production threat database ships; the single test signature proves
//!     the path to V5, not detection coverage;
//!   * there is no publisher signature check, so indicator 9 stays at zero;
//!   * nothing is blocked, isolated, or deleted. This command only looks.
//!
//! Run `odina selftest` to verify the engine against the standard test file.
//!
//! ## Two builds, one source
//!
//! The public build has no measurement code in it at all: `odina-report` and
//! `odina-instrument` are optional dependencies, so a default build does not
//! link them. The instrumented build (`--features instrument`) adds one
//! command, `check`, which writes a local JSON file of counts and timings, and
//! can export an `odina-report/v1` object for each file that would interrupt
//! the user. Nothing is ever sent — the file is the user's to send or delete.
//! See `docs/instrumented-build.md`.

// Compiled only for the check build, or for this crate's own tests. In the
// public build the module does not exist, and neither does the dependency it
// needs.
#[cfg(any(feature = "instrument", test))]
mod check;

mod repair;
mod resident;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use odina_core::assessment::{assess, Assessment as Report};
use odina_core::g1_reference;
use odina_core::hashing::{hash_bytes, hash_reader};
use odina_core::Score;
#[cfg(test)]
use odina_core::{score_with_known_bad, Evidence, FileHashes, Indicator, Thresholds};

const USAGE: &str = "\
odina — scan one file and print the decision (gate G1)

USAGE:
    odina scan <path> [--json] [--recursive]
    odina repair-db [--demo]       # verify external data; optional in-memory repair exercise
    odina selftest                 # in memory: writes nothing to disk
    # The resident service is part of the product, not a lab tool: it is in
    # every build.
    odina serve <path> --service NAME [--cache-dir <private-directory>]
    odina status|scan-now|stop --service NAME
    odina verdict <relative-path> --service NAME
    odina check <path> [--out <file>] [--cache-dir <private-directory>]   # instrumented build only
    odina check <path> --service NAME --out <new-file>                    # instrumented build only
    odina --version | --help

EXIT CODES:
    0   clean or monitor   — nothing would be shown to the user
    2   suspicious or above — the app would interrupt the user
    1   the command itself failed (missing file, unreadable, bad usage)

A directory is scanned one level deep unless --recursive is given.
This binary never blocks, isolates, or deletes input files.

`check` exists only in the instrumented build. It uses the same decision rules,
writes one local JSON report and, only with --cache-dir, a private local index.
Keep the index and report outside the scan target. Nothing is sent anywhere.";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--help") | Some("-h") | None => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Some("--version") | Some("-V") => {
            println!(
                "odina {} (core {})",
                env!("CARGO_PKG_VERSION"),
                odina_core::VERSION
            );
            ExitCode::SUCCESS
        }
        Some("selftest") => selftest(),
        Some("repair-db") => repair::command(&args[1..]),
        // The resident service ships in every build. It is the core of a
        // virus guard, not a lab tool; only the report layer is measured.
        Some(cmd @ ("serve" | "status" | "scan-now" | "stop" | "verdict")) => {
            resident::command(cmd, &args[1..])
        }
        // The public refusal is outside the module, so it has no check code.
        Some("check") => {
            #[cfg(feature = "instrument")]
            {
                check::command(&args[1..])
            }
            #[cfg(not(feature = "instrument"))]
            {
                eprintln!(
                    "odina: this is the public build; it contains no measurement code.\n\
                     \n\
                     The check build is the same source built with --features instrument.\n\
                     It adds local measurements and a local export of reports; it sends\n\
                     nothing, and the file it writes is yours. See docs/instrumented-build.md."
                );
                ExitCode::FAILURE
            }
        }
        Some("scan") => {
            let json = args.iter().any(|a| a == "--json");
            let recursive = args.iter().any(|a| a == "--recursive" || a == "-r");
            let Some(target) = args
                .iter()
                .skip(1)
                .find(|a| !a.starts_with("--") && *a != "-r")
            else {
                eprintln!("odina: scan needs a path\n\n{USAGE}");
                return ExitCode::FAILURE;
            };
            let path = Path::new(target);

            if path.is_dir() {
                return scan_dir(path, json, recursive);
            }

            match scan(path, None) {
                Ok(report) => {
                    if json {
                        println!("{}", report_to_json(&report));
                    } else {
                        print!("{}", report_to_text(&report));
                    }
                    if report.score.interrupts() {
                        ExitCode::from(2)
                    } else {
                        ExitCode::SUCCESS
                    }
                }
                Err(err) => {
                    eprintln!("odina: {target}: {err}");
                    ExitCode::FAILURE
                }
            }
        }
        Some(other) => {
            eprintln!("odina: unknown command {other}\n\n{USAGE}");
            ExitCode::FAILURE
        }
    }
}

/// Scan every file in a directory. This is what makes the command useful
/// rather than a demo: pointing it at `~/Downloads` answers the question the
/// user actually has.
///
/// One unreadable file does not abort the run — it is counted and named at the
/// end. A scanner that dies on a locked file is a scanner the user stops
/// trusting.
fn scan_dir(dir: &Path, json: bool, recursive: bool) -> ExitCode {
    let entries = match collect_files(dir, recursive) {
        Ok(list) => list,
        Err(err) => {
            eprintln!("odina: {}: {err}", dir.display());
            return ExitCode::FAILURE;
        }
    };

    let mut reports = Vec::new();
    let mut unreadable: Vec<(PathBuf, String)> = Vec::new();

    for path in entries {
        match scan(&path, None) {
            Ok(report) => reports.push(report),
            Err(err) => unreadable.push((path, err.to_string())),
        }
    }

    let interrupting = reports.iter().filter(|r| r.score.interrupts()).count();

    if json {
        let mut out = String::from("[");
        for (i, r) in reports.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str(&report_to_json(r));
        }
        out.push(']');
        println!("{out}");
    } else {
        println!("المجلد: {}", dir.display());
        println!("الملفات: {}", reports.len());
        println!();
        for r in &reports {
            let (ar, _) = verdict_label(&r.score);
            println!("  {:<14} {:.2}  {}", ar, r.score.value, r.path.display());
        }
        println!();
        for (path, err) in &unreadable {
            println!("  تعذّرت قراءته: {} ({err})", path.display());
        }
        println!(
            "الخلاصة: {} ملفًا · {} يستدعي تنبيهًا · {} تعذّرت قراءته",
            reports.len(),
            interrupting,
            unreadable.len()
        );
        println!("مرجع البصمات: بصمة EICAR الاختبارية فقط، وليست قاعدة تهديدات إنتاجية");
        println!("هذه الأداة تقرأ فقط: لا تمنع، ولا تعزل، ولا تحذف.");
    }

    if interrupting > 0 {
        ExitCode::from(2)
    } else if !unreadable.is_empty() && reports.is_empty() {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// Every file under `dir`, skipping anything that could make the walk hang or
/// run forever.
///
/// List every file under `dir`, mirroring `odina_scan::walk`'s rules so
/// `scan_dir` and the resident pass agree: symlinks are never entered
/// (`Path::is_dir` follows junctions, and one pointing at an ancestor is an
/// infinite walk), non-regular files are never opened (a named pipe blocks on
/// open), and results are sorted for a stable report.
///
/// This is intentionally not routed through `odina_scan::pass`: `pass` owns an
/// entire inventory (walk, `Totals`, index boundaries) and reports through a
/// callback, whereas `scan_dir` wants a plain sorted path list it scores file
/// by file and prints in its own tabular format. Two shallow listings with the
/// same skip rules, not two pass loops.
fn collect_files(dir: &Path, recursive: bool) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    let mut visited: u64 = 0;
    while let Some(current) = stack.pop() {
        if visited >= 200_000 {
            break;
        }
        visited += 1;
        // Kept as two separate lists rather than one list of pairs: a
        // `FileType` has no ordering, and sorting a tuple that contains one
        // does not compile. Splitting also says what the loop means — these
        // are folders, these are files, and a link is neither.
        let mut dirs: Vec<PathBuf> = Vec::new();
        let mut plain: Vec<PathBuf> = Vec::new();
        for entry in std::fs::read_dir(&current)?.filter_map(|e| e.ok()) {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_symlink() {
                continue;
            }
            let path = entry.path();
            if kind.is_dir() {
                dirs.push(path);
            } else if kind.is_file() {
                plain.push(path);
            }
        }
        dirs.sort();
        plain.sort();
        files.extend(plain);
        if recursive {
            stack.extend(dirs);
        }
    }
    files.sort();
    Ok(files)
}

/// Public digest-reporting path for a single file: open, hash, assess. `odina
/// scan <file>` is single-shot by design, so it never touches an index and
/// never omits a requested hash.
///
/// Not routed through `odina_scan::pass`: `pass` walks a whole tree and folds
/// it into `Totals` + an index boundary, while `scan` answers about one path
/// and returns a `Report` — a different shape entirely. Same decision as
/// `check` (`assess` over `hash_reader`), different loop.
fn scan(path: &Path, max_full_digest: Option<u64>) -> std::io::Result<Report> {
    scan_opened(path, &mut std::fs::File::open(path)?, max_full_digest)
}

/// Hash a handle and pass its evidence to the same decision as `check`.
fn scan_opened(
    path: &Path,
    file: &mut std::fs::File,
    max_full_digest: Option<u64>,
) -> std::io::Result<Report> {
    Ok(assess(path, hash_reader(file, max_full_digest)?))
}

fn verdict_label(score: &Score) -> (&'static str, &'static str) {
    use odina_core::scoring::Verdict::*;
    match score.verdict {
        Clean => ("آمن", "clean"),
        Monitor => ("مراقبة", "monitor"),
        Suspicious => ("مشتبه", "suspicious"),
        HighRisk => ("مشتبه بشدة", "high risk"),
        Confirmed => ("مؤكد", "confirmed"),
    }
}

/// The one-line reason a user would see in an alert. The spec (§8) forbids
/// percentages in the alert; this line is what replaces them.
fn reason(report: &Report) -> &'static str {
    if report.known_bad {
        return "بصمتها الكاملة تطابق ملف اختبار EICAR";
    }
    if report.name_score >= 0.40 && report.path_score >= 0.60 {
        return "اسمها خادع ومكانها غير معتاد";
    }
    if report.name_score >= 0.40 {
        return "اسمها مصمّم ليبدو غير ما هي";
    }
    if report.path_score >= 0.85 {
        return "مكانها غير معتاد لتشغيل برنامج";
    }
    if report.path_score >= 0.50 {
        return "نُزّلت أو وُضعت في مكان شائع للبرامج المؤذية";
    }
    "لا سبب — لا شيء يستدعي التنبيه"
}

fn report_to_text(report: &Report) -> String {
    let (ar, en) = verdict_label(&report.score);
    let mut out = String::new();

    out.push_str(&format!("الملف: {}\n", report.path.display()));
    out.push_str(&format!("الحجم: {} بايت\n", report.hashes.size));
    out.push_str(&format!("بصمة كاملة: {}\n", or_dash(&report.hashes.full)));
    out.push_str(&format!("بصمة الرأس:  {}\n", or_dash(&report.hashes.head)));
    out.push_str(&format!("بصمة الذيل:  {}\n", or_dash(&report.hashes.tail)));
    out.push('\n');
    out.push_str(&format!(
        "مرجع البصمات: EICAR الاختباري فقط — {}\n",
        if report.known_bad {
            "مطابقة"
        } else {
            "بلا مطابقة"
        }
    ));
    out.push_str(&format!("الحكم: {ar} ({en})\n"));
    out.push_str(&format!("السبب: {}\n", reason(report)));
    out.push_str(&format!("النتيجة: {:.2}\n", report.score.value));

    if !report.score.caps.is_empty() {
        let names: Vec<&str> = report.score.caps.iter().map(|c| c.label()).collect();
        out.push_str(&format!("سقوف مطبَّقة: {}\n", names.join(" · ")));
    }
    out.push_str(&format!(
        "الإجراء المقرّر (لا ينفّذه أمر الفحص): {}\n",
        match report.score.verdict {
            odina_core::Verdict::Confirmed => "منع التشغيل وبدء المعالجة",
            odina_core::Verdict::HighRisk | odina_core::Verdict::Suspicious => "منع التشغيل",
            _ => "لا إجراء",
        }
    ));

    out.push_str("\nالمؤشرات المقيسة فقط:\n");
    out.push_str(&format!("  اسم الملف (2): {:.2}\n", report.name_score));
    out.push_str(&format!("  المسار (3):    {:.2}\n", report.path_score));
    out.push_str("  الأخرى: بلا بيانات — لا تُحسب صفرًا كأنها سليمة، بل غائبة\n");

    out.push_str("\nهذه الأداة تقرأ فقط: لا تمنع، ولا تعزل، ولا تحذف.\n");
    out
}

/// Hand-written JSON. A serialiser crate would be a second dependency for one
/// output format, and this format is fixed by the report schema.
fn report_to_json(report: &Report) -> String {
    let (ar, en) = verdict_label(&report.score);
    let mut caps = String::from("[");
    for (i, cap) in report.score.caps.iter().enumerate() {
        if i > 0 {
            caps.push(',');
        }
        caps.push_str(&format!("\"{}\"", cap.label()));
    }
    caps.push(']');

    let veto = match report.score.veto {
        Some(v) => format!("\"{v:?}\""),
        None => "null".to_string(),
    };

    format!(
        concat!(
            "{{",
            "\"engine\":\"odina-core {core}\",",
            "\"cli\":\"{cli}\",",
            "\"path\":{path},",
            "\"size\":{size},",
            "\"hashes\":{{\"full\":{full},\"head\":{head},\"tail\":{tail}}},",
            "\"database_present\":true,\"db_version\":\"{db_version}\",",
            "\"known_bad\":{bad},",
            "\"score\":{score:.4},",
            "\"verdict\":\"{verdict_en}\",",
            "\"verdict_ar\":\"{verdict_ar}\",",
            "\"reason_ar\":{reason},",
            "\"timeout_action\":\"{timeout}\",",
            "\"caps\":{caps},",
            "\"veto\":{veto},",
            "\"interrupts\":{interrupts},",
            "\"indicators\":{{\"name\":{name:.4},\"path\":{pth:.4}}}",
            "}}"
        ),
        core = odina_core::VERSION,
        cli = env!("CARGO_PKG_VERSION"),
        path = json_string(&report.path.display().to_string()),
        size = report.hashes.size,
        full = json_string(or_dash(&report.hashes.full)),
        head = json_string(or_dash(&report.hashes.head)),
        tail = json_string(or_dash(&report.hashes.tail)),
        db_version = g1_reference::VERSION,
        bad = report.known_bad,
        score = report.score.value,
        verdict_en = en,
        verdict_ar = ar,
        reason = json_string(reason(report)),
        timeout = report.score.verdict.timeout_action(),
        caps = caps,
        veto = veto,
        interrupts = report.score.interrupts(),
        name = report.name_score,
        pth = report.path_score,
    )
}

fn or_dash(value: &str) -> &str {
    if value.is_empty() {
        "—"
    } else {
        value
    }
}

/// Minimal JSON string escaping: quotes, backslashes and control characters.
/// Paths on Windows contain backslashes, so this is not optional.
fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// `odina selftest` — proves the hashing engine against the standard 68-byte
/// EICAR string, **without writing it to disk**.
///
/// Why in memory: every antivirus on the machine (including Microsoft
/// Defender, which is on by default on Windows) treats the EICAR string as
/// malware and may quarantine or delete it the instant it appears in a file.
/// The old version wrote it to `%TEMP%` and scanned it; on a machine with a
/// live scanner that is a race, and the self-test then failed with a confusing
/// "file not found" that said nothing about Odina. The bytes are the test
/// vector, not a file: hashing them in memory tests the same code path
/// (`hash_bytes`, the one the tests use) without alarming anything else.
///
/// A real file path is still exercised by `odina scan` and by the CLI tests
/// that scan ordinary folders.
fn selftest() -> ExitCode {
    use g1_reference::EICAR_SHA256 as EXPECTED;
    let payload: Vec<u8> = [
        "X5O!P%@AP[4\\PZX54(P^)7CC)7}$",
        "EICAR-STANDARD-ANTIVIRUS-",
        "TEST-FILE!$H+H*",
    ]
    .concat()
    .into_bytes();

    let hashes = hash_bytes(&payload);
    let size_ok = hashes.size == 68;
    let full_ok = hashes.full == EXPECTED;
    // 68 bytes < 4 KB, so both windows must see the whole vector.
    let head_ok = hashes.head == EXPECTED;
    let tail_ok = hashes.tail == EXPECTED;

    println!("محرك Odina {}", odina_core::VERSION);
    println!("الاختبار: متجه EICAR القياسي، 68 بايتًا، في الذاكرة (لا ملف على القرص).");
    println!("الحجم:    {}", hashes.size);
    println!("البصمة:   {}", hashes.full);
    println!("الرأس:    {}", hashes.head);
    println!("الذيل:    {}", hashes.tail);
    println!();

    if size_ok && full_ok && head_ok && tail_ok {
        println!("الاختبار الذاتي: نجح — الحجم والبصمات الثلاث تطابق المتجه المنشور.");
        ExitCode::SUCCESS
    } else {
        println!(
            "الاختبار الذاتي: فشل — \
             size_ok={size_ok} full_ok={full_ok} head_ok={head_ok} tail_ok={tail_ok}"
        );
        ExitCode::FAILURE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_escapes_windows_paths() {
        let s = json_string(r"C:\Users\a\b.exe");
        assert_eq!(s, r#""C:\\Users\\a\\b.exe""#);
    }

    #[test]
    fn json_escapes_quotes_and_controls() {
        assert_eq!(json_string("a\"b"), r#""a\"b""#);
        assert_eq!(json_string("a\tb"), r#""a\tb""#);
    }

    #[test]
    fn a_confirmed_report_does_not_claim_the_file_was_quarantined() {
        let r = Report {
            path: PathBuf::from("sample.dat"),
            hashes: FileHashes::default(),
            known_bad: true,
            score: score_with_known_bad(&Evidence::new(), &Thresholds::default(), None, true),
            name_score: 0.0,
            path_score: 0.0,
        };
        assert!(report_to_json(&r).contains(r#""timeout_action":"block_and_remediate""#));
        assert!(report_to_text(&r).contains("منع التشغيل وبدء المعالجة"));
    }

    #[test]
    fn reason_prefers_the_strongest_true_statement() {
        // A misleading name on an unremarkable path.
        let mut e = Evidence::new();
        e.set(Indicator::NamePattern, 0.45);
        let r = Report {
            path: PathBuf::from("/home/u/x.pdf.exe"),
            hashes: FileHashes::default(),
            known_bad: false,
            score: score_with_known_bad(&e, &Thresholds::default(), None, false),
            name_score: 0.45,
            path_score: 0.0,
        };
        assert_eq!(reason(&r), "اسمها مصمّم ليبدو غير ما هي");
    }
}
