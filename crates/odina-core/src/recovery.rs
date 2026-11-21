// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K13 Safe-Mode & Recovery Contract.
//!
//! Enforces that corrupted config or cache never silently disables protection;
//! instead the engine falls back to `SafeMinimal` with protection active.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryMode {
    Normal,
    SafeMinimal,
    EmergencyReadOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BootHealth {
    pub config_valid: bool,
    pub cache_valid: bool,
    pub protection_enabled: bool,
    pub mode: RecoveryMode,
}

pub fn resolve_boot_mode(config_valid: bool, cache_valid: bool) -> BootHealth {
    match (config_valid, cache_valid) {
        (true, true) => BootHealth {
            config_valid: true,
            cache_valid: true,
            protection_enabled: true,
            mode: RecoveryMode::Normal,
        },
        (false, _) => BootHealth {
            config_valid: false,
            cache_valid,
            protection_enabled: true,
            mode: RecoveryMode::SafeMinimal,
        },
        (true, false) => BootHealth {
            config_valid: true,
            cache_valid: false,
            protection_enabled: true,
            mode: RecoveryMode::EmergencyReadOnly,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corrupt_config_or_cache_never_disables_protection() {
        let bad_cfg = resolve_boot_mode(false, true);
        assert!(bad_cfg.protection_enabled);
        assert_eq!(bad_cfg.mode, RecoveryMode::SafeMinimal);

        let bad_cache = resolve_boot_mode(true, false);
        assert!(bad_cache.protection_enabled);
        assert_eq!(bad_cache.mode, RecoveryMode::EmergencyReadOnly);

        let ok = resolve_boot_mode(true, true);
        assert!(ok.protection_enabled);
        assert_eq!(ok.mode, RecoveryMode::Normal);
    }
}
