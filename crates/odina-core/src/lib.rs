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

pub mod assessment;
pub mod g1_reference;
pub mod hashing;
pub mod heuristics;
pub mod policy;
pub mod proactive;
pub mod quarantine;
pub mod remediation;
pub mod repair_db;
pub mod scoring;

pub use hashing::{FileHashes, HEAD_BYTES, TAIL_BYTES};
pub use heuristics::{guess_path_flags, name_shape, path_score, NameShape, PathFlags};
pub use policy::{ConsentTier, Duration, Exception, Granted, Mode, Policy, Refusal, Rule, Target};
pub use scoring::{
    banded_verdict, score, score_with_known_bad, Cap, Evidence, Indicator, Score, Thresholds,
    Verdict, Veto,
};

/// The crate version, so a report can name the exact engine that produced it.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
