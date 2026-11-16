// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! The measurement layer of the **instrumented build** — §34 of
//! `docs/odina-agent-prompt-v17.md`.
//!
//! Three promises hold this crate together, and each one has a test:
//!
//!   1. **Nothing leaves the machine.** There is no socket, no URL, no client
//!      in this crate or its dependencies. The output is a file on disk that
//!      the user chooses to send.
//!   2. **No user data is measured.** Counts, durations, sizes and verdict
//!      categories. No paths, no names, no hashes, no times of day — those
//!      exist only in the opt-in `odina-report` objects, which the user
//!      inspects field by field before sending.
//!   3. **A gate that was not measured is reported as unmeasured**, with the
//!      reason, never as a pass and never as a zero.
//!
//! The third promise is the one that makes this layer worth trusting: a report
//! full of `0` values looks like a perfect run, and that is exactly how
//! instrumented builds lie.

pub mod meter;
pub mod perf;

pub use meter::{cpu_time, io_ops, rss_kb, Meter};
pub use perf::{Gate, Measurement, Metrics, PerfReport, ScanStats, PERF_SCHEMA};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
