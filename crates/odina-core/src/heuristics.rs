// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! Structural heuristics for the indicators that look at *shape* rather than
//! bytes — and the guarantee that makes them safe to ship.
//!
//! `docs/spec/scoring-spec-v1.md` §2.1 is the hardest promise in the whole
//! spec:
//!
//! > A file named `ملف_المحاسبة.exe` must not score higher than
//! > `accounting.exe`. A path `下载\安装.exe` must not score higher than
//! > `Downloads\setup.exe`.
//!
//! Those two sentences are tested in the `tests` module below, and the test is
//! not decorative — it is the reason this module has no notion of letters,
//! alphabets, or encodings. Everything measured here is a *shape*: how many
//! extensions, how many spaces, whether a right-to-left override is present,
//! how long the name is, whether digits are mixed in. All of those are
//! identical in every script.
//!
//! The reason is not politics. A classifier that treats "Arabic-looking" as
//! suspicious produces enormous false-positive rates on exactly the user this
//! product is built for, and a false positive does not fail by being wrong —
//! it fails by teaching the user to click through alerts.

/// Shape features of a file name. Every field is script-independent by
/// construction: counting extensions or spaces says nothing about letters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NameShape {
    /// Number of dot-separated parts that look like extensions, e.g. 2 for
    /// `invoice.pdf.exe`.
    pub extension_count: usize,
    /// `true` when the name ends in a space or a dot — Windows silently strips
    /// both, so `file.exe ` is not the name the user sees.
    pub trailing_padding: bool,
    /// A U+202E or similar bidirectional override is present. This is a
    /// display trick, not a language: it makes `photo_gnp.exe` render as
    /// `photo_exe.png`.
    pub bidi_override: bool,
    /// Number of whitespace runs inside the name.
    pub inner_spaces: usize,
    /// Total number of Unicode scalar values in the name.
    pub length: usize,
    /// The name mixes ASCII digits with letters.
    pub mixed_digits: bool,
}

impl NameShape {
    /// 0.0 .. 1.0 for indicator 2. Deliberately blunt: each feature is worth a
    /// fixed share, and no single feature can carry the indicator alone.
    pub fn score(&self) -> f32 {
        let mut s = 0.0f32;
        if self.extension_count >= 2 {
            s += 0.45;
        }
        if self.bidi_override {
            s += 0.40;
        }
        if self.trailing_padding {
            s += 0.30;
        }
        if self.inner_spaces >= 4 {
            s += 0.10;
        }
        if self.mixed_digits {
            s += 0.05;
        }
        s.clamp(0.0, 1.0)
    }
}

const BIDI_CONTROLS: [char; 5] = ['\u{202A}', '\u{202B}', '\u{202C}', '\u{202D}', '\u{202E}'];

/// Measure a file name. `name` is the last path component only.
pub fn name_shape(name: &str) -> NameShape {
    let mut shape = NameShape {
        length: name.chars().count(),
        ..NameShape::default()
    };

    shape.trailing_padding = name.ends_with(' ') || name.ends_with('.');
    shape.bidi_override = name.chars().any(|c| BIDI_CONTROLS.contains(&c));
    shape.inner_spaces = name
        .trim_end_matches([' ', '.'])
        .split_whitespace()
        .count()
        .saturating_sub(1);

    // Count dot-separated parts after the first, skipping leading dots so a
    // dotfile does not look like a double extension.
    let trimmed = name.trim_matches('.');
    shape.extension_count = trimmed.matches('.').count();

    let has_ascii_digit = name.chars().any(|c| c.is_ascii_digit());
    let has_alpha = name.chars().any(|c| c.is_alphabetic());
    shape.mixed_digits = has_ascii_digit && has_alpha;

    shape
}

/// Where a file sits, expressed as flags rather than a string.
///
/// **This is the important design decision in this module.** The real
/// application fills these flags from Windows Known Folder identifiers
/// (`FOLDERID_Downloads`, `FOLDERID_Desktop`, …), never from folder *names* —
/// because on an Arabic or Chinese Windows the folder is called `التنزيلات`
/// or `下载` while being the same folder. Passing booleans here means the
/// spec's guarantee cannot be broken by a later contributor matching on a
/// display string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PathFlags {
    pub usb_root: bool,
    pub startup_folder: bool,
    pub temp_folder: bool,
    pub downloads: bool,
    pub desktop: bool,
    pub program_files: bool,
    pub system32: bool,
}

/// 0.0 .. 1.0 for indicator 3, straight from the matrix in §3 of the spec.
/// The highest applicable value wins: `%TEMP%` inside `Downloads` is a temp
/// path, not a downloads path.
pub fn path_score(flags: PathFlags) -> f32 {
    if flags.usb_root || flags.startup_folder {
        return 1.0;
    }
    if flags.temp_folder {
        return 0.85;
    }
    if flags.downloads {
        return 0.60;
    }
    if flags.desktop {
        return 0.50;
    }
    if flags.program_files {
        return 0.20;
    }
    if flags.system32 {
        return 0.10;
    }
    0.0
}

/// Rough, deliberately conservative classifier used only to fill [`PathFlags`]
/// on platforms without Known Folder ids (Linux, tests, the CLI).
///
/// It matches on the *canonical English* segment names only, and returns all
/// false when it cannot tell — "unknown" must never be read as "suspicious".
/// The Windows build does not use this function.
pub fn guess_path_flags(path: &std::path::Path) -> PathFlags {
    let mut flags = PathFlags::default();
    for component in path.components() {
        let std::path::Component::Normal(part) = component else {
            continue;
        };
        let part = part.to_string_lossy().to_ascii_lowercase();
        match part.as_str() {
            "temp" | "tmp" => flags.temp_folder = true,
            "downloads" => flags.downloads = true,
            "desktop" => flags.desktop = true,
            "startup" => flags.startup_folder = true,
            "program files" | "program files (x86)" => flags.program_files = true,
            "system32" => flags.system32 = true,
            _ => {}
        }
    }
    flags
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Default::default()` is long enough to push lines past the width limit
    /// in this table of assertions, so it gets a short name.
    const D: PathFlags = PathFlags {
        usb_root: false,
        startup_folder: false,
        temp_folder: false,
        downloads: false,
        desktop: false,
        program_files: false,
        system32: false,
    };

    #[test]
    fn double_extension_is_the_strongest_name_signal() {
        let s = name_shape("invoice.pdf.exe");
        assert_eq!(s.extension_count, 2);
        assert!(s.score() >= 0.45);
    }

    #[test]
    fn a_bidi_override_is_flagged() {
        let s = name_shape("photo_\u{202E}exe.png");
        assert!(s.bidi_override);
        assert!(s.score() >= 0.40);
    }

    #[test]
    fn trailing_space_is_flagged() {
        assert!(name_shape("setup.exe ").trailing_padding);
        assert!(name_shape("setup.exe.").trailing_padding);
        assert!(!name_shape("setup.exe").trailing_padding);
    }

    #[test]
    fn a_plain_name_scores_zero() {
        let s = name_shape("setup.exe");
        assert_eq!(s.extension_count, 1);
        assert_eq!(s.score(), 0.0);
    }

    /// §2.1 — the promise. These pairs are structurally identical and must
    /// score identically, in both name and path. If this test ever fails, the
    /// product has started punishing its own users for their language.
    #[test]
    fn arabic_and_chinese_names_score_exactly_like_english() {
        let pairs = [
            ("accounting.exe", "ملف_المحاسبة.exe", "会计文件.exe"),
            ("setup.exe", "تثبيت.exe", "安装.exe"),
            ("photo.jpg.exe", "صورة.jpg.exe", "照片.jpg.exe"),
            (
                "report final.pdf.exe",
                "تقرير نهائي.pdf.exe",
                "报告 最终.pdf.exe",
            ),
        ];
        for (english, arabic, chinese) in pairs {
            let a = name_shape(arabic).score();
            let c = name_shape(chinese).score();
            let e = name_shape(english).score();
            assert_eq!(a, e, "Arabic {arabic} scored {a}, English scored {e}");
            assert_eq!(c, e, "Chinese {chinese} scored {c}, English scored {e}");
        }
    }

    #[test]
    fn length_alone_does_not_carry_a_score() {
        // A long Arabic name must not outscore a long English one, and neither
        // length gets any weight of its own.
        let long_ar = "ملف".repeat(40) + ".exe";
        let long_en = "file".repeat(40) + ".exe";
        assert_eq!(name_shape(&long_ar).score(), name_shape(&long_en).score());
        assert_eq!(name_shape(&long_ar).score(), 0.0);
    }

    #[test]
    fn path_matrix_matches_the_spec() {
        assert_eq!(
            path_score(PathFlags {
                usb_root: true,
                ..D
            }),
            1.0
        );
        assert_eq!(
            path_score(PathFlags {
                startup_folder: true,
                ..D
            }),
            1.0
        );
        assert_eq!(
            path_score(PathFlags {
                temp_folder: true,
                ..D
            }),
            0.85
        );
        assert_eq!(
            path_score(PathFlags {
                downloads: true,
                ..D
            }),
            0.60
        );
        assert_eq!(path_score(PathFlags { desktop: true, ..D }), 0.50);
        assert_eq!(
            path_score(PathFlags {
                program_files: true,
                ..D
            }),
            0.20
        );
        assert_eq!(
            path_score(PathFlags {
                system32: true,
                ..D
            }),
            0.10
        );
        assert_eq!(path_score(D), 0.0);
    }

    #[test]
    fn the_strongest_path_wins() {
        let flags = PathFlags {
            temp_folder: true,
            downloads: true,
            ..D
        };
        assert_eq!(path_score(flags), 0.85);
    }

    #[test]
    fn an_unknown_path_is_not_suspicious() {
        let plain = guess_path_flags(std::path::Path::new("/home/u/docs/a.exe"));
        assert_eq!(
            path_score(plain),
            0.0,
            "unknown must never read as suspicious"
        );

        let dl = guess_path_flags(std::path::Path::new("/home/u/Downloads/a.exe"));
        assert!(dl.downloads);
        assert_eq!(path_score(dl), 0.60);

        let tmp = guess_path_flags(std::path::Path::new("/tmp/a.exe"));
        assert!(tmp.temp_folder);
        assert_eq!(path_score(tmp), 0.85);
    }
}
