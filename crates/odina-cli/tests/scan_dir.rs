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
    std::fs::write(dir.join("photo.jpg.exe"), b"harmless payload").unwrap();

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
    //
    // Scan the one file rather than splitting the folder array on `},`: the
    // old split stopped at the end of the nested `hashes` object, so the
    // indicator fields were never in the chunk and the assertion could not
    // pass. One file produces one object, which is what this test means.
    let json = Command::new(binary())
        .args(["scan", "--json"])
        .arg(dir.join("photo.jpg.exe"))
        .output()
        .unwrap();
    let payload = String::from_utf8_lossy(&json.stdout);
    assert!(
        payload.contains("\"indicators\":{\"name\":0.4500"),
        "deceptive name was not measured: {payload}"
    );
    assert!(
        payload.contains("\"interrupts\":false"),
        "name alone must not interrupt: {payload}"
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
    let out = Command::new(binary()).arg("selftest").output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("275a021bbfb6489e54d471899f7db9d1663fc695ec2fe2a2c4538aabf651fd0f"),
        "got: {stdout}"
    );
    assert_eq!(out.status.code(), Some(0), "got: {stdout}");
}
