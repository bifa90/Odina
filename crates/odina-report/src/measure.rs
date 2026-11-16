// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! File attributes and the few metadata facts the report may carry —
//! `report-schema-v1.md` §2 field 10, and nothing else.
//!
//! The schema allows exactly three booleans: `hidden`, `system`,
//! `executable`. Not owner, not times, not ACLs, not the archive bit. This
//! module reads from `std::fs::Metadata` on every platform, with one rule: a
//! platform that cannot answer says `false`, because "I cannot see it" is not
//! "it is hidden".

/// The three attributes the schema allows. Nothing else is measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Attributes {
    pub hidden: bool,
    pub system: bool,
    pub executable: bool,
}

impl Attributes {
    /// Score for indicator 4: the fraction of suspicious attributes present.
    /// `system` alone is not suspicious; `system` **and** `hidden` together is
    /// the combination this indicator exists for.
    pub fn score(&self) -> f32 {
        let mut present = 0u32;
        if self.hidden {
            present += 1;
        }
        if self.system {
            present += 1;
        }
        // Executable is measured for the report but is far too common to be
        // evidence of anything on its own — the spec's 1.0 case is "hidden +
        // system + executable together".
        if self.executable && (self.hidden || self.system) {
            present += 1;
        }
        present as f32 / 3.0
    }
}

/// Read the three attributes. Unreadable metadata yields the default, not an
/// error: the report is not the place to fail a scan.
pub fn attributes(meta: &std::fs::Metadata) -> Attributes {
    let mut attrs = Attributes::default();

    #[cfg(windows)]
    {
        #[allow(unused_imports)]
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
        const FILE_ATTRIBUTE_SYSTEM: u32 = 0x4;
        let bits = meta.file_attributes();
        attrs.hidden = bits & FILE_ATTRIBUTE_HIDDEN != 0;
        attrs.system = bits & FILE_ATTRIBUTE_SYSTEM != 0;
        // "Executable" on Windows is an extension, not a bit; the caller knows
        // the name and sets it. Leaving it false keeps this function honest
        // about what it can actually see.
    }

    #[cfg(unix)]
    {
        #[allow(unused_imports)]
        use std::os::unix::fs::MetadataExt;
        // The executable bit is real on Unix. There is no `system` bit and no
        // hidden bit; a dotfile is the Unix spelling of "hidden", and the
        // caller decides that from the name.
        attrs.executable = meta.mode() & 0o111 != 0;
    }

    attrs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_attributes_no_score() {
        assert_eq!(Attributes::default().score(), 0.0);
    }

    #[test]
    fn hidden_plus_system_is_the_strongest_pair() {
        let a = Attributes {
            hidden: true,
            system: true,
            executable: false,
        };
        assert!((a.score() - 2.0 / 3.0).abs() < 1e-6);
    }

    #[test]
    fn executable_alone_is_not_evidence() {
        let a = Attributes {
            hidden: false,
            system: false,
            executable: true,
        };
        assert_eq!(a.score(), 0.0);
    }

    #[test]
    fn all_three_together_score_one() {
        let a = Attributes {
            hidden: true,
            system: true,
            executable: true,
        };
        assert_eq!(a.score(), 1.0);
    }

    #[test]
    fn a_dotfile_on_this_platform_is_read_without_panicking() {
        let dir = std::env::temp_dir().join("odina-report-attrs");
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join(".hidden");
        std::fs::write(&file, b"x").unwrap();
        let meta = std::fs::metadata(&file).unwrap();
        let _ = attributes(&meta);
        let _ = std::fs::remove_file(&file);
    }
}
