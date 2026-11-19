// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! Repair-first incident protocol (D47). The adapter supplies evidence from
//! the walls, a durable rollback journal and the outcomes of OS operations.
//! This module changes neither original bytes nor the machine.

use crate::hashing::sha256_hex;
use crate::proactive::Outcome;
use crate::repair_db::{Candidate, Catalog, Format, Reason, RepairScope};
use crate::scoring::Verdict;
use std::collections::{btree_map::Entry, BTreeMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Scan,
    Neutralize,
    Clean,
    Repair,
    Verify,
}
/// The adapter supplies a receipt for each wall *in order*. It never runs a
/// suspect file just to activate a conditional wall (§2 of protection policy).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wall {
    FileDatabase,
    Behavior,
    Proactive,
}
impl Wall {
    pub const ORDER: [Self; 3] = [Self::FileDatabase, Self::Behavior, Self::Proactive];
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallResult {
    Checked,
    NotApplicable,
    Incomplete,
}

/// Stable identity of a scanned object, never just a filename.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileId(pub u64);

/// Classification by the scanner; lack of a family match is NOT evidence
/// that the entire object is malware. An unknown host takes the safe fallback.
pub enum Object<'a> {
    MalwareOnly,
    Host {
        family: &'a str,
        format: Format,
        original: &'a [u8],
        backup: Option<&'a [u8]>,
    },
    UnsupportedHost,
}
#[derive(Debug, PartialEq, Eq)]
pub enum FilePlan<'a> {
    Stage(Candidate<'a>),
    Quarantine(Reason),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Disposition<'a> {
    /// Read back from the committed file, not the staging buffer.
    Restore {
        stored: &'a [u8],
        rollback_saved: bool,
    },
    Quarantine,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileState {
    Detected,
    Staged { digest: String, scope: RepairScope },
    ReadyToCommit { digest: String },
    NeedsQuarantine(Reason),
    Restored,
    Quarantined(Reason),
}

#[derive(Debug, Clone)]
pub struct Incident {
    stage: Stage,
    files: BTreeMap<FileId, FileState>,
    walls_checked: usize,
}
impl Incident {
    pub(crate) fn new(first: FileId) -> Self {
        Self {
            stage: Stage::Scan,
            walls_checked: 0,
            files: BTreeMap::from([(first, FileState::Detected)]),
        }
    }
    pub(crate) fn include(&mut self, file: FileId) -> bool {
        if self.files.contains_key(&file) {
            return true;
        } // Never reset an existing file.
        if self.stage != Stage::Scan {
            return false;
        } // Start a separate job for new findings.
        self.walls_checked = 0; // The inventory the previous receipts covered has changed.
        self.files.insert(file, FileState::Detected);
        true
    }
    pub fn stage(&self) -> Stage {
        self.stage
    }
    pub fn file(&self, id: FileId) -> Option<&FileState> {
        self.files.get(&id)
    }

    pub fn record_wall(&mut self, wall: Wall, result: WallResult) -> bool {
        if self.stage != Stage::Scan
            || Wall::ORDER.get(self.walls_checked) != Some(&wall)
            || result == WallResult::Incomplete
            || (wall == Wall::FileDatabase && result != WallResult::Checked)
        {
            return false;
        }
        self.walls_checked += 1;
        true
    }

    /// A Scan success attests that the three-wall assessment is complete
    /// (including a recorded not-applicable where appropriate). Neutralize
    /// success attests that active danger is stopped, not just detected.
    /// Repair success additionally covers *all* incident damage, including
    /// changes outside these files. No file callback releases the policy hold.
    pub fn advance(&mut self, completed: Stage, outcome: Outcome) -> bool {
        if self.stage != completed || outcome != Outcome::Succeeded {
            return false;
        }
        self.stage = match completed {
            Stage::Scan if self.walls_checked == Wall::ORDER.len() => Stage::Neutralize,
            Stage::Neutralize => Stage::Clean,
            Stage::Clean
                if self
                    .files
                    .values()
                    .all(|s| !matches!(s, FileState::Detected)) =>
            {
                Stage::Repair
            }
            Stage::Repair
                if self
                    .files
                    .values()
                    .all(|s| matches!(s, FileState::Restored | FileState::Quarantined(_))) =>
            {
                Stage::Verify
            }
            _ => return false,
        };
        true
    }

    pub(crate) fn verify_incident(&mut self, rescan: Option<Verdict>) -> bool {
        if self.stage != Stage::Verify {
            return false;
        }
        match rescan {
            Some(Verdict::Clean) => true,
            None => false, // Missing/cancelled rescan: retry, never release.
            Some(_) => {
                self.stage = Stage::Repair;
                false
            } // Investigate; require a new repair acknowledgement.
        }
    }

    /// Read-only operation on a stable snapshot of the confirmed object. No
    /// recipe is executable. None means this call was stale or out of order.
    pub fn plan<'a>(
        &mut self,
        file: FileId,
        object: Object<'a>,
        db: Option<&Catalog>,
    ) -> Option<FilePlan<'a>> {
        if self.stage != Stage::Clean || self.file(file) != Some(&FileState::Detected) {
            return None;
        }
        let result = match object {
            Object::MalwareOnly => Err(Reason::MalwareOnly),
            Object::UnsupportedHost => Err(Reason::NoSafeRule),
            Object::Host {
                family,
                format,
                original,
                backup,
            } => db
                .ok_or(Reason::DatabaseUnavailable)
                .and_then(|db| db.prepare(family, format, original, backup)),
        };
        Some(match result {
            Ok(candidate) => {
                self.files.insert(
                    file,
                    FileState::Staged {
                        digest: sha256_hex(candidate.bytes),
                        scope: candidate.scope,
                    },
                );
                FilePlan::Stage(candidate)
            }
            Err(reason) => {
                self.files.insert(file, FileState::NeedsQuarantine(reason));
                FilePlan::Quarantine(reason)
            }
        })
    }

    /// Verify the staged output, NOT the infected original. Require both the
    /// expected known-good bytes and a fresh clean verdict on those bytes.
    /// Timeout/cancellation is NOT evidence of a clean or unrecoverable file:
    /// leave it staged and retry. `Err` is an established unsafe repair.
    pub fn verify_file(
        &mut self,
        file: FileId,
        repaired: Result<(&[u8], Verdict), UnsafeRepair>,
    ) -> Option<&FileState> {
        if self.stage != Stage::Repair {
            return None;
        }
        let state = self.files.get_mut(&file)?;
        let FileState::Staged { digest, .. } = state else {
            return None;
        };
        *state = match repaired {
            Ok((bytes, Verdict::Clean)) if sha256_hex(bytes) == *digest => {
                FileState::ReadyToCommit {
                    digest: digest.clone(),
                }
            }
            Ok(_) => FileState::NeedsQuarantine(Reason::VerificationFailed),
            Err(UnsafeRepair) => FileState::NeedsQuarantine(Reason::CannotRepair),
        };
        Some(state)
    }

    /// Report ONLY after the adapter durably commits the verified bytes (with
    /// rollback), or secures the original in reversible quarantine. A locked
    /// file, failed commit or cancelled move is never reported as resolved.
    pub fn record_disposition(
        &mut self,
        file: FileId,
        done: Disposition<'_>,
        outcome: Outcome,
    ) -> bool {
        if self.stage != Stage::Repair || outcome != Outcome::Succeeded {
            return false;
        }
        let Entry::Occupied(mut entry) = self.files.entry(file) else {
            return false;
        };
        let state = match (entry.get(), done) {
            (
                FileState::ReadyToCommit { digest },
                Disposition::Restore {
                    stored,
                    rollback_saved: true,
                },
            ) if sha256_hex(stored) == *digest => FileState::Restored,
            (FileState::NeedsQuarantine(why), Disposition::Quarantine) => {
                FileState::Quarantined(*why)
            }
            _ => return false,
        };
        entry.insert(state);
        true
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnsafeRepair;

/// Closed model of the demo.settings recipe's one permitted setting. Replay
/// only a journaled before-image, never overwrite an intervening user edit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Normal,
    Compact,
    ChangedByFixture,
}
#[derive(Debug, Clone, Copy)]
pub struct ViewJournal {
    pub before: View,
    pub current: View,
}
impl RepairScope {
    pub fn repair_view(self, journal: Option<ViewJournal>) -> Result<Option<View>, UnsafeRepair> {
        match (self, journal) {
            (Self::File, _) => Ok(None),
            (Self::FileAndSettings, Some(ViewJournal { before, current }))
                if before != View::ChangedByFixture
                    && (current == View::ChangedByFixture || current == before) =>
            {
                Ok(Some(before))
            }
            _ => Err(UnsafeRepair),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn no_disinfection_without_ordered_wall_receipts_and_neutralization() {
        use Wall::*;
        use WallResult::*;
        let mut i = Incident::new(A);
        assert!(!i.advance(Stage::Scan, Succeeded));
        assert!(!i.record_wall(FileDatabase, NotApplicable));
        assert!(!i.record_wall(Proactive, Checked));
        assert!(i.record_wall(FileDatabase, Checked));
        assert!(i.include(A)); // repeat detection does not reset the wall
        assert!(!i.record_wall(Behavior, Incomplete));
        assert!(i.record_wall(Behavior, NotApplicable));
        assert!(i.include(B)); // new confirmed object: old scan no longer covers all files
        assert!(!i.record_wall(Proactive, Checked));
        for (w, result) in [
            (FileDatabase, Checked),
            (Behavior, Checked),
            (Proactive, NotApplicable),
        ] {
            assert!(i.record_wall(w, result));
        }
        assert!(i.advance(Stage::Scan, Succeeded));
        assert!(!i.record_wall(FileDatabase, Checked));
        assert!(!i.advance(Stage::Neutralize, Cancelled));
        assert!(i.plan(A, note(), None).is_none());
    }
    use crate::proactive::{JobId, Outcome::*, ProtectionMode::*, SurfacePolicy};
    const DB: &[u8] = include_bytes!("../../../components/repair/odina-repair-v1.tsv");
    const DIRTY: &[u8] = b"Odina training note.\n[test marker]\n";
    const CLEAN: &[u8] = b"Odina training note.\n";
    const A: FileId = FileId(1);
    const B: FileId = FileId(2);
    const JOB: JobId = JobId(3);
    fn note() -> Object<'static> {
        Object::Host {
            family: "demo.note",
            format: Format::Text,
            original: DIRTY,
            backup: None,
        }
    }
    fn cleaning() -> Incident {
        let mut i = Incident::new(A);
        for w in Wall::ORDER {
            assert!(i.record_wall(w, WallResult::Checked));
        }
        assert!(i.advance(Stage::Scan, Succeeded));
        assert!(i.advance(Stage::Neutralize, Succeeded));
        i
    }

    #[test]
    fn one_job_two_objects_release_only_after_repair_and_clean_final_rescan() {
        let db = Catalog::load(DB).unwrap();
        let mut p = SurfacePolicy::default();
        for id in [A, B] {
            assert!(p.observe_file(JOB, id, Verdict::Confirmed, AlertsMuted));
        }
        let i = p.incident_mut(JOB, AlertsMuted).unwrap();
        assert!(i.plan(A, note(), Some(&db)).is_none());
        for w in Wall::ORDER {
            assert!(i.record_wall(w, WallResult::Checked));
        }
        assert!(i.advance(Stage::Scan, Succeeded));
        assert!(!i.advance(Stage::Neutralize, Failed));
        assert!(i.plan(A, note(), Some(&db)).is_none());
        assert!(i.advance(Stage::Neutralize, Succeeded));
        assert!(!i.include(FileId(33)));
        let Some(FilePlan::Stage(c)) = i.plan(A, note(), Some(&db)) else {
            panic!("expected recovery")
        };
        assert_eq!(c.bytes, CLEAN);
        assert!(DIRTY.ends_with(b"[test marker]\n"));
        assert!(!i.advance(Stage::Clean, Succeeded)); // another file is unresolved
        assert_eq!(
            i.plan(B, Object::MalwareOnly, None),
            Some(FilePlan::Quarantine(Reason::MalwareOnly))
        );
        assert!(i.plan(A, note(), Some(&db)).is_none()); // duplicate
        assert!(i.advance(Stage::Clean, Succeeded));
        assert!(!i.record_disposition(
            A,
            Disposition::Restore {
                stored: CLEAN,
                rollback_saved: true
            },
            Succeeded
        )); // unverified bytes
        assert!(matches!(
            i.verify_file(A, Ok((c.bytes, Verdict::Clean))),
            Some(FileState::ReadyToCommit { .. })
        ));
        for (stored, rollback_saved) in [(DIRTY, true), (CLEAN, false)] {
            assert!(!i.record_disposition(
                A,
                Disposition::Restore {
                    stored,
                    rollback_saved
                },
                Succeeded
            ));
        }
        assert!(!i.record_disposition(
            A,
            Disposition::Restore {
                stored: CLEAN,
                rollback_saved: true
            },
            Failed
        )); // failed atomic commit
        assert!(i.record_disposition(
            A,
            Disposition::Restore {
                stored: CLEAN,
                rollback_saved: true
            },
            Succeeded
        ));
        assert_eq!(i.file(A), Some(&FileState::Restored));
        assert!(!i.record_disposition(B, Disposition::Quarantine, Cancelled));
        assert!(!i.advance(Stage::Repair, Succeeded));
        assert!(i.record_disposition(B, Disposition::Quarantine, Succeeded));
        assert!(!i.advance(Stage::Repair, Failed)); // file results ≠ repaired incident
        assert!(!p.finish_verified(JOB, On, Some(Verdict::Clean)));
        assert!(p
            .incident_mut(JOB, On)
            .unwrap()
            .advance(Stage::Repair, Succeeded));
        assert!(!p.finish_verified(JOB, On, None));
        for v in [Verdict::Monitor, Verdict::Confirmed] {
            assert!(!p.finish_verified(JOB, On, Some(v)));
            assert!(!p.finish_verified(JOB, On, Some(Verdict::Clean))); // stale pass
            assert!(p
                .incident_mut(JOB, On)
                .unwrap()
                .advance(Stage::Repair, Succeeded));
        }
        assert!(!p.finish_verified(JOB, OffByUser, Some(Verdict::Clean)));
        assert!(p.finish_verified(JOB, AlertsMuted, Some(Verdict::Clean)));
        assert!(!p.finish_verified(JOB, On, Some(Verdict::Clean)));
    }

    #[test]
    fn absent_data_and_unsafe_or_unverified_outputs_never_release_original() {
        let db = Catalog::load(DB).unwrap();
        let mut i = cleaning();
        assert_eq!(
            i.plan(A, note(), None),
            Some(FilePlan::Quarantine(Reason::DatabaseUnavailable))
        );
        for (report, reason) in [
            (Err(UnsafeRepair), Reason::CannotRepair),
            (Ok((DIRTY, Verdict::Clean)), Reason::VerificationFailed),
            (Ok((CLEAN, Verdict::Monitor)), Reason::VerificationFailed),
        ] {
            let mut i = cleaning();
            assert!(matches!(
                i.plan(A, note(), Some(&db)),
                Some(FilePlan::Stage(_))
            ));
            assert!(i.advance(Stage::Clean, Succeeded));
            assert_eq!(
                i.verify_file(A, report),
                Some(&FileState::NeedsQuarantine(reason))
            );
            assert!(!i.record_disposition(
                A,
                Disposition::Restore {
                    stored: CLEAN,
                    rollback_saved: true
                },
                Succeeded
            ));
            assert!(i.record_disposition(A, Disposition::Quarantine, Succeeded));
            assert_eq!(i.file(A), Some(&FileState::Quarantined(reason)));
        }
    }

    #[test]
    fn setting_repair_restores_the_journal_not_a_factory_default() {
        use RepairScope::*;
        use View::*;
        assert_eq!(File.repair_view(None), Ok(None));
        for current in [ChangedByFixture, Compact] {
            assert_eq!(
                FileAndSettings.repair_view(Some(ViewJournal {
                    before: Compact,
                    current
                })),
                Ok(Some(Compact))
            );
        }
        assert_eq!(FileAndSettings.repair_view(None), Err(UnsafeRepair));
        assert_eq!(
            FileAndSettings.repair_view(Some(ViewJournal {
                before: Compact,
                current: Normal
            })),
            Err(UnsafeRepair)
        );
    }
}
