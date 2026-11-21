// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K1 signature/threat catalogue contract — the *format*, not the data.
//!
//! `docs/spec/threat-db-v1.md` is the reference. This module owns three rules
//! that a future loader must never re-decide locally:
//!
//! 1. **A catalogue that cannot be verified is not opened.** Unsigned, wrong
//!    digest, unknown newer schema, or a field outside the closed vocabulary
//!    all refuse the whole file; no partial load, no "best effort" rows.
//! 2. **A failed update keeps the last good catalogue.** It never falls back to
//!    an empty one, and it never accepts an older or equal generation: a signed
//!    replay is still stale data.
//! 3. **A Bloom index from another generation downgrades coverage to exact
//!    matching.** It is reported as `ExactOnly`, never as full protection.
//!
//! There is no language field and there cannot be one: the vocabulary is a
//! closed enum of bits, so an unknown bit — a language column included — is a
//! rejected file (§4.8, D84). No key material, no network and no file I/O live
//! here; the caller supplies the verified facts.

/// Eight magic bytes at offset 0 of the catalogue.
pub const MAGIC: [u8; 8] = *b"ODINADB1";
/// Highest schema this build understands. A newer file is refused, not guessed.
pub const SCHEMA_VERSION: u16 = 1;

/// One field a record may carry. Closed on purpose — see the module note.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    /// Identity: the full SHA-256. Every tier has it.
    Sha256,
    /// Indicator 1 — exact byte size.
    Size,
    /// Family identifier, never a human sentence and never a language name.
    Family,
    /// Indicator 2 — name structure.
    NamePattern,
    /// Indicator 3 — path structure.
    PathPattern,
    /// Indicator 4 — file attributes.
    Attributes,
    /// Indicator 5 — header hash.
    HeaderHash,
    /// Indicator 6 — tail hash.
    TailHash,
    /// Indicator 7 — PE/imphash.
    PeHash,
    /// Indicator 8 — TLSH.
    Fuzzy,
    /// Indicator 9 — signature absence.
    Unsigned,
    /// Indicator 10 — declared behaviour fingerprint (imports/strings).
    DeclaredBehaviour,
}

impl Field {
    pub const ALL: [Field; 12] = [
        Field::Sha256,
        Field::Size,
        Field::Family,
        Field::NamePattern,
        Field::PathPattern,
        Field::Attributes,
        Field::HeaderHash,
        Field::TailHash,
        Field::PeHash,
        Field::Fuzzy,
        Field::Unsigned,
        Field::DeclaredBehaviour,
    ];

    pub const fn bit(self) -> u16 {
        1 << (self as u16)
    }
}

/// A record's declared field set, as read from the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldSet(u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbReject {
    BadMagic,
    NewerSchema,
    Unsigned,
    DigestMismatch,
    UnknownField,
    IncompleteTier,
    Empty,
    /// Older or equal generation: valid signature, stale data.
    NotNewer,
}

impl FieldSet {
    pub fn from_bits(bits: u16) -> Result<Self, DbReject> {
        let known = Field::ALL.iter().fold(0u16, |acc, f| acc | f.bit());
        if bits & !known != 0 {
            // This is the only place a column such as "language" could enter,
            // and it is the place that refuses it without knowing its name.
            return Err(DbReject::UnknownField);
        }
        Ok(Self(bits))
    }

    pub fn of(fields: &[Field]) -> Self {
        let mut bits = 0u16;
        for f in fields {
            bits |= f.bit();
        }
        Self(bits)
    }

    pub fn contains(self, field: Field) -> bool {
        self.0 & field.bit() != 0
    }

    pub fn bits(self) -> u16 {
        self.0
    }
}

/// The three tiers of §6.3. A record belongs to exactly one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    /// A — full: every indicator plus family. 550B per record is design, not measurement.
    A,
    /// B — hash + size + family.
    B,
    /// C — hash only.
    C,
}

impl Tier {
    pub const fn required_fields(self) -> &'static [Field] {
        match self {
            Tier::A => &Field::ALL,
            Tier::B => &[Field::Sha256, Field::Size, Field::Family],
            Tier::C => &[Field::Sha256],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TierStats {
    pub records: u64,
    pub fields: FieldSet,
}

impl TierStats {
    pub fn is_complete_for(self, tier: Tier) -> bool {
        tier.required_fields().iter().all(|f| self.fields.contains(*f))
    }
}

/// What the verified header says. Every flag is a fact the caller already
/// established; this type only decides what those facts mean together.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CatalogManifest {
    pub magic: [u8; 8],
    pub schema_version: u16,
    pub generation: u64,
    /// Generation of the separate Bloom index, if one shipped with this file.
    pub bloom_generation: Option<u64>,
    pub signature_valid: bool,
    pub digest_matches_manifest: bool,
    pub a: TierStats,
    pub b: TierStats,
    pub c: TierStats,
}

/// How much of the catalogue is actually usable for a decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Coverage {
    /// Bloom and index agree; a negative still needs the exact check elsewhere.
    Full,
    /// Bloom is missing or from another generation: exact matching only. This
    /// is narrower coverage and must be reported as such.
    ExactOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Catalog {
    pub generation: u64,
    pub coverage: Coverage,
    pub records: u64,
}

impl CatalogManifest {
    pub fn open(self) -> Result<Catalog, DbReject> {
        if self.magic != MAGIC {
            return Err(DbReject::BadMagic);
        }
        if self.schema_version > SCHEMA_VERSION {
            return Err(DbReject::NewerSchema);
        }
        if !self.signature_valid {
            return Err(DbReject::Unsigned);
        }
        if !self.digest_matches_manifest {
            return Err(DbReject::DigestMismatch);
        }
        if !self.a.is_complete_for(Tier::A)
            || !self.b.is_complete_for(Tier::B)
            || !self.c.is_complete_for(Tier::C)
        {
            return Err(DbReject::IncompleteTier);
        }
        let records = self.a.records + self.b.records + self.c.records;
        if records == 0 {
            return Err(DbReject::Empty);
        }
        let coverage = match self.bloom_generation {
            Some(g) if g == self.generation => Coverage::Full,
            _ => Coverage::ExactOnly,
        };
        Ok(Catalog {
            generation: self.generation,
            coverage,
            records,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallOutcome {
    Activated(Catalog),
    /// The previous generation, unchanged, stays active. `None` means no
    /// catalogue is active at all — never presented as clean or complete.
    KeptPrevious {
        active_generation: Option<u64>,
        reason: DbReject,
    },
}

/// Holds the active catalogue and enforces "last good wins".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CatalogStore {
    active: Option<Catalog>,
}

impl CatalogStore {
    pub fn new() -> Self {
        Self { active: None }
    }

    pub fn active(&self) -> Option<Catalog> {
        self.active
    }

    pub fn install(&mut self, manifest: CatalogManifest) -> InstallOutcome {
        let candidate = match manifest.open() {
            Ok(c) => c,
            Err(reason) => {
                return InstallOutcome::KeptPrevious {
                    active_generation: self.active.map(|c| c.generation),
                    reason,
                }
            }
        };
        if self.active.is_some_and(|c| candidate.generation <= c.generation) {
            return InstallOutcome::KeptPrevious {
                active_generation: self.active.map(|c| c.generation),
                reason: DbReject::NotNewer,
            };
        }
        self.active = Some(candidate);
        InstallOutcome::Activated(candidate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats(fields: &[Field], records: u64) -> TierStats {
        TierStats {
            records,
            fields: FieldSet::of(fields),
        }
    }

    fn manifest(generation: u64) -> CatalogManifest {
        CatalogManifest {
            magic: MAGIC,
            schema_version: SCHEMA_VERSION,
            generation,
            bloom_generation: Some(generation),
            signature_valid: true,
            digest_matches_manifest: true,
            a: stats(Tier::A.required_fields(), 3),
            b: stats(Tier::B.required_fields(), 5),
            c: stats(Tier::C.required_fields(), 7),
        }
    }

    #[test]
    fn tier_fields_are_exactly_the_spec_fields() {
        assert_eq!(Tier::A.required_fields().len(), 12);
        assert_eq!(Tier::C.required_fields(), [Field::Sha256].as_slice());
        assert_eq!(
            Tier::B.required_fields(),
            [Field::Sha256, Field::Size, Field::Family].as_slice()
        );
        // Tier A carries the ten indicators, an identity and a family id.
        let indicators = Field::ALL
            .iter()
            .filter(|f| !matches!(**f, Field::Sha256 | Field::Family))
            .count();
        assert_eq!(indicators, 10);
        assert!(manifest(1).a.is_complete_for(Tier::A));
        assert!(!stats(&[Field::Sha256], 1).is_complete_for(Tier::B));
    }

    #[test]
    fn a_language_column_cannot_be_encoded_at_all() {
        let known = Field::ALL.iter().fold(0u16, |acc, f| acc | f.bit());
        assert_eq!(FieldSet::from_bits(known), Ok(FieldSet::of(&Field::ALL)));
        assert_eq!(
            FieldSet::from_bits(known | (1 << 15)),
            Err(DbReject::UnknownField)
        );
        assert_eq!(FieldSet::from_bits(0), Ok(FieldSet::of(&[])));
    }

    #[test]
    fn unsigned_mismatched_newer_or_empty_files_never_open() {
        let base = manifest(1);
        assert_eq!(base.open().unwrap().coverage, Coverage::Full);
        assert_eq!(
            CatalogManifest {
                signature_valid: false,
                ..base
            }
            .open(),
            Err(DbReject::Unsigned)
        );
        assert_eq!(
            CatalogManifest {
                digest_matches_manifest: false,
                ..base
            }
            .open(),
            Err(DbReject::DigestMismatch)
        );
        assert_eq!(
            CatalogManifest {
                schema_version: SCHEMA_VERSION + 1,
                ..base
            }
            .open(),
            Err(DbReject::NewerSchema)
        );
        assert_eq!(
            CatalogManifest {
                magic: *b"NOTADB01",
                ..base
            }
            .open(),
            Err(DbReject::BadMagic)
        );
        let zero = |tier: Tier| TierStats {
            records: 0,
            fields: FieldSet::of(tier.required_fields()),
        };
        assert_eq!(
            CatalogManifest {
                a: zero(Tier::A),
                b: zero(Tier::B),
                c: zero(Tier::C),
                ..base
            }
            .open(),
            Err(DbReject::Empty)
        );
    }

    #[test]
    fn a_fresh_bloom_from_another_generation_downgrades_to_exact_only() {
        let mismatch = CatalogManifest {
            bloom_generation: Some(9),
            ..manifest(4)
        }
        .open()
        .unwrap();
        assert_eq!(mismatch.coverage, Coverage::ExactOnly);
        let absent = CatalogManifest {
            bloom_generation: None,
            ..manifest(4)
        }
        .open()
        .unwrap();
        assert_eq!(absent.coverage, Coverage::ExactOnly);
        // Exact-only is narrower coverage, never a smaller record count.
        assert_eq!(absent.records, mismatch.records);
    }

    #[test]
    fn a_failed_update_keeps_the_last_good_catalogue() {
        let mut store = CatalogStore::new();
        assert_eq!(
            store.install(manifest(2)),
            InstallOutcome::Activated(Catalog {
                generation: 2,
                coverage: Coverage::Full,
                records: 15
            })
        );
        for broken in [
            CatalogManifest {
                signature_valid: false,
                generation: 3,
                ..manifest(3)
            },
            CatalogManifest {
                digest_matches_manifest: false,
                generation: 3,
                ..manifest(3)
            },
            CatalogManifest {
                a: stats(&[Field::Sha256], 1),
                generation: 3,
                ..manifest(3)
            },
        ] {
            let outcome = store.install(broken);
            assert!(matches!(
                outcome,
                InstallOutcome::KeptPrevious {
                    active_generation: Some(2),
                    ..
                }
            ));
        }
        assert_eq!(store.active().unwrap().generation, 2);
    }

    #[test]
    fn a_signed_replay_is_still_stale_data() {
        let mut store = CatalogStore::new();
        store.install(manifest(7));
        for generation in [7, 6, 1] {
            assert_eq!(
                store.install(manifest(generation)),
                InstallOutcome::KeptPrevious {
                    active_generation: Some(7),
                    reason: DbReject::NotNewer
                }
            );
        }
        assert_eq!(store.active().unwrap().generation, 7);
        // No active catalogue is never reported as a clean state.
        let mut fresh = CatalogStore::new();
        assert_eq!(
            fresh.install(CatalogManifest {
                magic: *b"BROKEN!!",
                ..manifest(1)
            }),
            InstallOutcome::KeptPrevious {
                active_generation: None,
                reason: DbReject::BadMagic
            }
        );
        assert!(fresh.active().is_none());
    }
}
