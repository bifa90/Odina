// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! What the **public** build can do, asserted as a test.
//!
//! Before K1 this file could not exist: the resident service was compiled only
//! with `--features instrument`, so "the service works for a user who installed
//! the product" had no witness. It has one now, and it is the point of K1 —
//! the service is a core capability (v19 §1, D49), and the measurement layer is
//! the only thing the flag adds.
//!
//! `tests/resident.rs` keeps its instrumented parity test: that one pins the
//! lab build's report against its own scan. This one pins the public build's
//! scan against nothing but the verdicts the user would see.

#![cfg(not(feature = "instrument"))]
mod common;

use std::{fs, path::PathBuf, process::Command};

const BIN: &str = env!("CARGO_BIN_EXE_odina");

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("odina-public-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

/// `odina check` is the one command the flag adds. In the public build it must
/// refuse, and it must say why — a bare "unknown command" would read as a bug
/// in a command the documentation mentions.
#[test]
fn check_is_absent_from_the_public_build_and_says_so() {
    let dir = scratch("nocheck");
    let file = dir.join("a.txt");
    fs::write(&file, b"nothing interesting").unwrap();

    let out = Command::new(BIN).arg("check").arg(&file).output().unwrap();
    assert_eq!(out.status.code(), Some(1), "check must refuse, not scan");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        text.contains("public build") && text.contains("instrument"),
        "the refusal must name the build and the flag: {text}"
    );
    assert!(
        !dir.join("odina-check.json").exists(),
        "a refused check must not leave a report behind"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// The service, in the build a user installs: start it, ask it to scan, read a
/// verdict out of it, stop it. No `check`, no report, no cache flag required —
/// only what the product ships.
#[cfg(any(target_os = "linux", windows))]
mod resident {
    use super::*;
    use std::{
        io::{BufRead, BufReader},
        process::{Child, Stdio},
        sync::mpsc,
        thread,
        time::{Duration, Instant},
    };

    fn run(c: &mut Command, code: i32) -> String {
        let out = c.output().unwrap();
        assert_eq!(
            out.status.code(),
            Some(code),
            "{} {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    }

    struct Instance(Child, PathBuf);
    impl Drop for Instance {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
            let _ = fs::remove_dir_all(&self.1);
        }
    }

    fn wait_for(mut query: impl FnMut() -> String, expected: &str) -> String {
        let by = Instant::now() + Duration::from_secs(20);
        loop {
            let text = query();
            if text.contains(expected) {
                return text;
            }
            assert!(Instant::now() < by, "wanted {expected}: {text}");
            thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn the_public_service_scans_and_reports_the_embedded_vector() {
        let dir = scratch("service");
        let root = dir.join("corpus");
        fs::create_dir(&root).unwrap();
        // One harmless file and the standard non-malicious test vector. The
        // vector is the only known-bad digest the engine ships, so finding it
        // proves the public build runs the real scorer, not a stub.
        fs::write(root.join("notes.md"), b"harmless notes").unwrap();
        fs::write(root.join("vector.txt"), common::offline_vector()).unwrap();
        let name = format!("public-{}", std::process::id());

        let mut serve = Command::new(BIN);
        serve
            .arg("serve")
            .arg(&root)
            .args(["--service", &name, "--cache-dir"])
            .arg(dir.join("index"))
            .stdout(Stdio::piped());
        let mut guard = Instance(serve.spawn().unwrap(), dir.clone());
        let stdout = guard.0.stdout.take().unwrap();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            let mut first = String::new();
            let _ = reader.read_line(&mut first);
            let _ = tx.send(first);
            let _ = std::io::copy(&mut reader, &mut std::io::sink());
        });
        assert!(
            rx.recv_timeout(Duration::from_secs(20))
                .unwrap()
                .starts_with("READY"),
            "the service never announced itself"
        );

        let control = |op: &str| {
            let mut c = Command::new(BIN);
            c.args([op, "--service", &name]);
            c
        };
        let verdict = |rel: &str| {
            run(
                Command::new(BIN).args(["verdict", rel, "--service", &name]),
                0,
            )
        };

        // An explicit full scan through the public loop. `scan-now` answers
        // "accepted" immediately; the pass itself is what `completed` and
        // `last_exit` record, and the exit code comes from `Totals::exit_code`
        // in odina-scan — the same function the public CLI would call.
        assert!(
            run(&mut control("status"), 0).contains(r#""last_exit":null"#),
            "nothing has been scanned yet, so nothing may claim an exit code"
        );
        let accepted = run(&mut control("scan-now"), 0);
        assert!(accepted.contains("accepted"), "got: {accepted}");
        let status = wait_for(|| run(&mut control("status"), 0), r#""completed":1"#);
        assert!(
            status.contains(r#""last_exit":2"#),
            "the found vector must raise the code to 2: {status}"
        );

        // The verdict the user would see, through the resident, in the build
        // that ships without the measurement layer. `lookup` answers "unknown"
        // until the watcher has subscribed and reconciled — that is the
        // contract, so this waits for the verdict rather than for a timer.
        let found = wait_for(|| verdict("vector.txt"), r#""verdict":"confirmed""#);
        assert!(
            found.contains(r#""origin":"inventory""#),
            "the confirmation must come from the scan, not from an event: {found}"
        );
        let clean = wait_for(|| verdict("notes.md"), r#""verdict":"clean""#);
        assert!(
            !clean.contains(r#""state":"unknown""#),
            "an ordinary file must reach a verdict, not stay unknown: {clean}"
        );

        run(&mut control("stop"), 0);
        assert!(guard.0.wait().unwrap().success(), "the service must exit 0");

        // The index the public path seals is the same file format the lab path
        // writes, so the resident's warm start survives K1. Its *contents* are
        // hints and are not asserted here: publishing compares this pass with
        // the previous one, and a single pass over a fresh index legitimately
        // writes nothing. What must exist is the directory the reader opened.
        assert!(
            dir.join("index").is_dir(),
            "the public service must keep the cache directory it was given"
        );
    }
}
