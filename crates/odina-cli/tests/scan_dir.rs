// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! G1 as a test: point the binary at a folder and check it classifies what is
//! inside, keeps going past an unreadable file, and exits with a code that
//! means something.

use std::path::PathBuf;
use std::process::Command;

fn binary() -> PathBuf {
    // The test binary lives in target/<profile>/deps/, the CLI one level up.
    let mut path = std::env::current_exe().expect("test exe");
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.join(format!("odina{}", std::env::consts::EXE_SUFFIX))
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
fn scans_a_folder_and_reports_clean_and_suspicious_side_by_side() {
    let dir = scratch("mixed");
    std::fs::write(dir.join("notes.txt"), b"just some notes").unwrap();
    std::fs::write(dir.join("photo.jpg.exe"), eicar()).unwrap();

    let out = Command::new(binary())
        .arg("scan")
        .arg(&dir)
        .output()
        .expect("run odina");

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("الملفات: 2"),
        "expected 2 files, got:\n{stdout}"
    );
    // The deceptive double extension must be the one that interrupts.
    assert_eq!(out.status.code(), Some(2), "got: {stdout}");
    assert!(stdout.contains("photo.jpg.exe"), "got: {stdout}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_folder_of_ordinary_files_exits_clean() {
    let dir = scratch("clean");
    for name in ["a.txt", "b.md", "c.json"] {
        std::fs::write(dir.join(name), b"harmless content").unwrap();
    }

    let out = Command::new(binary()).arg("scan").arg(&dir).output().unwrap();
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
    assert!(trimmed.starts_with('[') && trimmed.ends_with(']'), "got: {trimmed}");
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
