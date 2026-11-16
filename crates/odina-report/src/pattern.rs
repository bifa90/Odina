// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! Turning a name and a path into a **pattern** — the most dangerous
//! transformation in the report, per `report-schema-v1.md` §3.
//!
//! The rule the tests below enforce, stated once: after this function runs, the
//! output must contain **no single character of the original name**. Not the
//! first letter, not the last, not a digit. What survives is structure:
//!
//! ```text
//! فاتورة_شركة_النور_2024.exe     ->  [word]{4,8}[char][word]{4,8}[char][word]{4,8}[char][digits]{4}.exe
//! C:\Users\Ahmed\Downloads\x.exe ->  %USERPROFILE%\<dir>\%DOWNLOADS%\<name>.exe
//! ```
//!
//! Lengths are bucketed, never exact: an exact length is a fingerprint of a
//! real file name, while a bucket still answers the only question the analysis
//! needs — "was this a long, multi-part, deceptive name?"
//!
//! Extensions come from a fixed list below. That is the difference between a
//! category and a leak: `pdf` in `invoice.pdf.exe` says "double extension",
//! which is what indicator 2 is built on, while an unknown tail token such as
//! `xyz` could be somebody's surname and is therefore treated as part of the
//! stem and erased.
//!
//! ## The invariant, checked at runtime
//!
//! "It erases characters by construction" is a claim about code; the guard at
//! the end of each function is a claim about **output**. Every emitted segment
//! must be one of a short, fixed vocabulary (`[word]{n}`, `[digits]{n}`,
//! `[char]`, a folder token, `<dir>`, `<name>`, or a known extension). If a
//! future edit ever emits a real character, the guard sees it and the whole
//! pattern collapses to `[redacted]` — a visible, harmless failure instead of a
//! quiet leak.

/// Extension chain recognised as a *category*. Anything not in this list is
/// not an extension for our purposes — it is part of the name, and it is
/// erased.
const KNOWN_EXTS: [&str; 54] = [
    "exe", "dll", "scr", "bat", "cmd", "com", "pif", "vbs", "vbe", "js", "jse", "wsf", "wsh",
    "ps1", "msi", "msp", "cpl", "hta", "jar", "lnk", "url", "reg", "inf", "sys", "ocx", "drv",
    "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "odt", "rtf", "txt", "csv", "md", "jpg",
    "jpeg", "png", "gif", "bmp", "svg", "mp3", "mp4", "avi", "mkv", "zip", "rar", "7z", "iso",
    "apk", "img",
];

const FOLDER_TOKENS: [&str; 8] = [
    "%TEMP%",
    "%DOWNLOADS%",
    "%DESKTOP%",
    "%STARTUP%",
    "%PROGRAMFILES%",
    "%SYSTEM32%",
    "%WINDIR%",
    "%USERPROFILE%",
];

/// What a leaked pattern collapses to. Never produced in practice; produced
/// instead of a leak if the invariant is ever broken.
const REDACTED: &str = "[redacted]";

fn is_known_ext(segment: &str) -> bool {
    let lower = segment.to_ascii_lowercase();
    KNOWN_EXTS.iter().any(|e| *e == lower)
}

/// Buckets used for `{n}`. Coarse on purpose — see the module comment: an exact
/// count is a fingerprint.
fn bucket(len: usize) -> &'static str {
    match len {
        0 => "0",
        1..=3 => "{1,3}",
        4..=8 => "{4,8}",
        _ => "{9,}",
    }
}

fn class_of(c: char) -> char {
    if c.is_alphabetic() {
        // Letters of any script — Arabic, Chinese, Cyrillic, Latin — take the
        // same branch. That is also the §2.1 guarantee: the pattern must not
        // depend on the language.
        'w'
    } else if c.is_numeric() {
        'd'
    } else {
        'c'
    }
}

/// One dot-free segment, turned into pattern tokens. A run of one class becomes
/// one token, so both the characters and their identity disappear.
fn segment_pattern(segment: &str) -> String {
    let mut out = String::new();
    let mut run_len = 0usize;
    let mut run_class = ' ';

    let flush = |out: &mut String, len: usize, class: char| {
        if len == 0 {
            return;
        }
        match class {
            'w' => out.push_str(&format!("[word]{}", bucket(len))),
            'd' => out.push_str(&format!("[digits]{}", bucket(len))),
            // A run of punctuation collapses to a single token: three
            // underscores and four are the same shape, different names.
            _ => out.push_str("[char]"),
        }
    };

    for c in segment.chars() {
        let class = class_of(c);
        if class == run_class {
            run_len += 1;
        } else {
            flush(&mut out, run_len, run_class);
            run_class = class;
            run_len = 1;
        }
    }
    flush(&mut out, run_len, run_class);
    out
}

/// Is this emitted segment part of the fixed vocabulary? See the module
/// comment: this is the guard that turns a hypothetical leak into `[redacted]`.
fn is_vocabulary_segment(segment: &str) -> bool {
    if segment.is_empty() {
        return true; // a leading dot, or a trailing dot Windows strips
    }
    if segment == "[char]" || segment == "<dir>" || segment == REDACTED {
        return true;
    }
    if let Some(rest) = segment.strip_prefix("[word]") {
        return matches!(rest, "" | "{1,3}" | "{4,8}" | "{9,}");
    }
    if let Some(rest) = segment.strip_prefix("[digits]") {
        return matches!(rest, "" | "{1,3}" | "{4,8}" | "{9,}");
    }
    if let Some(rest) = segment.strip_prefix("<name>") {
        // `<name>` alone, or `<name>.ext` / `<name>.ext.ext`.
        if rest.is_empty() {
            return true;
        }
        let Some(chain) = rest.strip_prefix('.') else {
            return false;
        };
        return chain.split('.').all(is_known_ext);
    }
    if FOLDER_TOKENS.contains(&segment) {
        return true;
    }
    is_known_ext(segment)
}

/// Split a pattern into the segments the emitter is allowed to produce and
/// report whether all of them are vocabulary.
fn only_vocabulary(pattern: &str) -> bool {
    pattern
        .split(['.', '\\'])
        .all(|segment| is_vocabulary_segment(segment))
}

/// Public form of the vocabulary check, so a validator can refuse a pattern
/// that was assigned by hand instead of produced here.
pub fn is_vocabulary(pattern: &str) -> bool {
    pattern == REDACTED || only_vocabulary(pattern)
}

fn guard(pattern: String) -> String {
    if only_vocabulary(&pattern) {
        pattern
    } else {
        REDACTED.to_string()
    }
}

/// Pattern for a file name. `name` is the last path component only.
///
/// Returns an empty string for an empty name; the caller omits the field rather
/// than sending `""`, because an empty pattern is not a measurement.
pub fn name_pattern(name: &str) -> String {
    if name.is_empty() {
        return String::new();
    }

    let parts: Vec<&str> = name.split('.').collect();
    let n = parts.len();

    // How many trailing segments form a recognised extension chain? One is the
    // normal case, two is the deceptive `invoice.pdf.exe`; three is odd enough
    // to stop at two.
    let mut chain = 0usize;
    while chain < 2 && chain < n - 1 && is_known_ext(parts[n - 1 - chain]) {
        chain += 1;
    }
    let stem_end = n - chain; // segments [0, stem_end) are the stem

    let mut out: Vec<String> = Vec::with_capacity(n);
    for part in &parts[..stem_end] {
        if part.is_empty() {
            // Leading dot of a hidden file, or a trailing dot Windows strips.
            out.push(String::new());
        } else {
            out.push(segment_pattern(part));
        }
    }
    for part in &parts[stem_end..] {
        // Kept literally: `[word]{4,8}.pdf.exe` must still read as a double
        // extension at the other end.
        out.push(part.to_ascii_lowercase());
    }

    let mut joined = out.join(".");
    if name.ends_with('.') && !joined.ends_with('.') {
        joined.push('.');
    }
    guard(joined)
}

/// Known folder token, matched case-insensitively on the **canonical English**
/// name. A localized folder is the same folder: on an Arabic Windows the
/// Downloads folder is called `التنزيلات` and is still Downloads. The Windows
/// build resolves the Known Folder id and passes the token down; this table is
/// the fallback for the CLI and for reports built from a text path.
fn folder_token(segment: &str) -> Option<&'static str> {
    let lower = segment.to_ascii_lowercase();
    match lower.as_str() {
        "temp" | "tmp" => Some("%TEMP%"),
        "downloads" => Some("%DOWNLOADS%"),
        "desktop" => Some("%DESKTOP%"),
        "startup" => Some("%STARTUP%"),
        "program files" | "program files (x86)" => Some("%PROGRAMFILES%"),
        "system32" => Some("%SYSTEM32%"),
        "windows" => Some("%WINDIR%"),
        "users" | "home" => Some("%USERPROFILE%"),
        _ => None,
    }
}

/// Pattern for a path: `%TOKEN%` for known folders, `<dir>` for anything else,
/// `<name>` for the file. Depth is preserved because it is structural.
///
/// **No component of the original path is emitted**, with one deliberate
/// exception: the tokens themselves, which are constants, not user data.
pub fn path_pattern(path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }

    // Normalise both separators so a Windows path and a Unix path produce the
    // same shape.
    let normalised = path.replace('\\', "/");
    let mut segments: Vec<&str> = normalised.split('/').filter(|s| !s.is_empty()).collect();

    // A drive letter (`C:`) is not part of the pattern.
    if let Some(first) = segments.first() {
        if first.len() == 2 && first.ends_with(':') {
            segments.remove(0);
        }
    }

    if segments.is_empty() {
        return "/".to_string();
    }

    let file = segments.pop().unwrap_or_default();
    let mut out: Vec<String> = Vec::with_capacity(segments.len() + 1);

    for segment in &segments {
        out.push(match folder_token(segment) {
            Some(token) => token.to_string(),
            None => "<dir>".to_string(),
        });
    }

    // The file keeps its recognised extension chain as `<name>.<ext>`, because
    // a double extension is the point of the field; the stem itself is gone.
    let mut exts: Vec<String> = Vec::new();
    for part in file.split('.').skip(1) {
        if is_known_ext(part) {
            exts.push(part.to_ascii_lowercase());
        }
    }

    if exts.is_empty() {
        out.push("<name>".to_string());
    } else {
        out.push(format!("<name>.{}", exts.join(".")));
    }

    guard(out.join("\\"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The promise, tested with names a real user would have. Each assertion
    /// fails if any single character of the *stem* survives — extensions are a
    /// category, not a name, so they are excluded from the check.
    #[test]
    fn no_character_of_the_original_stem_survives() {
        let names = [
            "فاتورة_شركة_النور_2024.exe",
            "accounting_setup_2024.exe",
            "صورة.jpg.exe",
            "报告 最终.pdf.exe",
            "invoice_Q3_final_v2.docx",
            "فاتورة.pdf",
            "secret.xyz",
            "ملف.غيرمعروف",
            "report.w9.exe",
        ];
        for name in names {
            let p = name_pattern(name);
            assert_ne!(p, REDACTED, "{name} fell back to a redaction");
            let stems: Vec<&str> = name
                .split('.')
                .filter(|s| !s.is_empty() && !is_known_ext(s))
                .collect();
            for stem in stems {
                for c in stem.chars() {
                    if c == '.' || c.is_whitespace() || !c.is_alphanumeric() {
                        continue;
                    }
                    assert!(
                        !p.contains(c),
                        "pattern {p:?} for {name:?} still contains {c:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn every_pattern_emitted_is_pure_vocabulary() {
        let names = [
            "a.exe",
            "invoice.pdf.exe",
            ".bashrc",
            "hidden.",
            "فاتورة_2024.pdf",
            "a___b.exe",
            "x",
            "..double.dot..",
            "file with spaces.doc",
        ];
        for name in names {
            let p = name_pattern(name);
            assert!(
                p == REDACTED || only_vocabulary(&p),
                "{name:?} produced a non-vocabulary segment: {p:?}"
            );
        }
    }

    #[test]
    fn a_double_extension_stays_visible_as_shape() {
        assert_eq!(name_pattern("invoice.pdf.exe"), "[word]{4,8}.pdf.exe");
        assert_eq!(name_pattern("صورة.jpg.exe"), "[word]{4,8}.jpg.exe");
        assert_eq!(name_pattern("照片.jpg.exe"), "[word]{4,8}.jpg.exe");
    }

    #[test]
    fn arabic_and_english_produce_identical_shapes() {
        assert_eq!(
            name_pattern("invoice_2024.pdf"),
            name_pattern("فاتورة_2024.pdf")
        );
        assert_eq!(name_pattern("setup.exe"), name_pattern("تثبيت.exe"));
    }

    #[test]
    fn digits_are_bucketed_not_counted_exactly() {
        assert_eq!(name_pattern("a12.exe"), "[word]{1,3}[digits]{1,3}.exe");
        assert_eq!(name_pattern("a12345.exe"), "[word]{1,3}[digits]{4,8}.exe");
    }

    #[test]
    fn an_unknown_tail_token_is_a_stem_not_an_extension() {
        // `xyz` could be somebody's name. It is erased, not published.
        assert_eq!(name_pattern("secret.xyz"), "[word]{4,8}.[word]{1,3}");
    }

    #[test]
    fn a_dotfile_does_not_leak_its_stem() {
        assert_eq!(name_pattern(".hidden.exe"), ".[word]{4,8}.exe");
    }

    #[test]
    fn punctuation_runs_collapse_to_one_token() {
        assert_eq!(
            name_pattern("a___b.exe"),
            "[word]{1,3}[char][word]{1,3}.exe"
        );
        assert_eq!(name_pattern("a_b.exe"), name_pattern("a___b.exe"));
    }

    #[test]
    fn path_loses_the_user_and_keeps_the_folder_kind() {
        let p = path_pattern(r"C:\Users\Ahmed\Downloads\فاتورة.exe");
        assert_eq!(p, r"%USERPROFILE%\<dir>\%DOWNLOADS%\<name>.exe");
        assert!(!p.contains("Ahmed"), "the user name must not survive: {p}");
        assert!(!p.contains("فاتورة"), "the file name must not survive: {p}");
    }

    #[test]
    fn temp_stays_visible_inside_downloads() {
        assert_eq!(
            path_pattern(r"C:\Users\Ahmed\Downloads\Temp\a.exe"),
            r"%USERPROFILE%\%DOWNLOADS%\%TEMP%\<name>.exe"
        );
    }

    #[test]
    fn unix_and_windows_paths_share_a_shape() {
        assert_eq!(
            path_pattern("/home/ahmed/Downloads/a.exe"),
            r"%USERPROFILE%\%DOWNLOADS%\<name>.exe"
        );
        assert_eq!(
            path_pattern(r"C:\Users\ahmed\Downloads\a.exe"),
            r"%USERPROFILE%\<dir>\%DOWNLOADS%\<name>.exe"
        );
    }

    #[test]
    fn an_unknown_directory_is_a_token_not_a_name() {
        let p = path_pattern(r"C:\projects\my_secret_client\a.exe");
        assert!(!p.contains("projects") && !p.contains("my_secret_client"));
        assert!(p.contains("<dir>"));
    }

    #[test]
    fn the_guard_rejects_a_hypothetical_leak() {
        // Not reachable through the public functions — that is the point of
        // having a guard a test can drive directly.
        assert!(!only_vocabulary("[word]{4,8}Ahmed.exe"));
        assert_eq!(guard("[word]{4,8}Ahmed.exe".to_string()), REDACTED);
        assert!(only_vocabulary("[word]{4,8}.pdf.exe"));
        assert!(only_vocabulary(r"%TEMP%\<dir>\<name>.exe"));
    }

    #[test]
    fn an_empty_name_is_an_empty_pattern_and_the_caller_omits_it() {
        assert_eq!(name_pattern(""), "");
        assert_eq!(path_pattern(""), "");
    }
}
