// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! Hardware facts for the instrumentation layer — and the exact limits of
//! what may be recorded.
//!
//! `docs/odina-agent-prompt-v17.md` §35.3 says a performance number without
//! the hardware it was measured on means nothing, so the CPU name and core
//! count must travel with every measurement. §34.3 says the same layer must
//! never measure anything about the user.
//!
//! Those two rules collide at exactly one point: **the CPU model**, which is
//! shared by millions of machines and identifies none of them. These tests pin
//! that boundary: a model string with a serial number, a UUID or an asset tag
//! in it is rejected rather than shipped.
//!
//! On a platform where the model cannot be read, `None` is returned. The
//! report says "unknown", never "generic".

/// What §35.3 requires: a CPU label and a core count. Nothing else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hardware {
    /// CPU model, e.g. `"Intel(R) Core(TM) i5-8250U CPU @ 1.60GHz"`.
    pub cpu_model: Option<String>,
    pub logical_cores: u32,
}

impl Hardware {
    /// Read what this platform will tell us, without a dependency.
    pub fn detect() -> Hardware {
        Hardware {
            cpu_model: platform_cpu_model(),
            logical_cores: std::thread::available_parallelism()
                .map(|n| n.get() as u32)
                .unwrap_or(1),
        }
    }

    /// Is this a CPU model string and nothing more? The check that keeps a
    /// serial number out of a "hardware" field.
    pub fn cpu_model_is_safe(model: &str) -> bool {
        let lower = model.to_ascii_lowercase();
        const FORBIDDEN: [&str; 4] = ["serial", "uuid", "asset tag", "oem id"];
        if FORBIDDEN.iter().any(|f| lower.contains(f)) {
            return false;
        }
        // A long run of hex digits with dashes is a UUID shape, whatever it is
        // called. `i5-8250U` and `i7-1185G7` are short; a UUID is 36 chars.
        let hexish = model
            .chars()
            .filter(|c| c.is_ascii_hexdigit() || *c == '-')
            .count();
        hexish * 100 / model.chars().count().max(1) < 80 || model.chars().count() < 20
    }

    /// One line for a report header, in the shape the perf protocol expects.
    pub fn label(&self) -> String {
        match &self.cpu_model {
            Some(m) => format!("{m} · {} logical cores", self.logical_cores),
            None => format!("unknown CPU · {} logical cores", self.logical_cores),
        }
    }
}

#[cfg(windows)]
fn platform_cpu_model() -> Option<String> {
    // Read the registry directly instead of spawning `wmic`: a measurement run
    // that shells out is measuring the shell as well.
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "advapi32")]
    extern "system" {
        fn RegGetValueW(
            hkey: isize,
            sub_key: *const u16,
            value: *const u16,
            flags: u32,
            out_type: *mut u32,
            data: *mut u8,
            data_size: *mut u32,
        ) -> i32;
    }

    const HKEY_LOCAL_MACHINE: isize = 0x80000002u32 as isize;
    const RRF_RT_REG_SZ: u32 = 0x0000_0002;

    fn wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(Some(0)).collect()
    }

    let sub = wide(r"HARDWARE\DESCRIPTION\System\CentralProcessor\0");
    let value = wide("ProcessorNameString");
    let mut buf = [0u16; 256];
    let mut size = (buf.len() * 2) as u32;

    // SAFETY: all pointers come from stack arrays of the stated sizes; the API
    // writes at most `size` bytes and reports the length it wrote.
    let rc = unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            sub.as_ptr(),
            value.as_ptr(),
            RRF_RT_REG_SZ,
            std::ptr::null_mut(),
            buf.as_mut_ptr() as *mut u8,
            &mut size,
        )
    };
    if rc != 0 {
        return None;
    }
    let end = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
    let text = String::from_utf16_lossy(&buf[..end]).trim().to_string();
    if text.is_empty() || !Hardware::cpu_model_is_safe(&text) {
        None
    } else {
        Some(text)
    }
}

#[cfg(target_os = "linux")]
fn platform_cpu_model() -> Option<String> {
    // x86 and most ARM boards expose a model line in /proc/cpuinfo.
    if let Some(model) = std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .as_deref()
        .and_then(cpu_model_from_cpuinfo)
    {
        return Some(model);
    }
    // Some arm64 kernels do not, but they do expose the MIDR register. A
    // register value is a model, not a board: the same value appears on every
    // device with that core.
    cpu_model_from_sysfs()
}

#[cfg(not(any(windows, target_os = "linux")))]
fn platform_cpu_model() -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_real_cpu_label_is_accepted() {
        assert!(Hardware::cpu_model_is_safe(
            "Intel(R) Core(TM) i5-8250U CPU @ 1.60GHz"
        ));
        assert!(Hardware::cpu_model_is_safe(
            "AMD Ryzen 5 3600 6-Core Processor"
        ));
        assert!(Hardware::cpu_model_is_safe("Apple M1"));
    }

    #[test]
    fn a_serial_number_is_not_a_cpu_model() {
        assert!(!Hardware::cpu_model_is_safe("Intel i5, Serial 8C4F2A1B"));
        assert!(!Hardware::cpu_model_is_safe(
            "CPU UUID 550e8400-e29b-41d4-a716-446655440000"
        ));
        assert!(!Hardware::cpu_model_is_safe("Box Asset Tag 12345"));
    }

    #[test]
    fn detection_never_panics_and_reports_at_least_one_core() {
        let hw = Hardware::detect();
        assert!(hw.logical_cores >= 1);
        assert!(!hw.label().is_empty());
    }
}

#[cfg(target_os = "linux")]
fn cpu_model_from_cpuinfo(text: &str) -> Option<String> {
    let line = text
        .lines()
        .find(|l| l.starts_with("model name") || l.starts_with("Model"))?;
    let model = line.split(':').nth(1)?.trim().to_string();
    if model.is_empty() || !Hardware::cpu_model_is_safe(&model) {
        None
    } else {
        Some(model)
    }
}

#[cfg(target_os = "linux")]
fn cpu_model_from_sysfs() -> Option<String> {
    let bytes = std::fs::read("/sys/devices/system/cpu/cpu0/regs/identification/midr_el1").ok()?;
    if bytes.len() < 4 {
        return None;
    }
    let midr = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    Some(format!("arm64 midr {midr:#010x}"))
}

// The module itself is gated, not just the tests inside it: on Windows every
// test below disappears, and a `use super::*` with nothing to import is an
// unused-import error there.
#[cfg(all(test, target_os = "linux"))]
mod linux_tests {
    use super::*;

    /// Parsing is tested against a fixture, because the machine running the
    /// test may be neither x86 nor Linux.
    #[test]
    #[cfg(target_os = "linux")]
    fn a_cpuinfo_line_becomes_a_model_string() {
        let fixture = "processor\t: 0\nmodel name\t: Intel(R) Core(TM) i7-10510U CPU @ 1.80GHz\n";
        assert_eq!(
            cpu_model_from_cpuinfo(fixture).as_deref(),
            Some("Intel(R) Core(TM) i7-10510U CPU @ 1.80GHz")
        );
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn a_cpuinfo_with_a_serial_is_refused() {
        let fixture = "model name\t: Intel i5 Serial 8C4F2A1B\n";
        assert!(cpu_model_from_cpuinfo(fixture).is_none());
    }
}
