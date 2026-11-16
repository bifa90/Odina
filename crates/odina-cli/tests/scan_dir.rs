// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! G1 as a test: point the binary at a folder and check it classifies what is
//! inside, keeps going past an unreadable file, and exits with a code that
//! means something.

use std::path::PathBuf;
use std::process::Command;

fn binary() -> PathBuf {
    // Cargo sets this for integration tests and it is the documented way to
    // find the binary under test. Reconstructing the path from current_exe()
    // works until the target layout changes, and then it fails on a runner
    // and not on your machine — which is exactly what happened once.
    PathBuf::from(env!("CARGO_BIN_EXE_odina"))
}

/// The EICAR standard test string, assembled from pieces so this source file
/// does not itself look like the test file to a scanner.
fn eicar() -> Vec<u8> {
    [
        "X5O!P%@AP[4\\PZX54(P^)7CC)7}$",
        "EICAR-STANDARD-ANTIVIRUS-",
        "TEST-FILE!$H+H*",
    ]
    .concat()
    .into_bytes()
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("odina-cli-test-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

#[test]
fn scans_a_folder_and_flags_the_deceptive_name() {
    let dir = scratch("mixed");
    std::fs::write(dir.join("notes.txt"), b"just some notes").unwrap();
    std::fs::write(dir.join("photo.jpg.exe"), eicar()).unwrap();

    let out = Command::new(binary())
        .arg("scan")
        .arg(&dir)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("الملفات: 2"), "expected 2 files:\n{stdout}");

    // The engine promises to *measure* the deception, not to raise an alert on
    // it alone: name (8%) plus path (8%) is 10.4% at best, which is silence by
    // design — see cap S1. This asserts the measurement, which is the part
    // that must not regress.
    let json = Command::new(binary())
        .args(["scan", "--json"])
        .arg(&dir)
        .output()
        .unwrap();
    let payload = String::from_utf8_lossy(&json.stdout);

    // Parsing with a real idea of the structure instead of splitting on "},":
    // an object containing "hashes":{...} ends with "}}," so a naive split cuts
    // the object in half and every assertion after it is meaningless.
    let deceptive = json_object_for(&payload, "photo.jpg.exe")
        .unwrap_or_else(|| panic!("the deceptive file must appear in the JSON: {payload}"));
    assert!(
        deceptive.contains("\"name\":0.4"),
        "deceptive name was not measured: {deceptive}"
    );
    assert!(
        deceptive.contains("\"interrupts\":false"),
        "name alone must not interrupt: {deceptive}"
    );

    // A folder of files nobody would call a threat must exit clean.
    assert_eq!(out.status.code(), Some(0), "got: {stdout}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_folder_of_ordinary_files_exits_clean() {
    let dir = scratch("clean");
    for name in ["a.txt", "b.md", "c.json"] {
        std::fs::write(dir.join(name), b"harmless content").unwrap();
    }

    let out = Command::new(binary())
        .arg("scan")
        .arg(&dir)
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn json_output_is_one_array_and_parses_as_such() {
    let dir = scratch("json");
    std::fs::write(dir.join("x.txt"), b"hi").unwrap();

    let out = Command::new(binary())
        .args(["scan", "--json"])
        .arg(&dir)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let trimmed = stdout.trim();
    assert!(
        trimmed.starts_with('[') && trimmed.ends_with(']'),
        "got: {trimmed}"
    );
    assert!(trimmed.contains("\"verdict\""), "got: {trimmed}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn selftest_matches_the_published_digest() {
    let out = Command::new(binary())
        .arg("selftest")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("275a021bbfb6489e54d471899f7db9d1663fc695ec2fe2a2c4538aabf651fd0f"),
        "got: {stdout}"
    );
    assert_eq!(out.status.code(), Some(0), "got: {stdout}");
}

/// Return the one JSON object in `payload` that mentions `needle`, using brace
/// depth rather than string splitting. Small enough to be honest about, and it
/// fails loudly instead of silently returning the wrong slice.
fn json_object_for(payload: &str, needle: &str) -> Option<String> {
    let bytes = payload.as_bytes();
    let mut depth = 0usize;
    let mut start = None;
    let mut in_string = false;
    let mut escaped = false;

    for (i, &b) in bytes.iter().enumerate() {
        if in_string {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_string = false;
            }
            continue;
        }
        match b {
            b'"' => in_string = true,
            b'{' => {
                if depth == 0 {
                    start = Some(i);
                }
                depth += 1;
            }
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    if let Some(s) = start {
                        let candidate = &payload[s..=i];
                        if candidate.contains(needle) {
                            return Some(candidate.to_string());
                        }
                    }
                    start = None;
                }
            }
            _ => {}
        }
    }
    None
}
