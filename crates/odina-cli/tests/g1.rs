// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! Black-box B6 regression, without a network dependency in CI. To repeat on
//! the official download, set ODINA_G1_EICAR to that file outside the checkout.

use odina_core::g1_reference::{EICAR_SHA256, VERSION};
use odina_core::hashing::hash_reader;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

struct Scratch(PathBuf);

impl Scratch {
    fn new(case: &str) -> Self {
        let path = std::env::temp_dir().join(format!("odina-g1-{case}-{}", std::process::id()));
        #[cfg(unix)]
        let created = {
            use std::os::unix::fs::DirBuilderExt;
            fs::DirBuilder::new().mode(0o700).create(&path)
        };
        #[cfg(not(unix))]
        let created = fs::create_dir(&path);
        created.expect("create a fresh test directory");
        Scratch(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn odina(command: &str, path: &Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_odina"));
    cmd.arg(command).arg(path);
    cmd
}

fn run(cmd: &mut Command, expected_exit: i32) -> String {
    let result = cmd.output().expect("run the CLI binary");
    let stdout = String::from_utf8(result.stdout).unwrap();
    assert_eq!(
        result.status.code(),
        Some(expected_exit),
        "{stdout}\n{}",
        String::from_utf8_lossy(&result.stderr)
    );
    stdout
}

#[cfg(feature = "instrument")]
fn check(path: &Path, dir: &Scratch, expected_exit: i32) -> String {
    let out = dir.0.join("report.json"); // never inside the safe corpus
    run(odina("check", path).arg("--out").arg(&out), expected_exit);
    fs::read_to_string(out).expect("check must write the local report")
}

#[test]
fn exact_eicar_file_reaches_v5() {
    let dir = Scratch::new("positive");
    let path = match std::env::var_os("ODINA_G1_EICAR") {
        Some(supplied) => PathBuf::from(supplied), // never fall back on a bad input
        None => {
            // The same canonical vector as the in-memory selftest, but here the
            // CLI must OPEN and classify a real file, with an unremarkable name.
            // This offline case tests the bytes, not their download provenance.
            let path = dir.0.join("note.txt");
            let bytes = [
                "X5O!P%@AP[4\\PZX54(P^)7CC)7}$",
                "EICAR-STANDARD-ANTIVIRUS-",
                "TEST-FILE!$H+H*",
            ]
            .concat();
            fs::write(&path, bytes).unwrap();
            path
        }
    };
    let file = fs::File::open(&path).expect("read the EICAR test file, never execute it");
    assert_eq!(file.metadata().unwrap().len(), 68);
    assert_eq!(hash_reader(file, Some(68)).unwrap().full, EICAR_SHA256);

    let scan = run(odina("scan", &path).arg("--json"), 2);
    for field in [
        r#""known_bad":true"#,
        r#""veto":"KnownBad""#,
        r#""verdict":"confirmed""#,
        r#""verdict_ar":"مؤكد""#,
        r#""interrupts":true"#,
    ] {
        assert!(scan.contains(field), "missing {field}: {scan}");
    }
    assert!(scan.contains(&format!(r#""full":"{EICAR_SHA256}""#)));
    assert!(scan.contains(&format!(r#""db_version":"{VERSION}""#)));

    #[cfg(feature = "instrument")]
    {
        let json = check(&path, &dir, 2);
        for field in [
            r#""files":1,"#,
            r#""bytes":68,"#,
            r#""confirmed":1,"#,
            r#""unreadable":0,"#,
        ] {
            assert!(json.contains(field), "missing {field}: {json}");
        }
        assert!(json.contains(r#""reports":[{"#));
        assert_eq!(json.matches(r#""schema":"odina-report/v1""#).count(), 1);
        assert!(json.contains(&format!(r#""sha256":"{EICAR_SHA256}""#)));
        assert!(json.contains(&format!(r#""db_version":"{VERSION}""#)));
        assert!(!json.contains(r#""path":"#) && !json.contains(r#""name":"#));
    }
}

#[test]
fn twenty_known_safe_files_are_all_read_without_alerts() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/safe");
    let scan = run(odina("scan", &path).arg("--json"), 0);
    let mut digests = std::collections::BTreeSet::new();
    for entry in fs::read_dir(&path).unwrap() {
        let entry = entry.unwrap();
        assert!(entry.file_type().unwrap().is_file());
        let hash = hash_reader(fs::File::open(entry.path()).unwrap(), None)
            .unwrap()
            .full;
        assert_ne!(hash, EICAR_SHA256);
        assert_eq!(
            scan.matches(&format!(r#""full":"{hash}""#)).count(),
            1,
            "every fixture must appear once"
        );
        assert!(digests.insert(hash), "duplicate fixture");
    }
    assert_eq!(digests.len(), 20);
    for field in [
        r#""verdict":"clean""#,
        r#""known_bad":false"#,
        r#""interrupts":false"#,
        r#""veto":null"#,
    ] {
        assert_eq!(scan.matches(field).count(), 20, "{field}: {scan}");
    }

    #[cfg(feature = "instrument")]
    {
        let json = check(&path, &Scratch::new("negative"), 0);
        for field in [
            r#""files":20,"#,
            r#""clean":20,"#,
            r#""confirmed":0,"#,
            r#""unreadable":0,"#,
            r#""read_errors":0,"#,
            r#""truncated":false,"#,
            r#""reports":[]"#,
        ] {
            assert!(json.contains(field), "missing {field}: {json}");
        }
    }
}
