// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K40 Temp & Downloads High-Scrutiny Scope Contract.

pub fn is_high_scrutiny_folder(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.contains("\\downloads\\")
        || lower.contains("/downloads/")
        || lower.contains("\\temp\\")
        || lower.contains("/tmp/")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn classifies_downloads_and_temp() {
        assert!(is_high_scrutiny_folder("C:\\Users\\u\\Downloads\\a.exe"));
        assert!(!is_high_scrutiny_folder("C:\\Windows\\System32\\a.dll"));
    }
}
