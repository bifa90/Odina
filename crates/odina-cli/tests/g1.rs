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

fn offline_vector() -> String {
    [
        "X5O!P%@AP[4\\PZX54(P^)7CC)7}$",
        "EICAR-STANDARD-ANTIVIRUS-",
        "TEST-FILE!$H+H*",
    ]
    .concat()
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
            fs::write(&path, offline_vector()).unwrap();
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

#[cfg(feature = "instrument")]
fn indexed(path: &Path, cache: &Path, output: &Path, code: i32) -> String {
    run(
        odina("check", path)
            .arg("--cache-dir")
            .arg(cache)
            .arg("--out")
            .arg(output),
        code,
    );
    fs::read_to_string(output).unwrap()
}

#[cfg(feature = "instrument")]
#[test]
fn changing_68_bytes_with_the_same_path_size_and_mtime_never_reuses_a_verdict() {
    let dir = Scratch::new("same-stamp");
    let path = dir.0.join("note.dat");
    let out = dir.0.join("measurement.json");
    let cache = dir.0.join("private-index");
    let benign = [b'a'; 68];
    fs::write(&path, benign).unwrap();
    let mtime = path.metadata().unwrap().modified().unwrap();
    assert!(indexed(&path, &cache, &out, 0).contains(r#""bloom_negatives":1,"sha_files":0"#));
    let rewrite = |bytes: &[u8]| {
        fs::write(&path, bytes).unwrap();
        fs::OpenOptions::new()
            .write(true)
            .open(&path)
            .unwrap()
            .set_modified(mtime)
            .unwrap();
        assert_eq!(path.metadata().unwrap().modified().unwrap(), mtime);
    };
    rewrite(offline_vector().as_bytes());
    for _ in 0..2 {
        let doc = indexed(&path, &cache, &out, 2);
        assert!(doc.contains(r#""confirmed":1"#) && doc.contains(EICAR_SHA256));
        assert!(
            doc.contains(r#""cache_hits":0,"bloom_negatives":0,"sha_files":1,"streamed_bytes":68"#)
        );
    }
    // Same necessary prefix AND length, but not the exact reference.
    let mut near = benign;
    near[..8].copy_from_slice(b"X5O!P%@A");
    rewrite(&near);
    let doc = indexed(&path, &cache, &out, 0);
    assert!(doc.contains(r#""reports":[]"#) && doc.contains(r#""sha_files":1"#));
}

#[cfg(feature = "instrument")]
#[test]
fn negative_cold_and_warm_passes_keep_counts_and_avoid_sha_without_changing_scan() {
    let dir = Scratch::new("delta-cache");
    let corpus = dir.0.join("corpus");
    fs::create_dir(&corpus).unwrap();
    let cache = dir.0.join("index");
    let out = dir.0.join("measurement.json");
    let original = corpus.join("document-with-private-name.dat");
    fs::write(&original, [b'z'; 512]).unwrap();
    let cold = indexed(&corpus, &cache, &out, 0);
    assert!(
        cold.contains(r#""cache_hits":0,"bloom_negatives":1,"sha_files":0,"streamed_bytes":512"#)
    );
    let warm = indexed(&corpus, &cache, &out, 0);
    let idx = cache.join("index-v1.tsv");
    // Unsupported filesystems legitimately never issue a hint. On native
    // CI runners the scratch volume must exercise the cross-process hit.
    if std::env::var_os("GITHUB_ACTIONS").is_some() {
        assert!(idx.is_file(), "expected native CI test volume");
    }
    if idx.is_file() {
        assert!(
            warm.contains(r#""cache_hits":1,"bloom_negatives":0,"sha_files":0,"streamed_bytes":0"#)
        );
        assert!(!fs::read_to_string(&idx).unwrap().contains("private-name"));
        fs::write(&idx, b"old-or-broken\n").unwrap();
        let recovered = indexed(&corpus, &cache, &out, 0);
        assert!(recovered.contains(r#""cache_hits":0,"bloom_negatives":1"#));
    }
    for json in [&cold, &warm] {
        assert!(json.contains(r#""files":1,"bytes":512,"clean":1"#));
        assert!(json.contains(r#""reports":[]"#));
    }
    let public_contract = run(odina("scan", &original).arg("--json"), 0);
    let full = odina_core::hashing::hash_bytes(&[b'z'; 512]).full;
    assert!(public_contract.contains(&format!(r#""full":"{full}""#)));
    run(
        odina("check", &corpus)
            .arg("--cache-dir")
            .arg(corpus.join("unsafe"))
            .arg("--out")
            .arg(&out),
        1,
    );
    assert!(!corpus.join("unsafe").exists());
}
