// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! Wall 3: decisions about *new activities*, not whole Windows components.
//! The adapter supplies a matching activity and the authoritative protection
//! mode. Nothing here opens a port, installs a driver, or changes an OS setting.

use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};

use crate::policy::{Mode, Policy, Rule, Target};
use crate::remediation::{FileId, Incident};
use crate::scoring::Verdict;

/// Scope and rationale: protection-policy-v1 §2.3.1. These tiers describe
/// product defaults, not file verdicts or measured exploitation probabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    Mandatory,
    Recommended,
    Optional,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Surface {
    /// Word or Excel launching cmd, PowerShell or mshta; not every Office child.
    OfficeInterpreter,
    /// Creating a permanent WMI event subscription from an untrusted source.
    UntrustedWmiSubscription,
    /// mshta, certutil or regsvr32 consuming untrusted temporary/internet input.
    UntrustedSystemTool,
    /// Creating or changing a scheduled task, not an existing task merely running.
    ScheduledTask,
    /// Changing a per-user COM registration.
    UserComRegistration,
    /// Adding or changing a registry Run/RunOnce startup entry.
    RegistryAutorun,
    /// Creating a service or installing a driver, not stopping existing ones.
    ServiceOrDriverInstall,
    /// A new inbound file-sharing/remote-desktop connection (SMB/RDP).
    InboundSmbRdp,
    /// A new Windows Script Host launch of a .js, .vbs or .wsf file.
    WshScript,
}

impl Surface {
    pub const ALL: [Self; 9] = [
        Self::OfficeInterpreter,
        Self::UntrustedWmiSubscription,
        Self::UntrustedSystemTool,
        Self::ScheduledTask,
        Self::UserComRegistration,
        Self::RegistryAutorun,
        Self::ServiceOrDriverInstall,
        Self::InboundSmbRdp,
        Self::WshScript,
    ];

    /// Stable settings IDs; localized labels are never persisted as preferences.
    pub const fn key(self) -> &'static str {
        match self {
            Self::OfficeInterpreter => "office_child",
            Self::UntrustedWmiSubscription => "wmi_subscription",
            Self::UntrustedSystemTool => "system_tools",
            Self::ScheduledTask => "scheduled_task",
            Self::UserComRegistration => "com_registration",
            Self::RegistryAutorun => "registry_autorun",
            Self::ServiceOrDriverInstall => "service_driver",
            Self::InboundSmbRdp => "inbound_smb_rdp",
            Self::WshScript => "wsh_script",
        }
    }

    pub const fn tier(self) -> Tier {
        match self {
            Self::OfficeInterpreter
            | Self::UntrustedWmiSubscription
            | Self::UntrustedSystemTool => Tier::Mandatory,
            Self::ScheduledTask
            | Self::UserComRegistration
            | Self::RegistryAutorun
            | Self::ServiceOrDriverInstall => Tier::Recommended,
            Self::InboundSmbRdp | Self::WshScript => Tier::Optional,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtectionMode {
    On,
    AlertsMuted,
    OffByUser,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockReason {
    Mandatory,
    UserChoice,
    Containment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Do not initiate or re-enable any enforcement on the user's behalf.
    OffByUser,
    Block(BlockReason),
    /// No block from this layer, NOT permission to ignore other walls, AMSI,
    /// Windows Firewall or a file verdict. Only ordinary WSH overlaps D45.
    ContinueChecks {
        wsh_exception: Option<Mode>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreferenceError {
    Mandatory,
    IncidentOpen,
}

/// Owned by the scan/remediation coordinator. IDs must be unique per job,
/// including retries of a *new* job; follow-up events carry the original ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct JobId(pub u64);

#[derive(Debug, Default, Clone)]
pub struct SurfacePolicy {
    // Persist preferences separately from unresolved jobs. Never save an
    // effective incident block as if it were a user's choice.
    user_blocks: BTreeSet<Surface>,
    jobs: BTreeMap<JobId, Incident>,
}

impl SurfacePolicy {
    /// Independent of active jobs; a matching mandatory block always wins.
    pub fn configured_block(&self, matched: &[Surface]) -> Option<BlockReason> {
        if matched.iter().any(|s| s.tier() == Tier::Mandatory) {
            Some(BlockReason::Mandatory)
        } else if matched.iter().any(|s| self.user_blocks.contains(s)) {
            Some(BlockReason::UserChoice)
        } else {
            None
        }
    }

    pub fn set_blocked(&mut self, surface: Surface, blocked: bool) -> Result<(), PreferenceError> {
        if surface.tier() == Tier::Mandatory {
            return Err(PreferenceError::Mandatory);
        }
        if !self.jobs.is_empty() {
            return Err(PreferenceError::IncidentOpen);
        }
        if blocked {
            self.user_blocks.insert(surface);
        } else {
            self.user_blocks.remove(&surface);
        }
        Ok(())
    }

    /// Observe a verdict *during the infection scan*. A routine scan, a
    /// suspicion or a delayed notification while explicitly off cannot lock
    /// the system. Multiple confirmed files in the same job acquire one hold.
    pub fn observe_file(
        &mut self,
        id: JobId,
        file: FileId,
        verdict: Verdict,
        mode: ProtectionMode,
    ) -> bool {
        if verdict != Verdict::Confirmed || mode == ProtectionMode::OffByUser {
            return false;
        }
        match self.jobs.entry(id) {
            Entry::Vacant(job) => {
                job.insert(Incident::new(file));
                true
            }
            Entry::Occupied(mut job) => job.get_mut().include(file),
        }
    }

    pub fn incident(&self, id: JobId) -> Option<&Incident> {
        self.jobs.get(&id)
    }
    pub fn incident_mut(&mut self, id: JobId, mode: ProtectionMode) -> Option<&mut Incident> {
        if mode == ProtectionMode::OffByUser {
            return None;
        }
        self.jobs.get_mut(&id)
    }
    /// A clean final incident rescan (after repair), not a single clean file,
    /// is the only event that releases this job. On a new finding, register a
    /// new job; on failed/cancelled verification, retain this job and retry.
    pub fn finish_verified(
        &mut self,
        id: JobId,
        mode: ProtectionMode,
        rescan: Option<Verdict>,
    ) -> bool {
        let Some(job) = self.incident_mut(id, mode) else {
            return false;
        };
        if !job.verify_incident(rescan) {
            return false;
        }
        self.jobs.remove(&id);
        true
    }
}

impl Policy {
    /// Resolve a matching event *before* considering any per-target exception.
    /// The trusted adapter supplies *all* matching surfaces, possibly none. Mode is
    /// an input, never a side effect of restoring preferences after repair.
    pub fn evaluate_activity(
        &self,
        matched: &[Surface],
        mode: ProtectionMode,
        target: &Target,
        now_ms: i64,
    ) -> Decision {
        if mode == ProtectionMode::OffByUser {
            return Decision::OffByUser;
        }
        if !matched.is_empty() && !self.surfaces.jobs.is_empty() {
            return Decision::Block(BlockReason::Containment);
        }
        if let Some(reason) = self.surfaces.configured_block(matched) {
            return Decision::Block(reason);
        }
        let wsh_exception = if matched.contains(&Surface::WshScript) {
            self.effective(Rule::WindowsScriptHost, target, now_ms)
        } else {
            None
        };
        Decision::ContinueChecks { wsh_exception }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::{ConsentTier, Duration};
    use crate::remediation::{Disposition, Object, Stage, Wall, WallResult};
    use BlockReason::*;
    use Outcome::*;
    use ProtectionMode::*;
    use Stage::*;

    fn complete(p: &mut Policy, job: JobId) {
        let i = p.surfaces.incident_mut(job, On).unwrap();
        for w in Wall::ORDER {
            assert!(i.record_wall(w, WallResult::Checked));
        }
        assert!(i.advance(Scan, Succeeded));
        assert!(i.advance(Neutralize, Succeeded));
        i.plan(FileId(0), Object::MalwareOnly, None).unwrap();
        assert!(i.advance(Clean, Succeeded));
        assert!(i.record_disposition(FileId(0), Disposition::Quarantine, Succeeded));
        assert!(i.advance(Repair, Succeeded));
        assert!(p.surfaces.finish_verified(job, On, Some(Verdict::Clean)));
    }

    const A: JobId = JobId(17);
    const B: JobId = JobId(18);

    fn decisions(p: &Policy, mode: ProtectionMode) -> Vec<Decision> {
        Surface::ALL
            .iter()
            .map(|s| p.evaluate_activity(&[*s], mode, &Target::Everything, 0))
            .collect()
    }
    fn all_contained(p: &Policy) {
        assert_eq!(decisions(p, On), vec![Decision::Block(Containment); 9]);
    }

    #[test]
    fn defaults_and_catalog_are_scoped_and_critical_blocks_are_locked() {
        let mut p = Policy::new();
        // The same acceptance fixture is exercised against the staged UI.
        let fixture = include_str!("../tests/fixtures/proactive-tiers.tsv");
        assert_eq!(fixture.lines().count(), Surface::ALL.len());
        for (s, line) in Surface::ALL.into_iter().zip(fixture.lines()) {
            let (key, tier) = line.split_once('\t').unwrap();
            assert_eq!(s.key(), key);
            let tier = match tier {
                "mandatory" => Tier::Mandatory,
                "recommended" => Tier::Recommended,
                "optional" => Tier::Optional,
                other => panic!("unknown tier {other}"),
            };
            assert_eq!(s.tier(), tier);
            let expected = if tier == Tier::Mandatory {
                assert_eq!(
                    p.surfaces.set_blocked(s, false),
                    Err(PreferenceError::Mandatory)
                );
                Decision::Block(Mandatory)
            } else {
                Decision::ContinueChecks {
                    wsh_exception: None,
                }
            };
            assert_eq!(
                p.evaluate_activity(&[s], On, &Target::Everything, 0),
                expected
            );
        }
    }

    #[test]
    fn only_confirmed_scans_lock_and_only_successful_repair_restores_user_choices() {
        let mut p = Policy::new();
        p.surfaces
            .set_blocked(Surface::RegistryAutorun, true)
            .unwrap();
        p.surfaces
            .set_blocked(Surface::InboundSmbRdp, true)
            .unwrap();
        let before = decisions(&p, On);
        for v in [
            Verdict::Clean,
            Verdict::Monitor,
            Verdict::Suspicious,
            Verdict::HighRisk,
        ] {
            p.surfaces.observe_file(A, FileId(0), v, On);
            assert_eq!(decisions(&p, On), before);
        }
        p.surfaces
            .observe_file(A, FileId(0), Verdict::Confirmed, AlertsMuted);
        assert_eq!(
            decisions(&p, AlertsMuted),
            vec![Decision::Block(Containment); 9]
        );
        assert_eq!(
            p.surfaces.set_blocked(Surface::ScheduledTask, true),
            Err(PreferenceError::IncidentOpen)
        );
        assert!(!p.surfaces.finish_verified(A, On, Some(Verdict::Clean)));
        for stage in [Scan, Neutralize, Clean, Repair] {
            p.surfaces
                .observe_file(A, FileId(0), Verdict::Confirmed, On);
            for outcome in [Failed, Cancelled] {
                assert!(!p
                    .surfaces
                    .incident_mut(A, On)
                    .unwrap()
                    .advance(stage, outcome));
                all_contained(&p);
            }
            let i = p.surfaces.incident_mut(A, On).unwrap();
            if stage == Scan {
                for w in Wall::ORDER {
                    assert!(i.record_wall(w, WallResult::Checked));
                }
            }
            if stage == Clean {
                i.plan(FileId(0), Object::MalwareOnly, None).unwrap();
            }
            if stage == Repair {
                assert!(i.record_disposition(FileId(0), Disposition::Quarantine, Succeeded));
            }
            assert!(i.advance(stage, Succeeded));
            assert!(!i.advance(stage, Succeeded));
        }
        all_contained(&p);
        assert!(p.surfaces.finish_verified(A, On, Some(Verdict::Clean)));
        assert!(p.surfaces.incident(A).is_none());
        assert_eq!(decisions(&p, On), before);
    }

    #[test]
    fn a_block_on_any_match_wins_but_unrelated_activities_are_not_frozen() {
        let mut p = Policy::new();
        let target = Target::Everything;
        let optional = Surface::WshScript;
        let critical = Surface::OfficeInterpreter;
        p.surfaces.set_blocked(optional, true).unwrap();
        for matched in [[optional, critical], [critical, optional]] {
            assert_eq!(
                p.evaluate_activity(&matched, On, &target, 0),
                Decision::Block(Mandatory)
            );
        }
        let verdict = crate::scoring::score_with_known_bad(
            &Default::default(),
            &Default::default(),
            None,
            true,
        )
        .verdict;
        p.surfaces.observe_file(A, FileId(0), verdict, On);
        all_contained(&p);
        assert_eq!(
            p.evaluate_activity(&[], On, &target, 0),
            Decision::ContinueChecks {
                wsh_exception: None
            }
        );
    }

    #[test]
    fn overlapping_jobs_and_explicit_off_are_independent_of_completion() {
        let mut p = Policy::new();
        let before = decisions(&p, On);
        p.surfaces
            .observe_file(A, FileId(0), Verdict::Confirmed, OffByUser);
        assert!(p.surfaces.incident(A).is_none());
        p.surfaces
            .observe_file(A, FileId(0), Verdict::Confirmed, On);
        p.surfaces
            .observe_file(B, FileId(0), Verdict::Confirmed, On);
        complete(&mut p, A);
        all_contained(&p);
        assert!(!p.surfaces.finish_verified(A, On, Some(Verdict::Clean)));
        assert!(p.surfaces.incident_mut(B, OffByUser).is_none());
        assert_eq!(decisions(&p, OffByUser), vec![Decision::OffByUser; 9]);
        complete(&mut p, B);
        assert_eq!(decisions(&p, OffByUser), vec![Decision::OffByUser; 9]);
        assert_eq!(decisions(&p, On), before);
    }

    #[test]
    fn containment_and_user_block_outrank_wsh_grants_without_extending_their_expiry() {
        let mut p = Policy::new();
        let target = Target::Everything;
        p.grant(
            Rule::WindowsScriptHost,
            target.clone(),
            Mode::Allow,
            Duration::Minutes(10),
            ConsentTier::Expert,
            0,
        )
        .unwrap();
        let wsh = Surface::WshScript;
        assert_eq!(
            p.evaluate_activity(&[wsh], On, &target, 0),
            Decision::ContinueChecks {
                wsh_exception: Some(Mode::Allow)
            }
        );
        assert_eq!(
            p.evaluate_activity(&[Surface::OfficeInterpreter], On, &target, 0),
            Decision::Block(Mandatory)
        );
        p.surfaces.set_blocked(wsh, true).unwrap();
        assert_eq!(
            p.evaluate_activity(&[wsh], On, &target, 0),
            Decision::Block(UserChoice)
        );
        p.surfaces.set_blocked(wsh, false).unwrap();
        p.surfaces
            .observe_file(A, FileId(0), Verdict::Confirmed, On);
        all_contained(&p);
        complete(&mut p, A);
        assert_eq!(
            p.evaluate_activity(&[wsh], On, &target, 600_000),
            Decision::ContinueChecks {
                wsh_exception: None
            }
        );
    }
}
