// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! The update check: what the sheet says, and what the program may ask it (D117).
//!
//! The owner's decision replaced "no network at all" with exactly two allowed
//! uses, and this module is the contract for the first one: the program reads a
//! **published, read-only Google Sheet** holding the release manifest and tells
//! the user whether a newer version exists. Nothing else.
//!
//! Rules that live here rather than in the caller, because they are the
//! difference between a version check and a beacon:
//!
//! * **One request, no payload.** A bare `GET` of a fixed `...output=csv` URL.
//!   No query string, no version, no device id, no language. The request itself
//!   shows the source IP to Google; we add nothing above that.
//! * **Once a month at most**, measured in months like the age badge.
//! * **The sheet is data, not instructions**: every row is validated (scheme,
//!   host, digest shape, month shape) and an unreadable sheet is silence, never
//!   an error dialog and never a downgrade.
//! * **No automatic download and no automatic install.** The check only answers
//!   "is there something newer"; the transfer is the user's, in the browser.
//!
//! The sheet's published CSV must start with this exact header row:
//! `version,release_month,installer_url,installer_sha256,db_generation,db_bytes,db_sha256`

use crate::release_age::ReleaseMonth;

/// The one host and path prefix a manifest row may point at.
pub const DOWNLOAD_PREFIX: &str = "https://github.com/bifa90/Odina/releases/download/";
/// A manifest is a handful of rows; this is the ceiling that keeps a hostile or
/// broken response from being read into memory.
pub const MAX_RESPONSE_BYTES: usize = 64 * 1024;
/// Minimum months between two checks. One, because the reminder is monthly.
pub const CHECK_INTERVAL_MONTHS: u32 = 1;
/// The request is a bare GET: no body, no custom headers, no version string.
pub const REQUEST_METHOD: &str = "GET";
/// User agent deliberately carries no version and no machine hint.
pub const USER_AGENT: &str = "Odina-update-check";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    /// `1.2.3` and `1.2.3-check.1` both parse; the suffix is a build label and
    /// never changes which version is newer.
    pub fn parse(text: &str) -> Option<Self> {
        let core = text.split(['-', '+']).next()?;
        let mut parts = core.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;
        let patch = parts.next()?.parse().ok()?;
        if parts.next().is_some() {
            return None;
        }
        Some(Self {
            major,
            minor,
            patch,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SheetReject {
    Header,
    Columns,
    Version,
    Month,
    Url,
    Digest,
    Generation,
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseRow {
    pub version: Version,
    pub month: ReleaseMonth,
    pub installer_url: String,
    pub installer_sha256: String,
    pub db_generation: u64,
    pub db_bytes: u64,
    pub db_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseSheet {
    rows: Vec<ReleaseRow>,
}

fn field(text: &str) -> &str {
    let trimmed = text.trim();
    // `strip_prefix`/`strip_suffix` rather than byte slicing: a quoted field may
    // hold Arabic text, and slicing by byte index would split a character.
    trimmed
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or(trimmed)
}

fn is_digest(text: &str) -> bool {
    text.len() == 64 && text.bytes().all(|b| b.is_ascii_hexdigit())
}

impl ReleaseSheet {
    pub fn parse(csv: &str) -> Result<Self, SheetReject> {
        const HEADER: &str =
            "version,release_month,installer_url,installer_sha256,db_generation,db_bytes,db_sha256";
        let mut lines = csv.lines().filter(|line| !line.trim().is_empty());
        let header = lines.next().ok_or(SheetReject::Empty)?;
        let header = header.trim_start_matches('\u{feff}').trim();
        if header != HEADER {
            return Err(SheetReject::Header);
        }
        let mut rows = Vec::new();
        for line in lines {
            let columns: Vec<&str> = line.split(',').map(field).collect();
            if columns.len() != 7 {
                return Err(SheetReject::Columns);
            }
            let version = Version::parse(columns[0]).ok_or(SheetReject::Version)?;
            let month = ReleaseMonth::parse(columns[1]).map_err(|_| SheetReject::Month)?;
            // The download must come from our own release page, over https,
            // and the digest must be present: a row we cannot verify is a row
            // we do not show.
            let url = columns[2];
            if !url.starts_with(DOWNLOAD_PREFIX) || url.contains(' ') {
                return Err(SheetReject::Url);
            }
            if !is_digest(columns[3]) || !is_digest(columns[6]) {
                return Err(SheetReject::Digest);
            }
            let generation = columns[4]
                .parse::<u64>()
                .map_err(|_| SheetReject::Generation)?;
            let bytes = columns[5]
                .parse::<u64>()
                .map_err(|_| SheetReject::Generation)?;
            rows.push(ReleaseRow {
                version,
                month,
                installer_url: url.to_string(),
                installer_sha256: columns[3].to_string(),
                db_generation: generation,
                db_bytes: bytes,
                db_sha256: columns[6].to_string(),
            });
        }
        if rows.is_empty() {
            return Err(SheetReject::Empty);
        }
        rows.sort_by(|a, b| b.version.cmp(&a.version));
        Ok(Self { rows })
    }

    /// The newest row, which is the only one the check reasons about.
    pub fn latest(&self) -> Option<&ReleaseRow> {
        self.rows.first()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateDecision {
    /// Present the installed version as current; say nothing.
    UpToDate,
    /// A newer version exists. The message names it and its month.
    Available {
        version: Version,
        month: ReleaseMonth,
    },
    /// The sheet was read but said nothing usable. Silence, not a warning:
    /// a broken sheet is our failure, not the user's problem.
    Unreadable,
}

/// Decide from a verified sheet. A sheet that failed to parse is `Unreadable`
/// and never `UpToDate` — "we could not tell" and "you are current" are
/// different answers and the caller must be able to distinguish them.
pub fn decide(local: Version, sheet: Result<&ReleaseSheet, SheetReject>) -> UpdateDecision {
    match sheet {
        Ok(sheet) => match sheet.latest() {
            Some(row) if row.version > local => UpdateDecision::Available {
                version: row.version,
                month: row.month,
            },
            Some(_) => UpdateDecision::UpToDate,
            None => UpdateDecision::Unreadable,
        },
        Err(_) => UpdateDecision::Unreadable,
    }
}

/// The installed version, from the package metadata — the same number the
/// release sheet will carry.
pub fn installed_version() -> Option<Version> {
    Version::parse(env!("CARGO_PKG_VERSION"))
}

/// At most one check per month, and never twice in the same month even if the
/// user keeps the program open for weeks.
pub fn should_check(last: Option<ReleaseMonth>, now: ReleaseMonth) -> bool {
    match last {
        None => true,
        Some(last) => last
            .months_until(now)
            .is_some_and(|months| months >= CHECK_INTERVAL_MONTHS),
    }
}

/// The sentence shown before a large transfer — the installer or the database,
/// not the version check (a check is a few kilobytes and needs no warning).
pub fn download_notice(bytes: u64, metered: bool) -> String {
    let mib = bytes.div_ceil(1024 * 1024);
    if metered {
        format!("هذا التنزيل {mib} ميغابايت على اتصال محدود. أجّله أو أكمل بقرارك.")
    } else {
        format!("هذا التنزيل {mib} ميغابايت.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::release_age::ReleaseMonth;

    const HEADER: &str =
        "version,release_month,installer_url,installer_sha256,db_generation,db_bytes,db_sha256";
    const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn row(version: &str, month: &str) -> String {
        format!(
            "{version},{month},{}Odina.zip,{DIGEST},7,94371840,{DIGEST}",
            DOWNLOAD_PREFIX
        )
    }

    fn sheet(rows: &[&str]) -> Result<ReleaseSheet, SheetReject> {
        let mut csv = String::from(HEADER);
        for row in rows {
            csv.push('\n');
            csv.push_str(row);
        }
        ReleaseSheet::parse(&csv)
    }

    fn m(text: &str) -> ReleaseMonth {
        ReleaseMonth::parse(text).unwrap()
    }

    #[test]
    fn a_version_is_three_numbers_and_the_build_suffix_is_ignored() {
        assert_eq!(
            Version::parse("0.2.1"),
            Some(Version {
                major: 0,
                minor: 2,
                patch: 1
            })
        );
        assert_eq!(
            Version::parse("1.0.0-check.2"),
            Some(Version {
                major: 1,
                minor: 0,
                patch: 0
            })
        );
        assert_eq!(Version::parse("1.0"), None);
        assert_eq!(Version::parse("1.0.0.0"), None);
        assert_eq!(Version::parse("v1.0.0"), None);
        assert!(Version::parse("0.10.0") > Version::parse("0.9.0"));
    }

    #[test]
    fn the_sheet_needs_its_exact_header_and_seven_verified_columns() {
        assert_eq!(sheet(&[&row("0.2.0", "2026-12")]).unwrap().len(), 1);
        assert_eq!(ReleaseSheet::parse(""), Err(SheetReject::Empty));
        assert_eq!(
            ReleaseSheet::parse("version,month\n"),
            Err(SheetReject::Header)
        );
        // A row that cannot be verified is refused as a whole sheet.
        for bad in [
            row("0.2", "2026-12"),
            row("0.2.0", "2026-13"),
            row("0.2.0", "2026-12").replace(DOWNLOAD_PREFIX, "http://example.com/"),
            row("0.2.0", "2026-12").replace(
                "https://github.com/bifa90/Odina/releases/download/",
                "https://evil.example/",
            ),
            row("0.2.0", "2026-12").replace(DIGEST, "nothex"),
            row("0.2.0", "2026-12").replace(",7,94371840,", ",x,94371840,"),
        ] {
            assert!(sheet(&[&bad]).is_err(), "{bad}");
        }
        // Quoted fields are tolerated because a published sheet may quote them.
        let quoted = format!("\"{}\"", DIGEST);
        let csv = format!(
            "{HEADER}\n0.2.0,2026-12,{}Odina.zip,{quoted},7,1,{quoted}\n",
            DOWNLOAD_PREFIX
        );
        assert!(ReleaseSheet::parse(&csv).is_ok());
    }

    #[test]
    fn the_newest_row_wins_and_an_unreadable_sheet_is_not_up_to_date() {
        let parsed = sheet(&[
            &row("0.1.0", "2026-11"),
            &row("0.3.0", "2027-01"),
            &row("0.2.0", "2026-12"),
        ])
        .unwrap();
        assert_eq!(
            parsed.latest().unwrap().version,
            Version {
                major: 0,
                minor: 3,
                patch: 0
            }
        );
        assert_eq!(
            decide(Version::parse("0.1.0").unwrap(), Ok(&parsed)),
            UpdateDecision::Available {
                version: Version {
                    major: 0,
                    minor: 3,
                    patch: 0
                },
                month: m("2027-01")
            }
        );
        assert_eq!(
            decide(Version::parse("0.3.0").unwrap(), Ok(&parsed)),
            UpdateDecision::UpToDate
        );
        assert_eq!(
            decide(Version::parse("0.1.0").unwrap(), Err(SheetReject::Header)),
            UpdateDecision::Unreadable
        );
        let empty = ReleaseSheet { rows: Vec::new() };
        assert_eq!(
            decide(Version::parse("0.1.0").unwrap(), Ok(&empty)),
            UpdateDecision::Unreadable
        );
    }

    #[test]
    fn the_installed_version_comes_from_the_package_not_from_a_file() {
        let installed = installed_version().expect("CARGO_PKG_VERSION must parse");
        assert_eq!(installed.major, 0);
        assert!(installed.patch < 1000);
    }

    #[test]
    fn one_check_per_month_and_never_a_second_in_the_same_one() {
        assert!(should_check(None, m("2026-11")));
        assert!(!should_check(Some(m("2026-11")), m("2026-11")));
        assert!(should_check(Some(m("2026-11")), m("2026-12")));
        assert!(should_check(Some(m("2026-11")), m("2027-05")));
        // A clock that went backwards is not a reason to check again.
        assert!(!should_check(Some(m("2026-11")), m("2026-10")));
    }

    #[test]
    fn the_warning_belongs_to_the_large_transfer_not_to_the_check() {
        let big = download_notice(94_371_840, true);
        assert!(big.contains("90"));
        assert!(big.contains("محدود"));
        let big_free = download_notice(94_371_840, false);
        assert!(!big_free.contains("محدود"));
        // 90 MiB rounds up rather than down: a size shown must never be a lie.
        assert!(download_notice(1, false).contains('1'));
        assert_eq!(REQUEST_METHOD, "GET");
        assert_eq!(USER_AGENT, "Odina-update-check");
        assert!(!USER_AGENT.contains("0."));
    }
}
