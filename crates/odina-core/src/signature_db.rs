// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K1 signature/threat catalogue contract — the *format*, the *trust*, and the
//! *states*, not the data itself.
//!
//! `docs/spec/threat-db-v1.md` is the reference, decided by D102–D111. This
//! module owns rules that no loader, updater or UI may re-decide locally:
//!
//! 1. **A catalogue that cannot be verified is not opened.** Unknown magic,
//!    newer schema, unknown signing key, bad signature, wrong digest, missing
//!    tier fields or a bit outside the closed vocabulary all refuse the whole
//!    file; no partial load, no "best effort" rows.
//! 2. **A failed update keeps the last good catalogue**, and an older or equal
//!    generation is refused even when it is correctly signed: a signed replay
//!    is still stale data.
//! 3. **"Protected" is a conclusion, not a default** (D104, §10.4). It appears
//!    only when protection is running, a catalogue is active, its Bloom index
//!    matches its generation, and its age is current. A seed is never coverage.
//! 4. **Age has one policy** (D106): a warning at 30 days, one user
//!    acknowledgement bought 30 more days with the warning still visible, and
//!    strict mode at 60 days without one.
//! 5. **Admission decides, then the database is simple** (D109): a hash enters
//!    with full verdict power or it does not enter. There is no trust grade
//!    inside the catalogue.
//!
//! No key material, no network and no file I/O live here; the caller supplies
//! verified facts. Reading is `mmap` when the filesystem allows it and a
//! sequential read when it does not (D103) — a USB stick or a network share is
//! not an error, it is a different access mode.

use crate::FileHashes;
use std::sync::OnceLock;

/// Eight magic bytes at offset 0 of the catalogue.
pub const MAGIC: [u8; 8] = *b"ODINADB1";
/// Highest schema this build understands. A newer file is refused, not guessed.
pub const SCHEMA_VERSION: u16 = 1;
/// Signing keys pinned in the binary (D108). An unknown id is a rejected file.
pub const KEY_MAIN: u8 = 1;
/// The standby key, pinned before it is used so rotation needs no new trust.
pub const KEY_NEXT: u8 = 2;
/// Days of age at which the catalogue shows a warning (D106).
pub const WARN_DAYS: u32 = 30;
/// Days of age at which scanning turns strict without an acknowledgement.
pub const STRICT_DAYS: u32 = 60;
/// One acknowledgement buys this many more days of warning — never silence.
pub const ACK_EXTENSION_DAYS: u32 = 30;
/// The first batch from a new source gets human eyes, capped so the review is
/// real rather than a rubber stamp (D109).
pub const MAX_FIRST_BATCH_REVIEW: usize = 200;
/// How large the seed inside the installer may grow (D104). A published cap is
/// a promise to the user about installer size, so it is enforced here and in
/// `tools/package-windows.py`, not in a comment.
pub const SEED_MAX_BYTES: usize = 2 * 1024 * 1024;

/// Whether a key id is pinned into this build.
pub fn pinned(key_id: u8) -> bool {
    key_id == KEY_MAIN || key_id == KEY_NEXT
}

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
    UnknownKey,
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
        (self.0 & field.bit()) != 0
    }

    pub fn bits(self) -> u16 {
        self.0
    }
}

/// The three tiers of §6.3. A record belongs to exactly one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    /// A — full: every indicator plus family.
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
        tier.required_fields()
            .iter()
            .all(|f| self.fields.contains(*f))
    }
}

/// Whether this file is the small seed shipped with the program or a full
/// catalogue downloaded separately (D104). A seed is never coverage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogKind {
    Seed,
    Full,
}

/// How the file is being read. `Mapped` is the fast path; `Sequential` exists
/// because a removable stick or a network share can refuse a mapping (D103).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    Mapped,
    Sequential,
}

/// Pick the access mode without pretending a mapping always works.
pub const fn access_mode(mapped: bool) -> Access {
    if mapped {
        Access::Mapped
    } else {
        Access::Sequential
    }
}

/// What the verified header says. Every flag is a fact the caller already
/// established; this type only decides what those facts mean together.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CatalogManifest {
    pub magic: [u8; 8],
    pub schema_version: u16,
    /// Which pinned key signed this file (D108).
    pub key_id: u8,
    pub generation: u64,
    /// Generation of the separate Bloom index, if one shipped with this file.
    pub bloom_generation: Option<u64>,
    pub signature_valid: bool,
    pub digest_matches_manifest: bool,
    pub kind: CatalogKind,
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
    pub key_id: u8,
    pub kind: CatalogKind,
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
        if !pinned(self.key_id) {
            return Err(DbReject::UnknownKey);
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
            key_id: self.key_id,
            kind: self.kind,
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
        if self
            .active
            .is_some_and(|c| candidate.generation <= c.generation)
        {
            return InstallOutcome::KeptPrevious {
                active_generation: self.active.map(|c| c.generation),
                reason: DbReject::NotNewer,
            };
        }
        self.active = Some(candidate);
        InstallOutcome::Activated(candidate)
    }
}

/// The single age policy (D106). One acknowledgement extends the warning
/// window; it never hides the warning and never becomes permanent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgeStatus {
    /// Within the warning window.
    Current,
    /// Old enough to warn, still below the strict deadline.
    Warn,
    /// Past the deadline but inside an acknowledged extension.
    Extended,
    /// Past every window: strict mode, still a working catalogue.
    Strict,
}

/// `acknowledged_days_ago` is the age of the catalogue when the user pressed
/// "I know" — an acknowledgement before the warning existed does not count.
pub fn age_status(age_days: u32, acknowledged_at_age: Option<u32>) -> AgeStatus {
    if age_days < WARN_DAYS {
        return AgeStatus::Current;
    }
    let acked = acknowledged_at_age.is_some_and(|at| at >= WARN_DAYS && at < STRICT_DAYS);
    let deadline = if acked {
        STRICT_DAYS + ACK_EXTENSION_DAYS
    } else {
        STRICT_DAYS
    };
    if age_days < deadline {
        return if acked {
            AgeStatus::Extended
        } else {
            AgeStatus::Warn
        };
    }
    AgeStatus::Strict
}

/// The four states the UI may show (§10.4). "Protected" is reached only by
/// argument, never inferred from the service simply running.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtectionState {
    Protected,
    WorkingWithAlert,
    NeedsAttention,
    Stopped,
}

pub fn protection_state(
    protection_enabled: bool,
    catalog: Option<&Catalog>,
    age: AgeStatus,
) -> ProtectionState {
    if !protection_enabled {
        return ProtectionState::Stopped;
    }
    let Some(catalog) = catalog else {
        return ProtectionState::NeedsAttention;
    };
    // A seed is not coverage (D104): matching a seed row may warn, never bless.
    if catalog.kind == CatalogKind::Seed
        || catalog.coverage == Coverage::ExactOnly
        || age != AgeStatus::Current
    {
        return ProtectionState::WorkingWithAlert;
    }
    ProtectionState::Protected
}

/// Where a hash came from. Independence is about the *origin*, not the site
/// that republished it (D109): two mirrors of one upstream are one source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Origin {
    pub site: String,
    pub upstream: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub sha256: String,
    pub origins: Vec<Origin>,
    /// A sample scanned here, with its full digest recomputed.
    pub local_sample_verified: bool,
    /// Membership checked against the clean corpus before admission.
    pub clean_corpus_checked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionReject {
    NotSha256,
    NoCleanCorpusCheck,
    Unverified,
    SameUpstream,
}

/// Admission gates (D109). A hash that passes gets full verdict power; one
/// that fails does not enter at all — there is no middle grade.
pub fn admit(candidate: &Candidate) -> Result<(), AdmissionReject> {
    if candidate.sha256.len() != 64 || !candidate.sha256.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(AdmissionReject::NotSha256);
    }
    if !candidate.clean_corpus_checked {
        return Err(AdmissionReject::NoCleanCorpusCheck);
    }
    if candidate.local_sample_verified {
        return Ok(());
    }
    if candidate.origins.len() < 2 {
        return Err(AdmissionReject::Unverified);
    }
    let first = candidate.origins[0].upstream.as_str();
    if candidate.origins[1..]
        .iter()
        .all(|o| o.upstream.as_str() == first)
    {
        return Err(AdmissionReject::SameUpstream);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemovalReason {
    /// The hash was never malicious: the only reason that offers a restore.
    FalsePositive,
    Superseded,
    LicenceWithdrawn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Removal {
    pub sha256: String,
    pub reason: RemovalReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quarantined {
    pub id: String,
    pub sha256: String,
}

/// A correction never un-quarantines anything by itself (D110): it returns the
/// entries the UI may *offer* to restore, with the incident as the record.
pub fn restore_offers(quarantined: &[Quarantined], removals: &[Removal]) -> Vec<String> {
    quarantined
        .iter()
        .filter(|q| {
            removals.iter().any(|r| {
                r.reason == RemovalReason::FalsePositive && r.sha256.eq_ignore_ascii_case(&q.sha256)
            })
        })
        .map(|q| q.id.clone())
        .collect()
}

/// The seed shipped inside the program (D104). It is a starting point, not
/// coverage, and it is CC0 like every row the project generates itself (D102).
pub const SEED_TSV: &str = include_str!("../data/seed-v1.tsv");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeedRow<'a> {
    pub sha256: &'a str,
    pub size: u64,
    pub family: &'a str,
    pub source: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeedReject {
    Header,
    ColumnCount,
    NotSha256,
    Size,
    Family,
    Licence,
    Source,
    Duplicate,
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeedCatalog<'a> {
    rows: Vec<SeedRow<'a>>,
}

impl<'a> SeedCatalog<'a> {
    /// Parses `sha256  size  family  licence  source` rows under a version
    /// header. Only a licence we can actually grant is accepted; anything else
    /// is refused rather than silently relicensed.
    pub fn parse(text: &'a str) -> Result<Self, SeedReject> {
        let mut rows: Vec<SeedRow<'a>> = Vec::new();
        let mut seen_header = false;
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if !seen_header {
                if line != "odina-seed/v1\t1" {
                    return Err(SeedReject::Header);
                }
                seen_header = true;
                continue;
            }
            let columns: Vec<&str> = line.split('\t').collect();
            if columns.len() != 5 {
                return Err(SeedReject::ColumnCount);
            }
            let sha256 = columns[0];
            if sha256.len() != 64 || !sha256.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(SeedReject::NotSha256);
            }
            let size = columns[1].parse::<u64>().map_err(|_| SeedReject::Size)?;
            if columns[2].is_empty() {
                return Err(SeedReject::Family);
            }
            if !columns[3].eq_ignore_ascii_case("cc0-1.0") {
                return Err(SeedReject::Licence);
            }
            if columns[4].is_empty() {
                return Err(SeedReject::Source);
            }
            if rows.iter().any(|r| r.sha256.eq_ignore_ascii_case(sha256)) {
                return Err(SeedReject::Duplicate);
            }
            rows.push(SeedRow {
                sha256,
                size,
                family: columns[2],
                source: columns[4],
            });
        }
        if rows.is_empty() {
            return Err(SeedReject::Empty);
        }
        Ok(Self { rows })
    }

    pub fn rows(&self) -> &[SeedRow<'a>] {
        &self.rows
    }

    pub fn contains(&self, sha256: &str) -> bool {
        self.rows
            .iter()
            .any(|r| r.sha256.eq_ignore_ascii_case(sha256))
    }
}

static SEED: OnceLock<Option<SeedCatalog<'static>>> = OnceLock::new();

/// The shipped seed, parsed once. A parse failure is a build defect and the
/// test below turns it into a failing CI run rather than a silent empty list.
pub fn seed_catalog() -> Option<&'static SeedCatalog<'static>> {
    SEED.get_or_init(|| SeedCatalog::parse(SEED_TSV).ok())
        .as_ref()
}

/// Seed membership by exact digest. Never a substitute for the full catalogue.
pub fn seed_matches(hashes: &FileHashes) -> bool {
    seed_catalog().is_some_and(|c| c.contains(&hashes.full))
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
            key_id: KEY_MAIN,
            generation,
            bloom_generation: Some(generation),
            signature_valid: true,
            digest_matches_manifest: true,
            kind: CatalogKind::Full,
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
        assert_eq!(
            CatalogManifest { key_id: 9, ..base }.open(),
            Err(DbReject::UnknownKey)
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
        assert_eq!(absent.records, mismatch.records);
    }

    #[test]
    fn a_failed_update_keeps_the_last_good_catalogue() {
        let mut store = CatalogStore::new();
        assert_eq!(
            store.install(manifest(2)),
            InstallOutcome::Activated(Catalog {
                generation: 2,
                key_id: KEY_MAIN,
                kind: CatalogKind::Full,
                coverage: Coverage::Full,
                records: 15,
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

    #[test]
    fn age_policy_warns_extends_and_then_is_strict() {
        assert_eq!(age_status(0, None), AgeStatus::Current);
        assert_eq!(age_status(WARN_DAYS - 1, None), AgeStatus::Current);
        assert_eq!(age_status(WARN_DAYS, None), AgeStatus::Warn);
        assert_eq!(age_status(STRICT_DAYS - 1, None), AgeStatus::Warn);
        assert_eq!(age_status(STRICT_DAYS, None), AgeStatus::Strict);
        // One acknowledgement, made inside the warning window, buys one window.
        assert_eq!(
            age_status(STRICT_DAYS, Some(WARN_DAYS)),
            AgeStatus::Extended
        );
        assert_eq!(
            age_status(STRICT_DAYS + ACK_EXTENSION_DAYS - 1, Some(WARN_DAYS)),
            AgeStatus::Extended
        );
        assert_eq!(
            age_status(STRICT_DAYS + ACK_EXTENSION_DAYS, Some(WARN_DAYS)),
            AgeStatus::Strict
        );
        // An acknowledgement before the warning existed is not one.
        assert_eq!(age_status(STRICT_DAYS, Some(1)), AgeStatus::Strict);
    }

    #[test]
    fn protected_is_a_conclusion_not_a_default() {
        let current = manifest(5).open().unwrap();
        assert_eq!(
            protection_state(true, Some(&current), AgeStatus::Current),
            ProtectionState::Protected
        );
        assert_eq!(
            protection_state(true, None, AgeStatus::Current),
            ProtectionState::NeedsAttention
        );
        assert_eq!(
            protection_state(false, Some(&current), AgeStatus::Current),
            ProtectionState::Stopped
        );
        for age in [AgeStatus::Warn, AgeStatus::Extended, AgeStatus::Strict] {
            assert_eq!(
                protection_state(true, Some(&current), age),
                ProtectionState::WorkingWithAlert
            );
        }
        let exact_only = CatalogManifest {
            bloom_generation: None,
            ..manifest(5)
        }
        .open()
        .unwrap();
        assert_eq!(
            protection_state(true, Some(&exact_only), AgeStatus::Current),
            ProtectionState::WorkingWithAlert
        );
        // A seed is never coverage, whatever its generation says.
        let seed = CatalogManifest {
            kind: CatalogKind::Seed,
            ..manifest(5)
        }
        .open()
        .unwrap();
        assert_eq!(seed.kind, CatalogKind::Seed);
        assert_eq!(seed.records, 15);
        assert_eq!(
            protection_state(true, Some(&seed), AgeStatus::Current),
            ProtectionState::WorkingWithAlert
        );
    }

    #[test]
    fn admission_needs_independent_origins_or_a_local_sample() {
        let sha = "a".repeat(64);
        let origin = |upstream: &str| Origin {
            site: "mirror.example".into(),
            upstream: upstream.into(),
        };
        let base = Candidate {
            sha256: sha.clone(),
            origins: vec![origin("upstream-a"), origin("upstream-b")],
            local_sample_verified: false,
            clean_corpus_checked: true,
        };
        assert_eq!(admit(&base), Ok(()));
        assert_eq!(
            admit(&Candidate {
                sha256: "xyz".into(),
                ..base.clone()
            }),
            Err(AdmissionReject::NotSha256)
        );
        assert_eq!(
            admit(&Candidate {
                clean_corpus_checked: false,
                ..base.clone()
            }),
            Err(AdmissionReject::NoCleanCorpusCheck)
        );
        assert_eq!(
            admit(&Candidate {
                local_sample_verified: true,
                origins: vec![],
                ..base.clone()
            }),
            Ok(())
        );
        assert_eq!(
            admit(&Candidate {
                origins: vec![origin("same"), origin("same")],
                ..base.clone()
            }),
            Err(AdmissionReject::SameUpstream)
        );
        assert_eq!(
            admit(&Candidate {
                origins: vec![origin("only")],
                ..base
            }),
            Err(AdmissionReject::Unverified)
        );
    }

    #[test]
    fn a_correction_offers_a_restore_and_never_performs_one() {
        let quarantined = [
            Quarantined {
                id: "q1".into(),
                sha256: "b".repeat(64),
            },
            Quarantined {
                id: "q2".into(),
                sha256: "c".repeat(64),
            },
        ];
        let removals = [
            Removal {
                sha256: "b".repeat(64),
                reason: RemovalReason::FalsePositive,
            },
            Removal {
                sha256: "c".repeat(64),
                reason: RemovalReason::Superseded,
            },
        ];
        assert_eq!(restore_offers(&quarantined, &removals), vec!["q1"]);
        assert!(restore_offers(&[], &removals).is_empty());
        assert!(restore_offers(&quarantined, &[]).is_empty());
    }

    #[test]
    fn the_shipped_seed_parses_and_is_never_called_coverage() {
        let seed = seed_catalog().expect("the shipped seed must parse");
        assert!(!seed.rows().is_empty());
        assert!(seed.contains("275A021BBFB6489E54D471899F7DB9D1663FC695EC2FE2A2C4538AABF651FD0F"));
        assert!(!seed.contains("f".repeat(64).as_str()));
        let first = seed.rows()[0];
        assert!(!first.family.is_empty());
        assert_eq!(access_mode(true), Access::Mapped);
        assert_eq!(access_mode(false), Access::Sequential);
    }

    #[test]
    fn the_seed_stays_inside_its_published_cap() {
        assert!(
            SEED_TSV.len() <= SEED_MAX_BYTES,
            "seed is {} bytes, cap is {}",
            SEED_TSV.len(),
            SEED_MAX_BYTES
        );
    }

    #[test]
    fn a_seed_row_with_a_licence_we_cannot_grant_is_refused() {
        let header = "odina-seed/v1\t1\n";
        let good = "aa".repeat(32);
        let row = |licence: &str| format!("{header}{good}\t68\tfam\t{licence}\tsrc\n");
        assert!(SeedCatalog::parse(&row("cc0-1.0")).is_ok());
        for bad in ["gpl-2.0", "agpl-3.0", "proprietary", ""] {
            assert_eq!(
                SeedCatalog::parse(&row(bad)),
                Err(SeedReject::Licence),
                "licence {bad}"
            );
        }
        assert_eq!(
            SeedCatalog::parse("wrong-header\n"),
            Err(SeedReject::Header)
        );
        assert_eq!(SeedCatalog::parse(header), Err(SeedReject::Empty));
        assert_eq!(
            SeedCatalog::parse(&format!("{header}{good}\t68\tfam\n")),
            Err(SeedReject::ColumnCount)
        );
        assert_eq!(
            SeedCatalog::parse(&format!("{header}nothex\t68\tfam\tcc0-1.0\tsrc\n")),
            Err(SeedReject::NotSha256)
        );
        assert_eq!(
            SeedCatalog::parse(&format!("{header}{good}\tNaN\tfam\tcc0-1.0\tsrc\n")),
            Err(SeedReject::Size)
        );
        assert_eq!(
            SeedCatalog::parse(&format!("{header}{good}\t68\tfam\tcc0-1.0\t\n")),
            Err(SeedReject::Source)
        );
    }
}
