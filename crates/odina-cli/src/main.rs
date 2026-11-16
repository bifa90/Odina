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
//!   * scores them with the spec's weights;
//!   * prints the verdict, the reason, and every cap that applied.
//!
//! What it does **not** do, and says so in its own output:
//!
//!   * there is no signature database yet, so a known-bad hash cannot be
//!     recognised — the "database" line is printed as empty rather than
//!     implied to have been consulted;
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

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use odina_core::hashing::{hash_bytes, hash_reader, FileHashes};
use odina_core::heuristics::{guess_path_flags, name_shape, path_score};
use odina_core::scoring::{score_with_known_bad, Evidence, Indicator, Score, Thresholds};

const USAGE: &str = "\
odina — scan one file and print the decision (gate G1)

USAGE:
    odina scan <path> [--json] [--recursive]
    odina selftest                 # in memory: writes nothing to disk
    odina check <path> [--out <file>]   # instrumented build only
    odina --version | --help

EXIT CODES:
    0   clean or monitor   — nothing would be shown to the user
    2   suspicious or above — the app would interrupt the user
    1   the command itself failed (missing file, unreadable, bad usage)

A directory is scanned one level deep unless --recursive is given.
This binary never blocks, isolates, or deletes anything. It only reads.

`check` exists only in the instrumented build. It scans the same way, writes
one local JSON file of measurements, and sends nothing anywhere.";

/// Everything the report needs, assembled in one place so the two output
/// formats cannot drift apart.
struct Report {
    path: PathBuf,
    hashes: FileHashes,
    /// False until a signature database ships. Printed, never implied.
    database_present: bool,
    known_bad: bool,
    score: Score,
    name_score: f32,
    path_score: f32,
}

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
        // The refusal lives here, not inside the module, so the public build
        // really has no `check` code: the module is not compiled at all.
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

            match scan(path) {
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
        match scan(&path) {
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
        println!("قاعدة البصمات: غير موجودة بعد — لم يُقارن أي ملف بأي قاعدة");
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

fn collect_files(dir: &Path, recursive: bool) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let mut entries: Vec<_> = std::fs::read_dir(&current)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                if recursive {
                    stack.push(path);
                }
            } else {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn scan(path: &Path) -> std::io::Result<Report> {
    let file = std::fs::File::open(path)?;
    // No cap on the full digest: this is a one-file tool and the user asked
    // about this exact file. The app applies the cap for large files.
    let hashes = hash_reader(file, None)?;

    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let name_score = name_shape(&file_name).score();
    let path_flags = guess_path_flags(path);
    let path_score = path_score(path_flags);

    let mut evidence = Evidence::new();
    evidence.set(Indicator::NamePattern, name_score);
    evidence.set(Indicator::PathPattern, path_score);
    // Indicators 5-8 and 10 need a database or a parser that does not exist
    // yet. They stay at zero — an unmeasured indicator is not evidence.

    // No signature database ships yet, so nothing can be recognised as
    // known-bad. This flag is the honest false, not an optimistic one.
    let database_present = false;
    let known_bad = false;

    let score = score_with_known_bad(&evidence, &Thresholds::default(), None, known_bad);

    Ok(Report {
        path: path.to_path_buf(),
        hashes,
        database_present,
        known_bad,
        score,
        name_score,
        path_score,
    })
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
    if report.score.veto.is_some() {
        return "بصمتها مطابقة لعيّنة خبيثة معروفة";
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
        "قاعدة البصمات: {}\n",
        if report.database_present {
            if report.known_bad {
                "مطابقة معروفة الخبث"
            } else {
                "موجودة، بلا مطابقة"
            }
        } else {
            "غير موجودة بعد — لم يُقارن الملف بأي قاعدة"
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
        "ما يحدث عند انتهاء المهلة: {}\n",
        report.score.verdict.timeout_action()
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
            "\"database_present\":{db},",
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
        db = report.database_present,
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
    const EXPECTED: &str = "275a021bbfb6489e54d471899f7db9d1663fc695ec2fe2a2c4538aabf651fd0f";
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
            "الاختبار الذاتي: فشل — size_ok={size_ok} full_ok={full_ok} head_ok={head_ok} tail_ok={tail_ok}"
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
    fn reason_prefers_the_strongest_true_statement() {
        // A misleading name on an unremarkable path.
        let mut e = Evidence::new();
        e.set(Indicator::NamePattern, 0.45);
        let r = Report {
            path: PathBuf::from("/home/u/x.pdf.exe"),
            hashes: FileHashes::default(),
            database_present: false,
            known_bad: false,
            score: score_with_known_bad(&e, &Thresholds::default(), None, false),
            name_score: 0.45,
            path_score: 0.0,
        };
        assert_eq!(reason(&r), "اسمها مصمّم ليبدو غير ما هي");
    }
}
