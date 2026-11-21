// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K32 Unicode Bidi Override Guard Contract.

pub fn has_bidi_override(name: &str) -> bool {
    name.chars()
        .any(|c| matches!(c, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}'))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detects_rlo_in_filename() {
        assert!(has_bidi_override("invoice\u{202E}fdp.exe"));
        assert!(!has_bidi_override("فاتورة.pdf"));
    }
}
