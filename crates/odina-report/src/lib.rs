// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! `odina-report/v1` — the sixteen-field report from
//! `docs/spec/report-schema-v1.md`, built so that the schema cannot drift away
//! from the code.
//!
//! Why this crate exists:
//!
//!   * the schema says **sixteen fields, and no seventeenth** — that promise is
//!     only worth something if a machine checks it, and here a test does;
//!   * `name_pattern` and `path_pattern` are the field where a leak would
//!     happen, so they live in their own module with their own tests;
//!   * the report must never carry a timestamp, a user name, a machine id or a
//!     path, and the only way to be sure is to never build one that could.
//!
//! What it does NOT do: read files, look at the clock, touch the network, or
//! store anything. Callers hand it measurements; it hands back JSON.
//!
//! The one thing it refuses to do is guess: a field it cannot measure is
//! absent, not zero, not empty, not "unknown".

pub mod hw;
pub mod measure;
pub mod pattern;
pub mod pe;
pub mod report;

pub use measure::Attributes;
pub use pattern::{name_pattern, path_pattern};
pub use report::{Report, ReportError, MAX_REPORT_BYTES, REPORT_SCHEMA};

/// Crate version, so a report can name the exact writer that produced it.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
