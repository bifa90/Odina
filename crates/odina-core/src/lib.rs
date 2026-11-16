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
//! spawn threads. Callers hand it bytes; it hands back verdicts.

pub mod hashing;
pub mod scoring;

pub use hashing::{FileHashes, HEAD_BYTES, TAIL_BYTES};
pub use scoring::{
    banded_verdict, score, Cap, Evidence, Indicator, Score, Thresholds, Verdict, Veto,
};

/// The crate version, so a report can name the exact engine that produced it.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
