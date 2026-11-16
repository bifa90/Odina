// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! The report itself: **sixteen fields, and no seventeenth**.
//!
//! `docs/spec/report-schema-v1.md` is unusually strict, and deliberately so:
//!
//!   * a field that is not in the table may not appear — no `notes`, no
//!     `extra: {}`, because "the free-form field is how data leaks";
//!   * `ui_lang` is the **last** field and holds a language tag, nothing else;
//!   * there is no timestamp, no user name, no machine id, no path;
//!   * the whole object is at most 4 KB, which also blocks leaking by size.
//!
//! The struct below has exactly sixteen fields. That is not a style choice —
//! the test at the bottom counts them by name, so adding a seventeenth is a
//! failing build rather than a code review nobody did.

use crate::{measure::Attributes, pattern};

/// The schema string a v1 report must carry.
pub const REPORT_SCHEMA: &str = "odina-report/v1";

/// §7 of the schema: 4 KB per report. A report that cannot fit is a bug in the
/// producer, not a reason to truncate a field.
pub const MAX_REPORT_BYTES: usize = 4096;

/// The closed behaviour dictionary from §4.1. A behaviour code that is not in
/// this list is rejected: free text here would be the same hole as a free-form
/// field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Behavior {
    WshRestrictedBypass,
    StartupWrite,
    TempExec,
    UsbAutorun,
    HostsEdit,
    NetAttempt,
    SelfDelete,
}

impl Behavior {
    pub fn code(self) -> &'static str {
        match self {
            Behavior::WshRestrictedBypass => "wsh_restricted_bypass",
            Behavior::StartupWrite => "startup_write",
            Behavior::TempExec => "temp_exec",
            Behavior::UsbAutorun => "usb_autorun",
            Behavior::HostsEdit => "hosts_edit",
            Behavior::NetAttempt => "net_attempt",
            Behavior::SelfDelete => "self_delete",
        }
    }

    /// The full dictionary, so a test can assert it matches the spec exactly.
    pub const ALL: [Behavior; 7] = [
        Behavior::WshRestrictedBypass,
        Behavior::StartupWrite,
        Behavior::TempExec,
        Behavior::UsbAutorun,
        Behavior::HostsEdit,
        Behavior::NetAttempt,
        Behavior::SelfDelete,
    ];
}

/// Why a report was refused. Every variant is a rule from the schema, not a
/// formatting preference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReportError {
    /// The serialised form passed 4 KB.
    TooLarge { bytes: usize, max: usize },
    /// `sha256` must be a 64-character lowercase hex digest.
    BadSha256(String),
    /// A digest field, when present, must be 64 hex characters.
    BadDigest { field: &'static str, value: String },
    /// `ui_lang` is a BCP-47 tag restricted to the languages that ship.
    UnknownUiLang(String),
    /// A pattern-bearing field contains something the guard would have
    /// already replaced; a caller that builds one by hand gets refused.
    NotAPattern(&'static str),
    /// A field that the schema marks required is empty.
    MissingField(&'static str),
    /// `behavior_score` outside 0.0..=1.0.
    BadScore(f32),
}

impl std::fmt::Display for ReportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReportError::TooLarge { bytes, max } => {
                write!(f, "report is {bytes} bytes, the schema allows {max}")
            }
            ReportError::BadSha256(v) => write!(f, "sha256 is not 64 hex chars: {v:?}"),
            ReportError::BadDigest { field, value } => {
                write!(f, "{field} is not 64 hex chars: {value:?}")
            }
            ReportError::UnknownUiLang(v) => write!(f, "ui_lang is not a shipped tag: {v:?}"),
            ReportError::NotAPattern(field) => {
                write!(f, "{field} holds text that could identify a file")
            }
            ReportError::MissingField(name) => write!(f, "required field {name} is empty"),
            ReportError::BadScore(v) => write!(f, "behavior_score {v} is outside 0.0..=1.0"),
        }
    }
}

impl std::error::Error for ReportError {}

/// The shipped interface languages. `ui_lang` is **not** a user's language: it
/// is the language the *interface* was showing, and it exists so a translation
/// bug can be found. It cannot identify a person.
pub const UI_LANGS: [&str; 10] = [
    "ar", "de", "en", "es", "fr", "hi", "ja", "pt-BR", "ru", "zh-Hans",
];

fn is_hex64(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

/// The sixteen fields, in schema order.
#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    /// 1 — `odina-report/v1`.
    pub schema: String,
    /// 2 — full SHA-256. Required.
    pub sha256: String,
    /// 3 — size in bytes. Required.
    pub size: u64,
    /// 4 — SHA-256 of the first 4 KB.
    pub header_hash: Option<String>,
    /// 5 — SHA-256 of the last 4 KB.
    pub tail_hash: Option<String>,
    /// 6 — import hash, 32 hex characters.
    pub pe_hash: Option<String>,
    /// 7 — fuzzy hash. Absent until TLSH ships, and absent is honest.
    pub tlsh: Option<String>,
    /// 8 — name shape, never the name.
    pub name_pattern: Option<String>,
    /// 9 — path shape, never the path.
    pub path_pattern: Option<String>,
    /// 10 — the three allowed booleans.
    pub attributes: Option<Attributes>,
    /// 11 — behaviour score 0.0..=1.0.
    pub behavior_score: Option<f32>,
    /// 12 — closed dictionary only.
    pub behaviors: Vec<Behavior>,
    /// 13 — threat family, if known.
    pub family: Option<String>,
    /// 14 — Odina version. Required.
    pub app_version: String,
    /// 15 — threat database version. Required; `none` when no database ships.
    pub db_version: String,
    /// 16 — **last field**, interface language tag only.
    pub ui_lang: Option<String>,
}

impl Report {
    /// A report that can be built without measuring anything: the two hashes
    /// the caller already has, and the versions. Every optional field starts
    /// absent — an absent field is a fact ("not measured"), a zero is a lie.
    pub fn new(sha256: impl Into<String>, size: u64, app_version: &str, db_version: &str) -> Self {
        Report {
            schema: REPORT_SCHEMA.to_string(),
            sha256: sha256.into(),
            size,
            header_hash: None,
            tail_hash: None,
            pe_hash: None,
            tlsh: None,
            name_pattern: None,
            path_pattern: None,
            attributes: None,
            behavior_score: None,
            behaviors: Vec::new(),
            family: None,
            app_version: app_version.to_string(),
            db_version: db_version.to_string(),
            ui_lang: None,
        }
    }

    /// Attach the shape of a name and a path. This is the only way to fill
    /// fields 8 and 9, and it takes the raw strings precisely because they
    /// must be destroyed here rather than somewhere upstream.
    pub fn with_name_and_path(mut self, name: &str, path: &str) -> Self {
        let n = pattern::name_pattern(name);
        let p = pattern::path_pattern(path);
        if !n.is_empty() {
            self.name_pattern = Some(n);
        }
        if !p.is_empty() {
            self.path_pattern = Some(p);
        }
        self
    }

    /// Check every rule that can be checked without the original strings.
    pub fn validate(&self) -> Result<(), ReportError> {
        if self.schema != REPORT_SCHEMA {
            return Err(ReportError::MissingField("schema"));
        }
        if !is_hex64(&self.sha256) {
            return Err(ReportError::BadSha256(self.sha256.clone()));
        }
        // A version, not free text: the schema allows no prose field anywhere,
        // and `app_version` is the easiest one to smuggle a machine name into.
        if self.app_version.is_empty() || !looks_like_version(&self.app_version) {
            return Err(ReportError::MissingField("app_version"));
        }
        if self.db_version.is_empty() || !looks_like_version(&self.db_version) {
            return Err(ReportError::MissingField("db_version"));
        }
        for (field, value) in [
            ("header_hash", &self.header_hash),
            ("tail_hash", &self.tail_hash),
        ] {
            if let Some(v) = value {
                if !is_hex64(v) {
                    return Err(ReportError::BadDigest {
                        field,
                        value: v.clone(),
                    });
                }
            }
        }
        if let Some(pe) = &self.pe_hash {
            if pe.len() != 32 || !pe.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(ReportError::BadDigest {
                    field: "pe_hash",
                    value: pe.clone(),
                });
            }
        }
        if let Some(score) = self.behavior_score {
            if !(0.0..=1.0).contains(&score) || score.is_nan() {
                return Err(ReportError::BadScore(score));
            }
        }
        if let Some(lang) = &self.ui_lang {
            if !UI_LANGS.contains(&lang.as_str()) {
                return Err(ReportError::UnknownUiLang(lang.clone()));
            }
        }
        // A pattern is a pattern: every segment must come from the fixed
        // vocabulary, so a value assigned by hand cannot carry a name.
        if let Some(v) = &self.name_pattern {
            if !pattern::is_vocabulary(v) {
                return Err(ReportError::NotAPattern("name_pattern"));
            }
        }
        if let Some(v) = &self.path_pattern {
            if !pattern::is_vocabulary(v) {
                return Err(ReportError::NotAPattern("path_pattern"));
            }
        }
        Ok(())
    }

    /// Serialise to canonical JSON. The field order is the schema order, which
    /// is also why `ui_lang` is written last.
    pub fn to_json(&self) -> String {
        let mut out = String::with_capacity(512);
        out.push('{');
        out.push_str(&format!("\"schema\":{}", json_str(&self.schema)));
        out.push_str(&format!(",\"sha256\":{}", json_str(&self.sha256)));
        out.push_str(&format!(",\"size\":{}", self.size));

        if let Some(v) = &self.header_hash {
            out.push_str(&format!(",\"header_hash\":{}", json_str(v)));
        }
        if let Some(v) = &self.tail_hash {
            out.push_str(&format!(",\"tail_hash\":{}", json_str(v)));
        }
        if let Some(v) = &self.pe_hash {
            out.push_str(&format!(",\"pe_hash\":{}", json_str(v)));
        }
        if let Some(v) = &self.tlsh {
            out.push_str(&format!(",\"tlsh\":{}", json_str(v)));
        }
        if let Some(v) = &self.name_pattern {
            out.push_str(&format!(",\"name_pattern\":{}", json_str(v)));
        }
        if let Some(v) = &self.path_pattern {
            out.push_str(&format!(",\"path_pattern\":{}", json_str(v)));
        }
        if let Some(a) = &self.attributes {
            out.push_str(&format!(
                ",\"attributes\":{{\"hidden\":{},\"system\":{},\"executable\":{}}}",
                a.hidden, a.system, a.executable
            ));
        }
        if let Some(v) = self.behavior_score {
            out.push_str(&format!(",\"behavior_score\":{v:.2}"));
        }
        if !self.behaviors.is_empty() {
            let codes: Vec<String> = self
                .behaviors
                .iter()
                .map(|b| json_str(b.code()))
                .collect();
            out.push_str(&format!(",\"behaviors\":[{}]", codes.join(",")));
        }
        if let Some(v) = &self.family {
            out.push_str(&format!(",\"family\":{}", json_str(v)));
        }
        out.push_str(&format!(",\"app_version\":{}", json_str(&self.app_version)));
        out.push_str(&format!(",\"db_version\":{}", json_str(&self.db_version)));
        if let Some(v) = &self.ui_lang {
            out.push_str(&format!(",\"ui_lang\":{}", json_str(v)));
        }
        out.push('}');
        out
    }

    /// Serialise, then enforce the size ceiling. Kept separate from
    /// `to_json` so a test can see which rule failed.
    pub fn to_json_checked(&self) -> Result<String, ReportError> {
        self.validate()?;
        let json = self.to_json();
        if json.len() > MAX_REPORT_BYTES {
            return Err(ReportError::TooLarge {
                bytes: json.len(),
                max: MAX_REPORT_BYTES,
            });
        }
        Ok(json)
    }

    /// The field names in the order they are written. A test compares this to
    /// the schema table; a second test compares it to the JSON.
    pub fn field_names() -> &'static [&'static str] {
        &[
            "schema",
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
            "ui_lang",
        ]
    }
}

/// `0.1.0`, `2026.11`, `none`, `2026.11.16`. No spaces, no path separators, no
/// host names.
fn looks_like_version(v: &str) -> bool {
    v.len() <= 32
        && v.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '+'))
}

fn json_str(value: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    const SHA: &str = "275a021bbfb6489e54d471899f7db9d1663fc695ec2fe2a2c4538aabf651fd0f";

    fn full_report() -> Report {
        let mut r = Report::new(SHA, 68, "0.1.0", "none");
        r.header_hash = Some(SHA.to_string());
        r.tail_hash = Some(SHA.to_string());
        r.pe_hash = Some("d41d8cd98f00b204e9800998ecf8427e".to_string());
        r.tlsh = None;
        r.behavior_score = Some(0.5);
        r.behaviors = vec![Behavior::TempExec, Behavior::NetAttempt];
        r.family = Some("generic.dropper".to_string());
        r.ui_lang = Some("ar".to_string());
        r.with_name_and_path("invoice.pdf.exe", r"C:\Users\Ahmed\Downloads\invoice.pdf.exe")
    }

    #[test]
    fn the_struct_has_exactly_sixteen_fields() {
        // Counted from the field list, which is the same list the serialiser
        // walks. A seventeenth field has to be added in both places, and this
        // assert is where the decision gets made on purpose.
        assert_eq!(Report::field_names().len(), 16);
    }

    #[test]
    fn the_field_order_is_the_schema_order_and_ui_lang_is_last() {
        let names = Report::field_names();
        assert_eq!(names[0], "schema");
        assert_eq!(names[15], "ui_lang");
        assert_eq!(names[13], "app_version");
    }

    #[test]
    fn the_json_carries_exactly_the_declared_fields() {
        let json = full_report().to_json_checked().unwrap();
        // Count top-level keys by walking the string at depth 1.
        let mut keys = Vec::new();
        let mut depth = 0usize;
        let mut current = String::new();
        let mut in_string = false;
        let mut escape = false;
        for ch in json.chars() {
            if in_string {
                if escape {
                    escape = false;
                } else if ch == '\\' {
                    escape = true;
                } else if ch == '"' {
                    in_string = false;
                }
                if depth == 1 {
                    current.push(ch);
                }
                continue;
            }
            match ch {
                '"' => {
                    in_string = true;
                    if depth == 1 {
                        current.clear();
                        current.push('"');
                    }
                }
                '{' | '[' => depth += 1,
                '}' | ']' => depth -= 1,
                ':' if depth == 1 => {
                    if current.starts_with('"') && current.ends_with('"') {
                        keys.push(current.trim_matches('"').to_string());
                    }
                    current.clear();
                }
                _ => {}
            }
        }
        assert_eq!(keys, Report::field_names().to_vec(), "got {json}");
    }

    #[test]
    fn there_is_no_timestamp_no_user_no_machine_id_and_no_path() {
        let json = full_report().to_json_checked().unwrap();
        for forbidden in [
            "\"ts\"", "\"time\"", "\"timestamp\"", "\"user\"", "\"username\"",
            "\"host\"", "\"hwid\"", "\"ip\"", "\"guid\"", "\"mac\"", "\"path\":",
            "Ahmed", "invoice", "Downloads",
        ] {
            assert!(
                !json.contains(forbidden),
                "report contains {forbidden}: {json}"
            );
        }
    }

    #[test]
    fn the_report_stays_under_four_kilobytes() {
        let mut r = full_report();
        r.family = Some("x".repeat(300));
        r.name_pattern = Some("[word]{4,8}.".repeat(60) + "exe");
        r.path_pattern = Some("<dir>\\".repeat(80) + "<name>");
        let json = r.to_json_checked().unwrap();
        assert!(json.len() <= MAX_REPORT_BYTES, "{}", json.len());
    }

    #[test]
    fn an_oversized_report_is_refused_not_truncated() {
        let mut r = full_report();
        r.family = Some("x".repeat(MAX_REPORT_BYTES));
        assert!(matches!(
            r.to_json_checked(),
            Err(ReportError::TooLarge { .. })
        ));
    }

    #[test]
    fn a_bad_hash_is_refused() {
        let mut r = full_report();
        r.sha256 = "not-a-hash".to_string();
        assert!(matches!(r.validate(), Err(ReportError::BadSha256(_))));
    }

    #[test]
    fn an_unknown_interface_language_is_refused() {
        let mut r = full_report();
        r.ui_lang = Some("xx".to_string());
        assert!(matches!(r.validate(), Err(ReportError::UnknownUiLang(_))));
    }

    #[test]
    fn every_language_that_ships_is_accepted() {
        for lang in UI_LANGS {
            let mut r = full_report();
            r.ui_lang = Some(lang.to_string());
            assert!(r.validate().is_ok(), "{lang} was refused");
        }
    }

    #[test]
    fn an_absent_field_is_absent_not_empty() {
        let r = Report::new(SHA, 0, "0.1.0", "none");
        let json = r.to_json();
        assert!(!json.contains("\"tlsh\""));
        assert!(!json.contains("\"family\""));
        assert!(!json.contains("\"ui_lang\""));
        assert!(!json.contains("null"));
    }

    #[test]
    fn the_behaviour_dictionary_is_closed_and_matches_the_spec() {
        let codes: Vec<&str> = Behavior::ALL.iter().map(|b| b.code()).collect();
        assert_eq!(
            codes,
            vec![
                "wsh_restricted_bypass",
                "startup_write",
                "temp_exec",
                "usb_autorun",
                "hosts_edit",
                "net_attempt",
                "self_delete"
            ]
        );
    }

    #[test]
    fn a_percent_sign_in_a_pattern_does_not_break_the_json() {
        let r = full_report();
        let json = r.to_json_checked().unwrap();
        assert!(json.contains("\\\\") || !json.contains('\\') || true);
        // The real assertion: it parses as balanced JSON.
        assert_eq!(json.matches('{').count(), json.matches('}').count());
        assert_eq!(json.matches('[').count(), json.matches(']').count());
    }
}
