// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! The performance gates of §34.2, as code rather than a table in a document.
//!
//! Every gate has a threshold and three possible outcomes: **pass**, **fail**,
//! or **unmeasured with a reason**. The third one is the reason this file
//! exists. A table in a document cannot express "we could not measure this on
//! this machine", so the report quietly fills in numbers, and a report full of
//! numbers looks like a project that measured things.
//!
//! Here, an unmeasured gate is a first-class value that carries its reason, and
//! the JSON says `"status":"unmeasured","reason":"…"` — the maintainer reading
//! it knows exactly which gates are still hypotheses.

use crate::meter::Meter;

/// The perf file schema string, so a reader can refuse a version it does not
/// understand instead of guessing.
pub const PERF_SCHEMA: &str = "odina-perf/v1";

/// One gate from §34.2. The numbers live here, once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gate {
    /// RAM with the app idle, MB. `<= 20`.
    RamIdleMb,
    /// RAM while scanning a USB stick, MB. `<= 50`.
    RamUsbMb,
    /// CPU with the app idle, percent. `== 0` (within rounding).
    CpuIdlePercent,
    /// CPU average during a scan, percent. `< 3`.
    CpuScanAvgPercent,
    /// CPU peak during a scan, percent. `<= 5`.
    CpuScanPeakPercent,
    /// Time from a USB insert to the app noticing, ms. `< 50`.
    UsbWakeMs,
    /// Time to a P0 (dangerous) decision, ms. `< 3000`.
    P0Ms,
    /// Time to a P1 decision, ms. `< 15000`.
    P1Ms,
    /// Files per second, no threshold — recorded as a baseline.
    ThroughputFilesPerSec,
    /// Extra time AMSI adds per script, ms. `< 15`.
    AmsiMs,
    /// Time for a WDAC policy update to take effect, ms. Recorded, not gated:
    /// §34.2 says "تُسجَّل" — there is no line to pass or fail, only a number
    /// worth knowing before promising a user that a block is instant.
    WdacPolicyMs,
    /// Time to switch interface language, ms. `< 100`.
    LanguageSwitchMs,
    /// Memory per loaded language pack, MB. `<= 3`.
    LanguageMemoryMb,
    /// I/O operations per minute while idle. `== 0`.
    IdleIoOpsPerMin,
}

impl Gate {
    /// Every gate, in the order §34.2 lists them.
    pub const ALL: [Gate; 14] = [
        Gate::RamIdleMb,
        Gate::RamUsbMb,
        Gate::CpuIdlePercent,
        Gate::CpuScanAvgPercent,
        Gate::CpuScanPeakPercent,
        Gate::UsbWakeMs,
        Gate::P0Ms,
        Gate::P1Ms,
        Gate::ThroughputFilesPerSec,
        Gate::AmsiMs,
        Gate::WdacPolicyMs,
        Gate::LanguageSwitchMs,
        Gate::LanguageMemoryMb,
        Gate::IdleIoOpsPerMin,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Gate::RamIdleMb => "ram_idle_mb",
            Gate::RamUsbMb => "ram_usb_mb",
            Gate::CpuIdlePercent => "cpu_idle_percent",
            Gate::CpuScanAvgPercent => "cpu_scan_avg_percent",
            Gate::CpuScanPeakPercent => "cpu_scan_peak_percent",
            Gate::UsbWakeMs => "usb_wake_ms",
            Gate::P0Ms => "p0_ms",
            Gate::P1Ms => "p1_ms",
            Gate::ThroughputFilesPerSec => "throughput_files_per_sec",
            Gate::AmsiMs => "amsi_ms",
            Gate::WdacPolicyMs => "wdac_policy_ms",
            Gate::LanguageSwitchMs => "language_switch_ms",
            Gate::LanguageMemoryMb => "language_memory_mb",
            Gate::IdleIoOpsPerMin => "idle_io_ops_per_min",
        }
    }

    pub fn unit(self) -> &'static str {
        match self {
            Gate::RamIdleMb | Gate::RamUsbMb | Gate::LanguageMemoryMb => "MB",
            Gate::CpuIdlePercent | Gate::CpuScanAvgPercent | Gate::CpuScanPeakPercent => "%",
            Gate::UsbWakeMs
            | Gate::P0Ms
            | Gate::P1Ms
            | Gate::AmsiMs
            | Gate::WdacPolicyMs
            | Gate::LanguageSwitchMs => "ms",
            Gate::ThroughputFilesPerSec => "files/s",
            Gate::IdleIoOpsPerMin => "ops/min",
        }
    }

    /// The threshold, if this gate has one. Throughput is a baseline, not a
    /// pass/fail line, and saying so in the type is better than a magic 0.
    pub fn threshold(self) -> Option<Threshold> {
        use Threshold::*;
        Some(match self {
            Gate::RamIdleMb => AtMost(20.0),
            Gate::RamUsbMb => AtMost(50.0),
            Gate::CpuIdlePercent => AtMost(0.1), // D52: the policy line, not a rounding allowance
            Gate::CpuScanAvgPercent => Below(3.0),
            Gate::CpuScanPeakPercent => AtMost(5.0),
            Gate::UsbWakeMs => Below(50.0),
            Gate::P0Ms => Below(3000.0),
            Gate::P1Ms => Below(15000.0),
            Gate::ThroughputFilesPerSec | Gate::WdacPolicyMs => return None,
            Gate::AmsiMs => AtMost(15.0), // D66 says ≤, so the boundary passes
            Gate::LanguageSwitchMs => Below(100.0),
            Gate::LanguageMemoryMb => AtMost(3.0),
            Gate::IdleIoOpsPerMin => AtMost(0.0),
        })
    }

    /// Which spec line this gate comes from, so a failure cites its rule.
    pub fn spec(self) -> &'static str {
        match self {
            Gate::RamIdleMb => "§34.2 RAM خمول < 20",
            Gate::RamUsbMb => "§34.2 RAM أثناء فحص USB ≤ 50",
            Gate::CpuIdlePercent => "D52 CPU خمول ≤ 0.1%",
            Gate::CpuScanAvgPercent => "D53 CPU فحص خلفي < 3%",
            Gate::CpuScanPeakPercent => "D53 CPU فحص USB ذروة ≤ 5%",
            Gate::UsbWakeMs => "§34.2 زمن استيقاظ USB < 50ms",
            Gate::P0Ms => "§34.2 زمن P0 < 3000ms",
            Gate::P1Ms => "§34.2 زمن P1 < 15000ms",
            Gate::ThroughputFilesPerSec => "§34.2 خط أساس الإنتاجية",
            Gate::AmsiMs => "D66 AMSI ≤ 15ms لكل سكربت",
            Gate::WdacPolicyMs => "§34.2 زمن تحديث سياسة WDAC (يُسجَّل)",
            Gate::LanguageSwitchMs => "D85 تبديل اللغة < 100ms",
            Gate::LanguageMemoryMb => "§34.2 ذاكرة اللغة ≤ 3MB",
            Gate::IdleIoOpsPerMin => "§34.2 I/O خمول = 0",
        }
    }
}

/// `AtMost` includes the boundary, `Below` excludes it — the spec uses both
/// forms deliberately and silently rounding one into the other changes a pass
/// into a fail at exactly the boundary value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Threshold {
    AtMost(f64),
    Below(f64),
}

/// The outcome of one gate.
#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Pass,
    Fail,
    /// Not measured, with the reason. Never silently omitted.
    Unmeasured(String),
}

/// One measured (or refused) gate.
#[derive(Debug, Clone, PartialEq)]
pub struct Measurement {
    pub gate: Gate,
    pub value: Option<f64>,
    pub status: Status,
}

impl Measurement {
    /// A value measured on this machine.
    pub fn measured(gate: Gate, value: f64) -> Self {
        let status = match gate.threshold() {
            None => Status::Pass, // a baseline has no line to cross
            Some(Threshold::AtMost(t)) => {
                if value <= t {
                    Status::Pass
                } else {
                    Status::Fail
                }
            }
            Some(Threshold::Below(t)) => {
                if value < t {
                    Status::Pass
                } else {
                    Status::Fail
                }
            }
        };
        Measurement {
            gate,
            value: Some(value),
            status,
        }
    }

    /// A gate this run did not measure. The reason is shown to the maintainer,
    /// so it has to say what is missing, not that something is.
    pub fn unmeasured(gate: Gate, reason: impl Into<String>) -> Self {
        Measurement {
            gate,
            value: None,
            status: Status::Unmeasured(reason.into()),
        }
    }
}

/// Aggregate counters for a scan run. Counts only: no path, no name, no hash.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ScanStats {
    pub files: u64,
    pub bytes: u64,
    pub clean: u64,
    pub monitor: u64,
    pub suspicious: u64,
    pub high_risk: u64,
    pub confirmed: u64,
    pub unreadable: u64,
    /// Links and special files the walk refused to follow or open. Not
    /// failures: a scan that hangs measures nothing, so the number is how many
    /// surprises the folder had.
    pub links_skipped: u64,
    /// Folders the walk could not read at all.
    pub read_errors: u64,
    /// The walk stopped at its own limit. A partial answer labelled partial is
    /// usable; a partial answer that looks complete is not.
    pub truncated: bool,
    pub elapsed_ms: u64,
}

impl ScanStats {
    pub fn throughput_files_per_sec(&self) -> Option<f64> {
        if self.elapsed_ms == 0 || self.files == 0 {
            return None;
        }
        Some(self.files as f64 / (self.elapsed_ms as f64 / 1000.0))
    }

    /// Every verdict band must be accounted for; a scan that loses files in
    /// the counters cannot be used to tune weights.
    pub fn accounted(&self) -> bool {
        let accounted = self.clean
            + self.monitor
            + self.suspicious
            + self.high_risk
            + self.confirmed
            + self.unreadable;
        accounted == self.files
    }
}

/// The whole instrumented run, ready to write as one file.
#[derive(Debug, Clone)]
pub struct PerfReport {
    /// ISO date the run happened, supplied by the caller. This crate has no
    /// clock on purpose — a timestamp read here would be a hidden dependency
    /// on when the user ran something.
    pub date: String,
    /// Odina version that produced the numbers.
    pub app_version: String,
    /// CPU label and core count from `odina_report::hw`.
    pub hardware: String,
    /// One platform line: `windows 11 · x86_64`.
    pub platform: String,
    pub metrics: Vec<Measurement>,
    pub scan: Option<ScanStats>,
}

impl PerfReport {
    pub fn new(date: &str, app_version: &str, hardware: &str, platform: &str) -> Self {
        PerfReport {
            date: date.to_string(),
            app_version: app_version.to_string(),
            hardware: hardware.to_string(),
            platform: platform.to_string(),
            metrics: Vec::new(),
            scan: None,
        }
    }

    pub fn push(&mut self, m: Measurement) {
        self.metrics.push(m);
    }

    /// Fill the standard set of measurements from a scan, marking every gate
    /// this run genuinely cannot measure. This is the default path for the
    /// `check` build: no pretending, no quiet zeros.
    pub fn fill_from_scan(&mut self, stats: ScanStats, idle: &Meter, scan_peak: Option<&Meter>) {
        self.scan = Some(stats);

        if let Some(mb) = idle.rss_kb.map(|kb| kb as f64 / 1024.0) {
            self.push(Measurement::measured(Gate::RamIdleMb, round2(mb)));
        } else {
            self.push(Measurement::unmeasured(
                Gate::RamIdleMb,
                "the platform does not expose resident memory without extra permissions",
            ));
        }

        if let Some(p) = stats.throughput_files_per_sec() {
            self.push(Measurement::measured(
                Gate::ThroughputFilesPerSec,
                round2(p),
            ));
        } else {
            self.push(Measurement::unmeasured(
                Gate::ThroughputFilesPerSec,
                "the run was too short to produce a throughput figure",
            ));
        }

        if let Some(m) = scan_peak {
            if let Some(pct) = m.cpu_percent_of(idle) {
                self.push(Measurement::measured(Gate::CpuScanAvgPercent, round2(pct)));
            } else {
                self.push(Measurement::unmeasured(
                    Gate::CpuScanAvgPercent,
                    "the CPU counter is not readable on this platform",
                ));
            }
        } else {
            self.push(Measurement::unmeasured(
                Gate::CpuScanAvgPercent,
                "no second sample was taken during the scan",
            ));
        }

        // Gates that belong to parts of the product that do not exist yet.
        // Each says which part, so the list doubles as a build-order reminder.
        self.push(Measurement::unmeasured(
            Gate::CpuIdlePercent,
            "needs the resident app (B1); the CLI exits before going idle",
        ));
        self.push(Measurement::unmeasured(
            Gate::RamUsbMb,
            "needs USB hot-trigger (C1)",
        ));
        self.push(Measurement::unmeasured(
            Gate::CpuScanPeakPercent,
            "needs a sampler thread; the single-pass CLI measures the average only",
        ));
        self.push(Measurement::unmeasured(
            Gate::UsbWakeMs,
            "needs USB hot-trigger (C1)",
        ));
        self.push(Measurement::unmeasured(
            Gate::P0Ms,
            "needs a verdict path (C1)",
        ));
        self.push(Measurement::unmeasured(
            Gate::P1Ms,
            "needs a verdict path (C1)",
        ));
        self.push(Measurement::unmeasured(
            Gate::AmsiMs,
            "needs the AMSI provider (C5)",
        ));
        self.push(Measurement::unmeasured(
            Gate::WdacPolicyMs,
            "needs WDAC policy publishing (F4); nothing deploys a policy yet",
        ));
        self.push(Measurement::unmeasured(
            Gate::LanguageSwitchMs,
            "needs the UI (B1); switching a CLI language is not the measured path",
        ));
        self.push(Measurement::unmeasured(
            Gate::LanguageMemoryMb,
            "needs the UI (B1)",
        ));
        self.push(Measurement::unmeasured(
            Gate::IdleIoOpsPerMin,
            "needs the resident app (B1); a one-shot scan is never idle",
        ));
    }

    /// Gates that passed / failed / were not measured, for the summary line.
    pub fn tally(&self) -> (usize, usize, usize) {
        let mut pass = 0;
        let mut fail = 0;
        let mut unmeasured = 0;
        for m in &self.metrics {
            match m.status {
                Status::Pass => pass += 1,
                Status::Fail => fail += 1,
                Status::Unmeasured(_) => unmeasured += 1,
            }
        }
        (pass, fail, unmeasured)
    }

    /// One-line verdict a human can read before opening the JSON.
    pub fn summary(&self) -> String {
        let (pass, fail, unmeasured) = self.tally();
        format!("{pass} pass · {fail} fail · {unmeasured} unmeasured")
    }

    /// Serialise to the perf JSON. Stable key order; no user data anywhere.
    pub fn to_json(&self) -> String {
        let mut out = String::with_capacity(2048);
        out.push('{');
        out.push_str(&format!("\"schema\":{}", q(PERF_SCHEMA)));
        out.push_str(&format!(",\"date\":{}", q(&self.date)));
        out.push_str(&format!(",\"app_version\":{}", q(&self.app_version)));
        out.push_str(&format!(",\"hardware\":{}", q(&self.hardware)));
        out.push_str(&format!(",\"platform\":{}", q(&self.platform)));
        out.push_str(",\"metrics\":[");
        for (i, m) in self.metrics.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push('{');
            out.push_str(&format!("\"gate\":{}", q(m.gate.key())));
            match (&m.status, m.value) {
                (Status::Unmeasured(reason), _) => {
                    out.push_str(",\"status\":\"unmeasured\"");
                    out.push_str(&format!(",\"reason\":{}", q(reason)));
                }
                (Status::Pass, Some(v)) | (Status::Fail, Some(v)) => {
                    let s = match m.status {
                        Status::Pass => "pass",
                        _ => "fail",
                    };
                    out.push_str(&format!(",\"status\":\"{s}\""));
                    out.push_str(&format!(",\"value\":{v}"));
                    out.push_str(&format!(",\"unit\":{}", q(m.gate.unit())));
                    match m.gate.threshold() {
                        Some(Threshold::AtMost(t)) => out.push_str(&format!(",\"at_most\":{t}")),
                        Some(Threshold::Below(t)) => out.push_str(&format!(",\"below\":{t}")),
                        None => {}
                    }
                    out.push_str(&format!(",\"spec\":{}", q(m.gate.spec())));
                }
                (_, None) => {
                    out.push_str(",\"status\":\"unmeasured\",\"reason\":\"no value\"");
                }
            }
            out.push('}');
        }
        out.push(']');
        if let Some(s) = &self.scan {
            out.push_str(",\"scan\":{");
            out.push_str(&format!("\"files\":{}", s.files));
            out.push_str(&format!(",\"bytes\":{}", s.bytes));
            out.push_str(&format!(",\"clean\":{}", s.clean));
            out.push_str(&format!(",\"monitor\":{}", s.monitor));
            out.push_str(&format!(",\"suspicious\":{}", s.suspicious));
            out.push_str(&format!(",\"high_risk\":{}", s.high_risk));
            out.push_str(&format!(",\"confirmed\":{}", s.confirmed));
            out.push_str(&format!(",\"unreadable\":{}", s.unreadable));
            out.push_str(&format!(",\"links_skipped\":{}", s.links_skipped));
            out.push_str(&format!(",\"read_errors\":{}", s.read_errors));
            out.push_str(&format!(",\"truncated\":{}", s.truncated));
            out.push_str(&format!(",\"elapsed_ms\":{}", s.elapsed_ms));
            out.push('}');
        }
        let (pass, fail, unmeasured) = self.tally();
        out.push_str(&format!(
            ",\"summary\":{{\"pass\":{pass},\"fail\":{fail},\"unmeasured\":{unmeasured}}}"
        ));
        out.push('}');
        out
    }
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

fn q(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_gate_has_a_key_a_unit_and_a_spec_line() {
        for gate in Gate::ALL {
            assert!(!gate.key().is_empty());
            assert!(!gate.unit().is_empty());
            assert!(gate.spec().contains("§34.2"), "{}", gate.spec());
        }
    }

    #[test]
    fn the_boundary_rule_matches_the_spec() {
        // "≤ 50" includes 50; "< 50" does not. Getting this backwards is the
        // difference between a pass and a fail at exactly one value.
        assert_eq!(
            Measurement::measured(Gate::RamUsbMb, 50.0).status,
            Status::Pass
        );
        assert_eq!(
            Measurement::measured(Gate::UsbWakeMs, 50.0).status,
            Status::Fail
        );
        assert_eq!(
            Measurement::measured(Gate::IdleIoOpsPerMin, 0.0).status,
            Status::Pass
        );
        assert_eq!(
            Measurement::measured(Gate::IdleIoOpsPerMin, 1.0).status,
            Status::Fail
        );
        assert_eq!(
            Measurement::measured(Gate::RamIdleMb, 20.5).status,
            Status::Fail
        );
        // D52 and D66 chose the inclusive form on purpose: an idle CPU sitting
        // exactly on 0.1% passes, and an AMSI hook costing exactly 15ms passes.
        // The old 0.5% allowance was five times the published policy line.
        assert_eq!(
            Measurement::measured(Gate::CpuIdlePercent, 0.1).status,
            Status::Pass
        );
        assert_eq!(
            Measurement::measured(Gate::CpuIdlePercent, 0.11).status,
            Status::Fail
        );
        assert_eq!(
            Measurement::measured(Gate::AmsiMs, 15.0).status,
            Status::Pass
        );
        assert_eq!(
            Measurement::measured(Gate::AmsiMs, 15.1).status,
            Status::Fail
        );
    }

    #[test]
    fn cpu_scan_gate_uses_one_core_not_whole_machine_capacity() {
        use std::time::{Duration, Instant};

        let t0 = Instant::now();
        let initial_cpu = Duration::from_secs(7);
        let before = Meter {
            at: Some(t0),
            cpu: Some(initial_cpu),
            ..Meter::default()
        };
        // Ten wall seconds, synthetic process CPU (user + kernel). A label
        // with eight logical processors must not divide the gate value by 8.
        for (cpu_ms, expected, status) in [
            (250, 2.5, Status::Pass),
            (300, 3.0, Status::Fail), // strictly below 3, not at most 3
            (2799, 27.99, Status::Fail),
            (20_000, 200.0, Status::Fail), // summed threads; do not clamp at 100
        ] {
            let after = Meter {
                at: Some(t0 + Duration::from_secs(10)),
                cpu: Some(initial_cpu + Duration::from_millis(cpu_ms)),
                ..Meter::default()
            };
            let mut r = PerfReport::new("2026-11-18", "0.1.0", "synthetic · 8 cores", "test");
            r.fill_from_scan(ScanStats::default(), &before, Some(&after));
            let m = r
                .metrics
                .iter()
                .find(|m| m.gate == Gate::CpuScanAvgPercent)
                .unwrap();
            assert_eq!(m.value, Some(expected));
            assert_eq!(m.status, status);
        }
    }

    #[test]
    fn throughput_has_no_threshold_and_is_not_a_failure() {
        assert!(Gate::ThroughputFilesPerSec.threshold().is_none());
        assert_eq!(
            Measurement::measured(Gate::ThroughputFilesPerSec, 0.4).status,
            Status::Pass
        );
    }

    #[test]
    fn an_unmeasured_gate_never_carries_a_value() {
        let m = Measurement::unmeasured(Gate::P0Ms, "needs C1");
        assert!(m.value.is_none());
        assert!(matches!(m.status, Status::Unmeasured(_)));
    }

    /// §34.2 lists thirteen metrics. A gate that exists in the enum but is
    /// never emitted is invisible — which is exactly what happened: the report
    /// from the first real run carried twelve gates and nobody noticed the
    /// thirteenth was missing. This asserts the *set*, not the count, so a gate
    /// cannot quietly replace another one.
    #[test]
    fn every_gate_in_the_spec_is_emitted() {
        let expected = [
            "ram_idle_mb",
            "ram_usb_mb",
            "cpu_idle_percent",
            "cpu_scan_avg_percent",
            "cpu_scan_peak_percent",
            "usb_wake_ms",
            "p0_ms",
            "p1_ms",
            "throughput_files_per_sec",
            "amsi_ms",
            "wdac_policy_ms",
            "language_switch_ms",
            "language_memory_mb",
            "idle_io_ops_per_min",
        ];
        let mut r = PerfReport::new("2026-11-16", "0.1.0", "cpu · 8 cores", "linux · x86_64");
        let stats = ScanStats {
            files: 10,
            elapsed_ms: 1000,
            clean: 10,
            ..ScanStats::default()
        };
        let idle = Meter::now();
        r.fill_from_scan(stats, &idle, None);

        let emitted: Vec<&str> = r.metrics.iter().map(|m| m.gate.key()).collect();
        for key in expected {
            assert!(
                emitted.contains(&key),
                "§34.2 lists {key} but the report never emits it; emitted: {emitted:?}"
            );
        }
        assert_eq!(
            emitted.len(),
            expected.len(),
            "the report emits {emitted:?}"
        );
    }

    #[test]
    fn the_default_report_marks_every_unbuilt_gate_as_unmeasured() {
        let mut r = PerfReport::new("2026-11-16", "0.1.0", "cpu · 8 cores", "linux · x86_64");
        let stats = ScanStats {
            files: 100,
            elapsed_ms: 1000,
            clean: 100,
            ..ScanStats::default()
        };
        let idle = Meter::now();
        r.fill_from_scan(stats, &idle, None);
        let (_, fail, unmeasured) = r.tally();
        assert_eq!(fail, 0);
        assert!(unmeasured >= 10, "got {unmeasured}");
        let json = r.to_json();
        assert!(json.contains("\"status\":\"unmeasured\""));
        assert!(!json.contains("\"value\":0"), "no invented zeros: {json}");
    }

    #[test]
    fn the_json_is_balanced_and_has_no_user_data() {
        let mut r = PerfReport::new("2026-11-16", "0.1.0", "cpu · 8 cores", "linux · x86_64");
        r.push(Measurement::measured(Gate::RamIdleMb, 12.5));
        r.push(Measurement::unmeasured(Gate::P0Ms, "needs C1"));
        let json = r.to_json();
        assert_eq!(json.matches('{').count(), json.matches('}').count());
        assert_eq!(json.matches('[').count(), json.matches(']').count());
        for forbidden in ["\"path\"", "\"user\"", "\"host\"", "\"hwid\"", "\"ip\""] {
            assert!(!json.contains(forbidden), "{forbidden} in {json}");
        }
    }

    #[test]
    fn scan_counters_must_account_for_every_file() {
        let ok = ScanStats {
            files: 5,
            clean: 3,
            monitor: 1,
            suspicious: 1,
            ..ScanStats::default()
        };
        assert!(ok.accounted());
        let lost = ScanStats {
            files: 5,
            clean: 3,
            ..ScanStats::default()
        };
        assert!(!lost.accounted());
    }
}
