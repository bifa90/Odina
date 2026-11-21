// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K33 Double-Extension Deception Contract.

pub fn is_deceptive_double_ext(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    (lower.ends_with(".exe") || lower.ends_with(".scr") || lower.ends_with(".bat"))
        && (lower.contains(".pdf.") || lower.contains(".docx.") || lower.contains(".jpg."))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn flags_pdf_exe() {
        assert!(is_deceptive_double_ext("report.pdf.exe"));
        assert!(!is_deceptive_double_ext("report.pdf"));
    }
}
