// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! Read-only inspection of the separately installed data component. Never
//! discover recipes in the working directory or mistake them for signatures.

use odina_core::proactive::{JobId, Outcome::Succeeded, ProtectionMode::On, SurfacePolicy};
use odina_core::remediation::{
    Disposition, FileId, FilePlan, FileState, Object, Stage, View, ViewJournal, Wall, WallResult,
};
use odina_core::repair_db::{Catalog, Format, Reason, COMPONENT_PATH, MAX_CATALOG_BYTES};
use odina_core::Verdict;
use std::io::Read;
use std::path::Path;
use std::process::ExitCode;

fn load(path: &Path) -> Result<Catalog, String> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .and_then(|f| f.take(MAX_CATALOG_BYTES as u64 + 1).read_to_end(&mut bytes))
        .map_err(|err| format!("{}: {err}", path.display()))?;
    Catalog::load(&bytes).map_err(|err| format!("catalog rejected: {err:?}"))
}

pub(super) fn command(args: &[String]) -> ExitCode {
    let demo = match args {
        [] => false,
        [flag] if flag == "--demo" => true,
        _ => {
            eprintln!("usage: odina repair-db [--demo]");
            return ExitCode::FAILURE;
        }
    };
    let db = std::env::current_exe()
        .map_err(|e| e.to_string())
        .and_then(|exe| {
            let dir = exe.parent().ok_or("no program directory")?;
            load(&dir.join(COMPONENT_PATH))
        });
    match db {
        Ok(db) => {
            println!("repair database v1/r1: pin verified; {} training recipes (not detection signatures)", db.recipe_count());
            if !demo { return ExitCode::SUCCESS; }
            if selftest(&db) {
                println!("Incident repair verified; temporary containment released.");
                return ExitCode::SUCCESS;
            }
            eprintln!("repair demo did not reach a verified outcome");
        }
        Err(reason) => eprintln!("{reason}\nNo recipe will run. Extract odina-repair-v1-r1.zip from the matching release beside the executable."),
    }
    ExitCode::FAILURE
}

// Use the normal policy, the external catalog and a known-good independent
// fixture oracle; the wall/OS acknowledgements here are explicitly SIMULATED.
fn selftest(db: &Catalog) -> bool {
    println!("In-memory simulation only. No real files, settings or processes are changed.");
    let mut policy = SurfacePolicy::default();
    let job = JobId(1);
    let files = [
        (
            "note",
            Object::Host {
                family: "demo.note",
                format: Format::Text,
                original: b"Odina training note.\n[test marker]\n",
                backup: None,
            },
        ),
        (
            "settings",
            Object::Host {
                family: "demo.settings",
                format: Format::Settings,
                original: b"view=changed\n",
                backup: Some(b"view=normal\n"),
            },
        ),
        ("standalone", Object::MalwareOnly),
        ("unsupported", Object::UnsupportedHost),
    ];
    for n in 0..files.len() {
        policy.observe_file(job, FileId(n as u64), Verdict::Confirmed, On);
    }
    let Some(i) = policy.incident_mut(job, On) else {
        return false;
    };
    for w in Wall::ORDER {
        if !i.record_wall(w, WallResult::Checked) {
            return false;
        }
    }
    if !i.advance(Stage::Scan, Succeeded) || !i.advance(Stage::Neutralize, Succeeded) {
        return false;
    }
    let plans: Vec<_> = files
        .into_iter()
        .enumerate()
        .map(|(n, (name, obj))| {
            let id = FileId(n as u64);
            (name, id, i.plan(id, obj, Some(db)))
        })
        .collect();
    if !i.advance(Stage::Clean, Succeeded) {
        return false;
    }
    for (name, id, plan) in plans {
        match (name, plan) {
            ("note" | "settings", Some(FilePlan::Stage(c))) => {
                let clean: &[u8] = if name == "note" {
                    b"Odina training note.\n"
                } else {
                    b"view=normal\n"
                };
                let journal = ViewJournal {
                    before: View::Compact,
                    current: View::ChangedByFixture,
                };
                if c.bytes != clean
                    || c.scope.repair_view(Some(journal))
                        != Ok((name == "settings").then_some(View::Compact))
                    || !matches!(
                        i.verify_file(id, Ok((c.bytes, Verdict::Clean))),
                        Some(FileState::ReadyToCommit { .. })
                    )
                    || !i.record_disposition(
                        id,
                        Disposition::Restore {
                            stored: c.bytes,
                            rollback_saved: true,
                        },
                        Succeeded,
                    )
                {
                    return false;
                }
                println!("{name}: disinfected, repaired and verified");
            }
            ("standalone", Some(FilePlan::Quarantine(Reason::MalwareOnly)))
            | ("unsupported", Some(FilePlan::Quarantine(Reason::NoSafeRule))) => {
                if !i.record_disposition(id, Disposition::Quarantine, Succeeded) {
                    return false;
                }
                println!("{name}: quarantine; no deletion");
            }
            _ => return false,
        }
    }
    i.advance(Stage::Repair, Succeeded) && policy.finish_verified(job, On, Some(Verdict::Clean))
}
