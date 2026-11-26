// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! Every message key the engine can hand to a screen, in one list.
//!
//! A screen does not carry text; it carries a key that the locale files
//! resolve. The failure mode of that design is silent: a key that no `.ftl`
//! file defines renders as an empty label, and an empty label beside a shield
//! reads like "nothing to worry about". Unit tests could not catch it because
//! they compared a key with itself.
//!
//! So the keys live here as constants, `UI_KEYS` lists all of them, and a test
//! opens **all ten** locale files and fails if one key is missing anywhere.
//! Adding a message is therefore two edits in the same commit: the constant,
//! and the ten translations.

/// The state line: what the guard is doing right now.
pub const STATE_PROTECTED: &str = "state.protected";
pub const STATE_SCANNING: &str = "state.scanning";
pub const STATE_ATTENTION: &str = "state.attention";
pub const STATE_PAUSED: &str = "state.paused";
pub const STATE_UNTRUSTED: &str = "state.untrusted";
pub const STATE_UNPROTECTED: &str = "state.unprotected";
pub const STATE_STALE: &str = "state.stale";

/// The update chip and the version line under the shield (D122, D113).
pub const UPDATE_CHIP: &str = "update.chip";
pub const HOME_VERSION: &str = "home.version";

/// Every key this crate may emit. The order is the order a reviewer reads them.
pub const UI_KEYS: &[&str] = &[
    STATE_PROTECTED,
    STATE_SCANNING,
    STATE_ATTENTION,
    STATE_PAUSED,
    STATE_UNTRUSTED,
    STATE_UNPROTECTED,
    STATE_STALE,
    UPDATE_CHIP,
    HOME_VERSION,
];

/// Whether a locale text defines `key`. Public because the shell and the CLI
/// will ask the same question at runtime; the pattern is deliberately narrower
/// than Fluent, because a pattern that also accepted attributes or terms would
/// also accept a typo.
pub fn defines(source: &str, key: &str) -> bool {
    source.lines().any(|line| {
        let Some((name, _)) = line.split_once(" = ") else {
            return false;
        };
        name.trim() == key
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOCALES: &[(&str, &str)] = &[
        ("ar", include_str!("../../../locales/ar/odina.ftl")),
        ("de", include_str!("../../../locales/de/odina.ftl")),
        ("en", include_str!("../../../locales/en/odina.ftl")),
        ("es", include_str!("../../../locales/es/odina.ftl")),
        ("fr", include_str!("../../../locales/fr/odina.ftl")),
        ("hi", include_str!("../../../locales/hi/odina.ftl")),
        ("ja", include_str!("../../../locales/ja/odina.ftl")),
        ("pt-BR", include_str!("../../../locales/pt-BR/odina.ftl")),
        ("ru", include_str!("../../../locales/ru/odina.ftl")),
        (
            "zh-Hans",
            include_str!("../../../locales/zh-Hans/odina.ftl"),
        ),
    ];

    #[test]
    fn every_key_the_engine_emits_exists_in_all_ten_locales() {
        for &(lang, source) in LOCALES {
            for &key in UI_KEYS {
                assert!(defines(source, key), "{lang} has no {key}");
            }
        }
    }

    #[test]
    fn the_reader_sees_a_key_it_knows_and_rejects_a_near_miss() {
        // A guard on the guard: were `defines` too permissive, the test above
        // would pass for a file it cannot actually read.
        assert!(defines("state.protected = x\n", STATE_PROTECTED));
        assert!(!defines("state.protected = x\n", STATE_PAUSED));
        assert!(!defines("## state.protected = x\n", STATE_PROTECTED));
        assert!(!defines("state.protectedX = x\n", STATE_PROTECTED));
    }

    #[test]
    fn the_list_has_no_duplicate_and_every_key_is_dotted_ascii() {
        const ALLOWED: &str = "abcdefghijklmnopqrstuvwxyz0123456789._";
        let mut seen = UI_KEYS.to_vec();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), UI_KEYS.len(), "a key is listed twice");
        for &key in UI_KEYS {
            assert!(key.contains('.'), "{key} is not dotted");
            assert!(key.chars().all(|c| ALLOWED.contains(c)), "{key} bad char");
        }
    }
}
