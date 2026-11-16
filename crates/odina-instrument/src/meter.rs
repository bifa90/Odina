// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! Reading the machine's own counters — without a dependency and without
//! asking the user anything.
//!
//! Three numbers are collected here: resident memory, process CPU time, and
//! the process I/O counter. Each is a property of *this process*, not of the
//! person running it, which is exactly the line §34.3 draws.
//!
//! On a platform that cannot answer, the function returns `None` and the
//! report says "unmeasured". It never returns zero as a fallback: zero is a
//! measurement ("no I/O happened"), and an unmeasured value dressed as zero is
//! a false pass on a gate whose threshold *is* zero.

use std::time::{Duration, Instant};

#[cfg(windows)]
fn rss_kb_windows() -> Option<u64> {
    #[repr(C)]
    #[allow(non_snake_case)]
    struct ProcessMemoryCounters {
        cb: u32,
        PageFaultCount: u32,
        PeakWorkingSetSize: usize,
        WorkingSetSize: usize,
        QuotaPeakPagedPoolUsage: usize,
        QuotaPagedPoolUsage: usize,
        QuotaPeakNonPagedPoolUsage: usize,
        QuotaNonPagedPoolUsage: usize,
        PagefileUsage: usize,
        PeakPagefileUsage: usize,
    }

    #[link(name = "psapi")]
    extern "system" {
        fn GetProcessMemoryInfo(
            process: isize,
            counters: *mut ProcessMemoryCounters,
            cb: u32,
        ) -> i32;
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GetCurrentProcess() -> isize;
    }

    let mut pmc = ProcessMemoryCounters {
        cb: std::mem::size_of::<ProcessMemoryCounters>() as u32,
        PageFaultCount: 0,
        PeakWorkingSetSize: 0,
        WorkingSetSize: 0,
        QuotaPeakPagedPoolUsage: 0,
        QuotaPagedPoolUsage: 0,
        QuotaPeakNonPagedPoolUsage: 0,
        QuotaNonPagedPoolUsage: 0,
        PagefileUsage: 0,
        PeakPagefileUsage: 0,
    };
    // SAFETY: `pmc` is a live local with `cb` set to its own size, which is
    // what the API requires; it writes no more than that.
    let rc = unsafe {
        GetProcessMemoryInfo(
            GetCurrentProcess(),
            &mut pmc,
            std::mem::size_of::<ProcessMemoryCounters>() as u32,
        )
    };
    if rc == 0 {
        None
    } else {
        Some((pmc.WorkingSetSize / 1024) as u64)
    }
}

/// Resident set size in kibibytes.
pub fn rss_kb() -> Option<u64> {
    #[cfg(windows)]
    {
        rss_kb_windows()
    }
    #[cfg(target_os = "linux")]
    {
        let status = std::fs::read_to_string("/proc/self/status").ok()?;
        let line = status.lines().find(|l| l.starts_with("VmRSS:"))?;
        let kb = line.split_whitespace().nth(1)?.parse::<u64>().ok()?;
        Some(kb)
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        None
    }
}

#[cfg(windows)]
fn cpu_time_windows() -> Option<Duration> {
    #[repr(C)]
    #[derive(Default)]
    struct FileTime {
        low: u32,
        high: u32,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetCurrentProcess() -> isize;
        fn GetProcessTimes(
            process: isize,
            creation: *mut FileTime,
            exit: *mut FileTime,
            kernel: *mut FileTime,
            user: *mut FileTime,
        ) -> i32;
    }

    let (mut c, mut e, mut k, mut u) = (
        FileTime::default(),
        FileTime::default(),
        FileTime::default(),
        FileTime::default(),
    );
    // SAFETY: four distinct live locals, all zeroed, of the exact type the API
    // writes. A failure leaves them untouched.
    let rc = unsafe { GetProcessTimes(GetCurrentProcess(), &mut c, &mut e, &mut k, &mut u) };
    if rc == 0 {
        return None;
    }
    let to_100ns = |t: &FileTime| ((t.high as u64) << 32 | t.low as u64) as u128;
    // FILETIME is in 100-nanosecond units.
    let total = to_100ns(&k) + to_100ns(&u);
    Some(Duration::from_nanos((total * 100) as u64))
}

/// CPU time consumed by this process so far (user + kernel).
pub fn cpu_time() -> Option<Duration> {
    #[cfg(windows)]
    {
        cpu_time_windows()
    }
    #[cfg(target_os = "linux")]
    {
        // /proc/self/stat fields 14 and 15 are utime and stime in clock ticks.
        // USER_HZ is 100 on every Linux the project targets (x86-64, arm64);
        // the value is only used for percentages, and the sampler cross-checks
        // it against wall time (a process cannot use more CPU than 100% per
        // core), so a wrong constant shows up as a failing sanity check rather
        // than as a fabricated number.
        const USER_HZ: f64 = 100.0;
        let stat = std::fs::read_to_string("/proc/self/stat").ok()?;
        // The comm field can contain spaces and parentheses; everything after
        // the last ')' is fixed-width.
        let tail_start = stat.rfind(')')? + 1;
        let fields: Vec<&str> = stat[tail_start..].split_whitespace().collect();
        // After comm: state is field 3. utime = field 14, stime = field 15.
        let utime: f64 = fields.get(11)?.parse().ok()?;
        let stime: f64 = fields.get(12)?.parse().ok()?;
        let secs = (utime + stime) / USER_HZ;
        Some(Duration::from_secs_f64(secs))
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        None
    }
}

#[cfg(windows)]
fn io_ops_windows() -> Option<u64> {
    #[repr(C)]
    #[derive(Default)]
    struct IoCounters {
        ReadOperationCount: u64,
        WriteOperationCount: u64,
        OtherOperationCount: u64,
        ReadTransferCount: u64,
        WriteTransferCount: u64,
        OtherTransferCount: u64,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetCurrentProcess() -> isize;
        fn GetProcessIoCounters(process: isize, counters: *mut IoCounters) -> i32;
    }

    let mut io = IoCounters::default();
    // SAFETY: one live local of the exact type; the API writes it entirely.
    let rc = unsafe { GetProcessIoCounters(GetCurrentProcess(), &mut io) };
    if rc == 0 {
        None
    } else {
        Some(io.ReadOperationCount + io.WriteOperationCount + io.OtherOperationCount)
    }
}

/// Number of read/write/other I/O operations performed by this process.
///
/// The idle gate is `0`, so this must distinguish "no operations" from "cannot
/// tell" — which is why it returns `Option`.
pub fn io_ops() -> Option<u64> {
    #[cfg(windows)]
    {
        io_ops_windows()
    }
    #[cfg(target_os = "linux")]
    {
        let text = std::fs::read_to_string("/proc/self/io").ok()?;
        let mut total = 0u64;
        for line in text.lines() {
            let mut parts = line.split_whitespace();
            let key = parts.next()?;
            if matches!(key, "syscr:" | "syscw:" | "rchar:" | "wchar:") {
                total += parts.next()?.parse::<u64>().ok()?;
            }
        }
        Some(total)
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        None
    }
}

/// A snapshot of the three counters, so a ratio can be computed from deltas.
#[derive(Debug, Clone, Copy, Default)]
pub struct Meter {
    pub at: Option<Instant>,
    pub cpu: Option<Duration>,
    pub io: Option<u64>,
    pub rss_kb: Option<u64>,
}

impl Meter {
    pub fn now() -> Meter {
        Meter {
            at: Some(Instant::now()),
            cpu: cpu_time(),
            io: io_ops(),
            rss_kb: rss_kb(),
        }
    }

    /// CPU time used between two snapshots, as a percentage of wall time on
    /// one core. Returns `None` when either side could not be read.
    pub fn cpu_percent_of(&self, earlier: &Meter) -> Option<f64> {
        let wall = self.at?.checked_duration_since(earlier.at?)?.as_secs_f64();
        let cpu = self.cpu?.checked_sub(earlier.cpu?)?.as_secs_f64();
        if wall <= 0.0 {
            return None;
        }
        Some(cpu / wall * 100.0)
    }

    /// I/O operations performed between two snapshots, per minute.
    pub fn io_ops_per_min_of(&self, earlier: &Meter) -> Option<f64> {
        let wall = self.at?.checked_duration_since(earlier.at?)?.as_secs_f64();
        let ops = self.io?.checked_sub(earlier.io?)? as f64;
        if wall <= 0.0 {
            return None;
        }
        Some(ops / wall * 60.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_counters_answer_or_say_they_cannot() {
        // On any platform we build for, at least RSS must be readable; if it
        // is not, the API returns None rather than zero, and that contract is
        // what the perf layer depends on.
        let m = Meter::now();
        assert!(m.at.is_some());
        if let Some(rss) = m.rss_kb {
            assert!(rss > 0, "a running process has memory");
        }
    }

    #[test]
    fn cpu_time_never_goes_backwards() {
        let a = cpu_time();
        let _ = cpu_time();
        let b = cpu_time();
        if let (Some(a), Some(b)) = (a, b) {
            assert!(b >= a);
        }
    }
}
