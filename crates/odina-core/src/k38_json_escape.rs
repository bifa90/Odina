// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K38 JSON Escaping Contract.

pub fn escape_json_str(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn escapes_windows_paths_and_quotes() {
        assert_eq!(escape_json_str("C:\\a\\\"b\""), "C:\\\\a\\\\\\\"b\\\"");
    }
}
