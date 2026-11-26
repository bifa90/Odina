// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! odina-core — the part that decides, with no operating system attached.
//!
//! Why this crate exists on its own:
//!
//!   * it can be tested on Linux, on Windows, or in CI, with no VM;
//!   * it cannot block, so it cannot be the reason the app freezes;
//!   * it can be fuzzed without touching a real file system;
//!   * every rule in `docs/spec/scoring-spec-v1.md` has a home here.
//!
//! What it does NOT do: read directories, watch USB, talk to Windows, or
//! spawn threads. Callers supply bytes or normalized activities; it returns
//! file verdicts or policy decisions, never OS side effects.

pub mod archive_guard;
pub mod artifact_verify;
pub mod assessment;
pub mod audit;
pub mod browser_consent;
pub mod diag_export;
pub mod explorer;
pub mod g1_reference;
pub mod hashing;
pub mod heuristics;
pub mod incident;
pub mod installer_integrity;
pub mod ipc_limits;
pub mod k30_entropy_guard;
pub mod k31_pe_header_guard;
pub mod k32_unicode_bidi;
pub mod k33_double_ext;
pub mod k34_quarantine_quota;
pub mod k35_cache_invalidation;
pub mod k36_service_restart;
pub mod k37_cli_exit_codes;
pub mod k38_json_escape;
pub mod k39_date_format;
pub mod k40_temp_dir_scope;
pub mod k41_allowlist_ceiling;
pub mod k42_scan_completeness;
pub mod k43_public_isolation;
pub mod k44_eicar_vector;
pub mod link_guard;
pub mod messages;
pub mod network_scope;
pub mod packaging;
pub mod path_privacy;
pub mod policy;
pub mod privilege;
pub mod proactive;
pub mod publisher_trust;
pub mod quarantine;
pub mod recovery;
pub mod release_age;
pub mod release_gate;
pub mod remediation;
pub mod removable_media;
pub mod repair_db;
pub mod report_submission;
pub mod resource_cap;
pub mod retention;
pub mod sandbox_exec;
pub mod scheduler;
pub mod scoring;
pub mod script_inspect;
pub mod sha256_lookup;
pub mod signature_db;
pub mod telemetry;
pub mod threat_db;
pub mod ui_state;
pub mod update;
pub mod update_check;
pub mod watchdog;

pub use hashing::{FileHashes, HEAD_BYTES, TAIL_BYTES};
pub use heuristics::{guess_path_flags, name_shape, path_score, NameShape, PathFlags};
pub use policy::{ConsentTier, Duration, Exception, Granted, Mode, Policy, Refusal, Rule, Target};
pub use scoring::{
    banded_verdict, score, score_with_known_bad, Cap, Evidence, Indicator, Score, Thresholds,
    Verdict, Veto,
};

/// The crate version, so a report can name the exact engine that produced it.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
