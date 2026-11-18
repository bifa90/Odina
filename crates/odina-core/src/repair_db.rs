// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! Read-only recipe database, NOT a detector. Only the digest is compiled into
//! the program. A downloaded file cannot supply its own trust anchor or code.

use crate::hashing::sha256_hex;
use std::collections::BTreeMap;

pub const COMPONENT_PATH: &str = "components/repair/odina-repair-v1.tsv";
const PIN: &str = include_str!("repair-v1.sha256");
pub const MAX_CATALOG_BYTES: usize = 128 * 1024 * 1024;
const MAX_CANDIDATE_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Format {
    Text,
    Settings,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RepairScope {
    File,
    FileAndSettings,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Reason {
    MalwareOnly,
    DatabaseUnavailable,
    NoSafeRule,
    NoVerifiedBackup,
    UnsafeOutput,
    CannotRepair,
    VerificationFailed,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogError {
    Size,
    Integrity,
    Schema,
}

#[derive(Clone, Copy, Debug)]
enum CleanOp {
    KeepPrefix(usize),
    VerifiedBackup,
}
#[derive(Debug)]
struct Recipe {
    clean: CleanOp,
    output_digest: String,
    scope: RepairScope,
}
#[derive(Debug)]
pub struct Catalog {
    rows: BTreeMap<(String, Format, String), Recipe>,
}

/// A verified *staging candidate*. This is not an authorization to replace
/// the original. Keep rollback data and independently rescan before commit.
#[derive(Debug, PartialEq, Eq)]
pub struct Candidate<'a> {
    pub bytes: &'a [u8],
    pub scope: RepairScope,
}

impl Catalog {
    pub fn load(bytes: &[u8]) -> Result<Self, CatalogError> {
        if bytes.len() > MAX_CATALOG_BYTES {
            return Err(CatalogError::Size);
        }
        if sha256_hex(bytes) != PIN.trim() {
            return Err(CatalogError::Integrity);
        }
        Self::parse(bytes)
    }

    // No fallback parser, globs, paths, scripts, DLLs, or external references.
    fn parse(bytes: &[u8]) -> Result<Self, CatalogError> {
        let bad = CatalogError::Schema;
        let mut lines = std::str::from_utf8(bytes).map_err(|_| bad)?.lines();
        if lines.next() != Some("odina-repair/v1\t1") {
            return Err(bad);
        }
        let mut rows = BTreeMap::new();
        for line in lines {
            if line.len() > 384 {
                return Err(bad);
            }
            let mut fields = line.split('\t');
            let mut field = || fields.next().ok_or(bad);
            let family = field()?;
            if family.is_empty()
                || family.len() > 64
                || !family
                    .bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b".-".contains(&c))
            {
                return Err(bad);
            }
            let format = match field()? {
                "text" => Format::Text,
                "settings" => Format::Settings,
                _ => return Err(bad),
            };
            let input = field()?;
            let clean = match (format, field()?, field()?) {
                (Format::Text, "keep-prefix", n) => {
                    let n: usize = n.parse().map_err(|_| bad)?;
                    if n == 0 || n > MAX_CANDIDATE_BYTES {
                        return Err(bad);
                    }
                    CleanOp::KeepPrefix(n)
                }
                (_, "verified-backup", "-") => CleanOp::VerifiedBackup,
                _ => return Err(bad),
            };
            let output = field()?;
            let digest = |s: &str| {
                s.len() == 64
                    && s.bytes()
                        .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
            };
            if !digest(input) || !digest(output) || input == output {
                return Err(bad);
            }
            let scope = match field()? {
                "file" => RepairScope::File,
                "file-and-settings" if format == Format::Settings => RepairScope::FileAndSettings,
                _ => return Err(bad),
            };
            let key = (family.to_owned(), format, input.to_owned());
            if fields.next().is_some()
                || rows
                    .insert(
                        key,
                        Recipe {
                            clean,
                            output_digest: output.into(),
                            scope,
                        },
                    )
                    .is_some()
            {
                return Err(bad);
            }
        }
        if rows.is_empty() {
            return Err(bad);
        }
        Ok(Self { rows })
    }

    pub fn recipe_count(&self) -> usize {
        self.rows.len()
    }

    pub(crate) fn prepare<'a>(
        &self,
        family: &str,
        format: Format,
        original: &'a [u8],
        backup: Option<&'a [u8]>,
    ) -> Result<Candidate<'a>, Reason> {
        if original.len() > MAX_CANDIDATE_BYTES || family.len() > 64 {
            return Err(Reason::NoSafeRule);
        }
        let key = (family.to_owned(), format, sha256_hex(original));
        let recipe = self.rows.get(&key).ok_or(Reason::NoSafeRule)?;
        let bytes = match recipe.clean {
            CleanOp::KeepPrefix(n) => original.get(..n).ok_or(Reason::UnsafeOutput)?,
            CleanOp::VerifiedBackup => backup.ok_or(Reason::NoVerifiedBackup)?,
        };
        if bytes.len() > MAX_CANDIDATE_BYTES
            || std::str::from_utf8(bytes).is_err()
            || sha256_hex(bytes) != recipe.output_digest
        {
            return Err(match recipe.clean {
                CleanOp::VerifiedBackup => Reason::NoVerifiedBackup,
                _ => Reason::UnsafeOutput,
            });
        }
        Ok(Candidate {
            bytes,
            scope: recipe.scope,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const DB: &[u8] = include_bytes!("../../../components/repair/odina-repair-v1.tsv");
    #[test]
    fn package_pin_and_strict_schema_are_independent_checks() {
        assert_eq!(Catalog::load(DB).unwrap().recipe_count(), 2);
        let s = std::str::from_utf8(DB).unwrap();
        for bad in [
            s.replace("/v1", "/v2"),
            s.replace("keep-prefix", "run-command"),
            s.replace("keep-prefix\t21", "keep-prefix\t0"),
            s.replacen("\tfile\n", "\tfile-and-settings\n", 1),
            format!("{}{}\n", s, s.lines().nth(1).unwrap()),
        ] {
            assert_eq!(
                Catalog::load(bad.as_bytes()).unwrap_err(),
                CatalogError::Integrity
            );
            assert_eq!(
                Catalog::parse(bad.as_bytes()).unwrap_err(),
                CatalogError::Schema
            );
        }
    }
    #[test]
    fn even_a_recognized_operation_must_produce_the_exact_safe_bytes() {
        // Test the second line of defence, separately from the release pin.
        let altered = std::str::from_utf8(DB)
            .unwrap()
            .replace("keep-prefix\t21", "keep-prefix\t20");
        let db = Catalog::parse(altered.as_bytes()).unwrap();
        assert_eq!(
            db.prepare(
                "demo.note",
                Format::Text,
                b"Odina training note.\n[test marker]\n",
                None
            ),
            Err(Reason::UnsafeOutput)
        );
    }

    #[test]
    fn exact_family_format_variant_and_backup_are_required() {
        let db = Catalog::load(DB).unwrap();
        let original = b"view=changed\n";
        let clean = b"view=normal\n";
        let prepare = |family, fmt, input: &'static [u8], backup: Option<&'static [u8]>| {
            db.prepare(family, fmt, input, backup)
        };
        assert_eq!(
            prepare("other", Format::Settings, original, Some(clean)),
            Err(Reason::NoSafeRule)
        );
        assert_eq!(
            prepare("demo.settings", Format::Text, original, Some(clean)),
            Err(Reason::NoSafeRule)
        );
        assert_eq!(
            prepare("demo.settings", Format::Settings, b"other", None),
            Err(Reason::NoSafeRule)
        );
        for backup in [None, Some(b"wrong".as_slice())] {
            assert_eq!(
                prepare("demo.settings", Format::Settings, original, backup),
                Err(Reason::NoVerifiedBackup)
            );
        }
        assert_eq!(
            prepare("demo.settings", Format::Settings, original, Some(clean)).unwrap(),
            Candidate {
                bytes: clean,
                scope: RepairScope::FileAndSettings
            }
        );
    }
}
