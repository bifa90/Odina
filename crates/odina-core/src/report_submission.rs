// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! The second allowed network use: a report the user chose to send (D118).
//!
//! §11.1 fixed the sixteen fields years before this module existed; this is the
//! enforcement of that list. Two things matter more than the fields:
//!
//! * **What must never leave** — the file's content, the full path, the user
//!   name, the locale of the operating system, anything identifying the machine.
//!   The fields here are patterns and digests, and a value that looks like a
//!   real path is refused rather than "cleaned", because cleaning leaks half of
//!   it and pretending otherwise is worse than refusing.
//! * **The same report twice in one session is refused by the client.** The
//!   counter the owner wants (`Repetition_Count`) lives in the sheet and rises
//!   with reports that arrive in *different* sessions; a user pressing "send"
//!   twice in a row must not look like two witnesses.
//!
//! A user report is still **not** an independent origin for admission (D109):
//! it needs a second origin, otherwise any single person could convict a file.

use std::collections::HashSet;

/// §11.1: at most this many reports per day from one installation.
pub const DAILY_CAP: usize = 10;
/// A report is a few hundred bytes; the ceiling keeps a runaway list of
/// behaviours from turning into an upload.
pub const MAX_PAYLOAD_BYTES: usize = 4096;
/// Longest accepted behaviour token, in bytes.
pub const MAX_TOKEN: usize = 64;

/// D109 stays as it was: a single user is not two origins.
pub const USER_REPORT_IS_INDEPENDENT_ORIGIN: bool = false;
/// And that is not a wish: the build fails if anyone flips it without reading
/// D109 and D119 first.
const _: () = assert!(!USER_REPORT_IS_INDEPENDENT_ORIGIN);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub sha256: String,
    pub size: u64,
    pub header_hash: String,
    pub tail_hash: String,
    pub pe_hash: String,
    pub tlsh: String,
    /// A *pattern* (e.g. `????????.exe`), never the real name: a file name can
    /// carry a person's name, a project name or a client name.
    pub name_pattern: String,
    /// A *pattern* (e.g. `%USERPROFILE%\\Downloads\\*.exe`), never a real path.
    pub path_pattern: String,
    pub attributes: u32,
    pub behavior_score: u8,
    pub behaviors: Vec<String>,
    /// The threat's own name — the only "name" this report carries.
    pub family: String,
    pub app_version: String,
    pub db_version: String,
    /// Unix seconds, UTC.
    pub ts: u64,
    /// §11.1: optional, anonymous, and not tied to any device identifier.
    pub ui_lang: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportReject {
    NotSha256,
    MissingFamily,
    MissingVersion,
    TooLarge,
    TooManyBehaviours(usize),
    LongToken,
    /// A field carries what looks like a real path, so the report is refused.
    PathLeak,
    /// A behaviour token carries a separator that would corrupt the payload.
    Separator,
}

fn is_digest(text: &str) -> bool {
    text.len() == 64 && text.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Drive letters, user-profile roots and UNC prefixes are the three shapes that
/// turn "pattern" back into "somebody's file".
fn looks_like_path(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    let drive = text.as_bytes().get(1) == Some(&b':');
    let has_drive = drive && text.as_bytes().first().is_some_and(u8::is_ascii_alphabetic);
    has_drive
        || lower.contains("\\users\\")
        || lower.contains("\\documents and settings\\")
        || lower.contains("/home/")
        || lower.contains("/users/")
        || text.starts_with("\\\\")
}

impl Report {
    /// The gate before the gate: nothing is rendered, encoded or shown to the
    /// user for approval until the report passes this.
    pub fn validate(&self) -> Result<(), ReportReject> {
        if !is_digest(&self.sha256) {
            return Err(ReportReject::NotSha256);
        }
        if self.family.trim().is_empty() {
            return Err(ReportReject::MissingFamily);
        }
        if self.app_version.trim().is_empty() || self.db_version.trim().is_empty() {
            return Err(ReportReject::MissingVersion);
        }
        if self.behaviors.len() > 16 {
            return Err(ReportReject::TooManyBehaviours(self.behaviors.len()));
        }
        for token in &self.behaviors {
            if token.len() > MAX_TOKEN {
                return Err(ReportReject::LongToken);
            }
            if token.contains(';') {
                return Err(ReportReject::Separator);
            }
        }
        for value in [&self.name_pattern, &self.path_pattern, &self.family] {
            if looks_like_path(value) {
                return Err(ReportReject::PathLeak);
            }
        }
        // The cap is on bytes, not on the number of fields: sixteen fields with
        // a megabyte in one of them is still a megabyte.
        let bytes: usize = self
            .to_form_fields()
            .iter()
            .map(|(key, value)| key.len() + value.len() + 2)
            .sum();
        if bytes > MAX_PAYLOAD_BYTES {
            return Err(ReportReject::TooLarge);
        }
        Ok(())
    }

    /// The exact pairs the form receives, in §11.1 order. This is also what the
    /// consent screen prints: the user approves the real payload, not a
    /// description of it.
    pub fn to_form_fields(&self) -> Vec<(String, String)> {
        let mut fields = vec![
            ("sha256".to_string(), self.sha256.clone()),
            ("size".to_string(), self.size.to_string()),
            ("header_hash".to_string(), self.header_hash.clone()),
            ("tail_hash".to_string(), self.tail_hash.clone()),
            ("pe_hash".to_string(), self.pe_hash.clone()),
            ("tlsh".to_string(), self.tlsh.clone()),
            ("name_pattern".to_string(), self.name_pattern.clone()),
            ("path_pattern".to_string(), self.path_pattern.clone()),
            ("attributes".to_string(), self.attributes.to_string()),
            (
                "behavior_score".to_string(),
                self.behavior_score.to_string(),
            ),
            ("behaviors".to_string(), self.behaviors.join(";")),
            ("family".to_string(), self.family.clone()),
            ("app_version".to_string(), self.app_version.clone()),
            ("db_version".to_string(), self.db_version.clone()),
            ("ts".to_string(), self.ts.to_string()),
        ];
        if let Some(lang) = &self.ui_lang {
            fields.push(("ui_lang".to_string(), lang.clone()));
        }
        fields
    }

    /// What the sheet keys repetition on: same file, same verdict, same size.
    pub fn fingerprint(&self) -> String {
        format!(
            "{}|{}|{}",
            self.sha256.to_ascii_lowercase(),
            self.size,
            self.family.trim().to_ascii_lowercase()
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Submit {
    /// Show the consent screen, then send.
    AskConsent,
    /// The same report was already sent from this session. Refused silently
    /// apart from a one-line note: the counter in the sheet must not be fed by
    /// a double click.
    DuplicateThisSession,
    /// `DAILY_CAP` reached; the button says "غدًا".
    DailyCapReached,
}

/// Per-session memory. Deliberately not persisted: another session is another
/// day's evidence, and the sheet's `Repetition_Count` is where that is counted.
#[derive(Debug, Clone, Default)]
pub struct SessionReports {
    seen: HashSet<String>,
    sent_today: usize,
}

impl SessionReports {
    pub fn new() -> Self {
        Self {
            seen: HashSet::new(),
            sent_today: 0,
        }
    }

    pub fn sent_today(&self) -> usize {
        self.sent_today
    }

    /// Decide, then record. A refused report never increments the cap.
    pub fn offer(&mut self, report: &Report) -> Submit {
        if self.sent_today >= DAILY_CAP {
            return Submit::DailyCapReached;
        }
        let fingerprint = report.fingerprint();
        if self.seen.contains(&fingerprint) {
            return Submit::DuplicateThisSession;
        }
        self.seen.insert(fingerprint);
        self.sent_today += 1;
        Submit::AskConsent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn report() -> Report {
        Report {
            sha256: DIGEST.to_string(),
            size: 68,
            header_hash: DIGEST.to_string(),
            tail_hash: DIGEST.to_string(),
            pe_hash: String::new(),
            tlsh: String::new(),
            name_pattern: "????????.exe".to_string(),
            path_pattern: "%USERPROFILE%\\Downloads\\*.exe".to_string(),
            attributes: 32,
            behavior_score: 90,
            behaviors: vec!["writes_autorun".to_string(), "drops_temp".to_string()],
            family: "eicar-test-file".to_string(),
            app_version: "0.1.0".to_string(),
            db_version: "7".to_string(),
            ts: 1_766_000_000,
            ui_lang: Some("ar".to_string()),
        }
    }

    #[test]
    fn the_sixteen_section_11_1_fields_are_the_payload_in_order() {
        let fields = report().to_form_fields();
        let names: Vec<&str> = fields.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(
            names,
            [
                "sha256",
                "size",
                "header_hash",
                "tail_hash",
                "pe_hash",
                "tlsh",
                "name_pattern",
                "path_pattern",
                "attributes",
                "behavior_score",
                "behaviors",
                "family",
                "app_version",
                "db_version",
                "ts",
                "ui_lang"
            ]
        );
        assert_eq!(report().validate(), Ok(()));
        let without_lang = Report {
            ui_lang: None,
            ..report()
        };
        assert_eq!(without_lang.to_form_fields().len(), 15);
        assert_eq!(without_lang.validate(), Ok(()));
        assert!(fields
            .iter()
            .any(|(k, v)| k == "behaviors" && v == "writes_autorun;drops_temp"));
    }

    #[test]
    fn a_real_path_is_refused_rather_than_cleaned() {
        for leak in [
            "C:\\Users\\ali\\Downloads\\a.exe".to_string(),
            "\\\\server\\share\\a.exe".to_string(),
            "/home/ali/a.exe".to_string(),
            "C:\\a.exe".to_string(),
        ] {
            let bad = Report {
                path_pattern: leak,
                ..report()
            };
            assert_eq!(bad.validate(), Err(ReportReject::PathLeak));
        }
        // The pattern form is what is expected and what passes.
        for good in ["%USERPROFILE%\\Downloads\\*.exe", "*.scr", "%TEMP%\\*"] {
            let ok = Report {
                path_pattern: good.to_string(),
                ..report()
            };
            assert_eq!(ok.validate(), Ok(()));
        }
        let named = Report {
            family: "C:\\evil.exe".to_string(),
            ..report()
        };
        assert_eq!(named.validate(), Err(ReportReject::PathLeak));
    }

    #[test]
    fn the_report_carries_the_threat_name_and_never_a_file_name() {
        let fields = report().to_form_fields();
        let keys: Vec<&str> = fields.iter().map(|(k, _)| k.as_str()).collect();
        assert!(keys.contains(&"family"));
        assert!(!keys.contains(&"file_name"));
        assert!(!keys.contains(&"path"));
        assert!(!keys.contains(&"username"));
        assert!(!keys
            .iter()
            .any(|k| k.contains("hwid") || k.contains("country")));
        assert_eq!(
            report().fingerprint(),
            format!("{DIGEST}|68|eicar-test-file")
        );
    }

    #[test]
    fn malformed_reports_are_refused_before_the_user_sees_a_consent_screen() {
        assert_eq!(
            Report {
                sha256: "zz".to_string(),
                ..report()
            }
            .validate(),
            Err(ReportReject::NotSha256)
        );
        assert_eq!(
            Report {
                family: "  ".to_string(),
                ..report()
            }
            .validate(),
            Err(ReportReject::MissingFamily)
        );
        assert_eq!(
            Report {
                db_version: String::new(),
                ..report()
            }
            .validate(),
            Err(ReportReject::MissingVersion)
        );
        assert_eq!(
            Report {
                behaviors: vec!["x".repeat(MAX_TOKEN + 1)],
                ..report()
            }
            .validate(),
            Err(ReportReject::LongToken)
        );
        assert_eq!(
            Report {
                behaviors: vec!["read;write".to_string()],
                ..report()
            }
            .validate(),
            Err(ReportReject::Separator)
        );
        let many: Vec<String> = (0..17).map(|i| format!("b{i}")).collect();
        assert_eq!(
            Report {
                behaviors: many,
                ..report()
            }
            .validate(),
            Err(ReportReject::TooManyBehaviours(17))
        );
        let long_name = Report {
            name_pattern: "a".repeat(MAX_PAYLOAD_BYTES),
            ..report()
        };
        assert_eq!(long_name.validate(), Err(ReportReject::TooLarge));
    }

    #[test]
    fn the_same_report_twice_in_one_session_is_one_witness() {
        let mut session = SessionReports::new();
        let report = report();
        assert_eq!(session.offer(&report), Submit::AskConsent);
        assert_eq!(session.offer(&report), Submit::DuplicateThisSession);
        assert_eq!(session.sent_today(), 1);
        // A different file is a different witness.
        let other = Report {
            sha256: "c".repeat(64),
            ..report.clone()
        };
        assert_eq!(session.offer(&other), Submit::AskConsent);
        assert_eq!(session.sent_today(), 2);
        // Same digest, different verdict = a new claim, and the sheet counts it.
        let reclassified = Report {
            family: "other-family".to_string(),
            ..report.clone()
        };
        assert_eq!(session.offer(&reclassified), Submit::AskConsent);
    }

    #[test]
    fn the_daily_cap_stops_at_ten_and_a_refusal_costs_nothing() {
        let mut session = SessionReports::new();
        for i in 0..DAILY_CAP {
            let fresh = Report {
                sha256: format!("{i:064x}"),
                ..report()
            };
            assert_eq!(session.offer(&fresh), Submit::AskConsent);
        }
        assert_eq!(session.sent_today(), DAILY_CAP);
        let eleventh = Report {
            sha256: "f".repeat(64),
            ..report()
        };
        assert_eq!(session.offer(&eleventh), Submit::DailyCapReached);
        assert_eq!(session.sent_today(), DAILY_CAP);
    }
}
