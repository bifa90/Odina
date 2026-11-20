// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
#![cfg(all(feature = "instrument", any(target_os = "linux", windows)))]
mod common;
use std::{
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
const BIN: &str = env!("CARGO_BIN_EXE_odina");
struct Instance(Child, PathBuf);
impl Drop for Instance {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
        let _ = fs::remove_dir_all(&self.1);
    }
}
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
fn check(root: &Path, out: &Path, name: Option<&str>, code: i32) -> String {
    let mut c = Command::new(BIN);
    c.arg("check").arg(root).arg("--out").arg(out);
    if let Some(n) = name {
        c.args(["--service", n]);
    }
    run(&mut c, code);
    fs::read_to_string(out).unwrap()
}
fn parity(a: &str, b: &str) {
    for doc in [a, b] {
        assert!(doc.contains(r#""unreadable":0"#) && doc.contains(r#""truncated":false"#));
    }
    fn fields(s: &str) -> (&str, &str) {
        (
            s.split_once("\"scan\":{")
                .unwrap()
                .1
                .split_once(",\"elapsed_ms\":")
                .unwrap()
                .0,
            s.rsplit_once("\"reports\":").unwrap().1,
        )
    }
    assert_eq!(fields(a), fields(b));
}
fn wait_for(mut query: impl FnMut() -> String, expected: &str) -> String {
    let by = Instant::now() + Duration::from_secs(10);
    loop {
        let text = query();
        if text.contains(expected) {
            return text;
        }
        assert!(Instant::now() < by, "wanted {expected}: {text}");
        thread::sleep(Duration::from_millis(3));
    }
}
fn number(s: &str, field: &str) -> u64 {
    s.split_once(&format!("\"{field}\":"))
        .unwrap()
        .1
        .split(|c: char| !c.is_ascii_digit())
        .next()
        .unwrap()
        .parse()
        .unwrap()
}
#[test]
fn resident_ipc_preserves_g1_reports_and_exit_reuses_index_and_stops() {
    let name = format!("g1-{}", std::process::id());
    let dir = std::env::temp_dir().join(format!("odina-service-{name}"));
    fs::create_dir(&dir).unwrap();
    let root = dir.join("corpus");
    fs::create_dir(&root).unwrap();
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/safe");
    for entry in fs::read_dir(fixtures).unwrap() {
        let e = entry.unwrap();
        fs::copy(e.path(), root.join(e.file_name())).unwrap();
    }
    let start = || {
        let mut c = Command::new(BIN);
        c.arg("serve")
            .arg(&root)
            .args(["--service", &name, "--cache-dir"])
            .arg(dir.join("index"));
        c
    };
    let mut guard = Instance(start().stdout(Stdio::piped()).spawn().unwrap(), dir.clone());
    let stdout = guard.0.stdout.take().unwrap();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut r = BufReader::new(stdout);
        let mut first = String::new();
        let _ = r.read_line(&mut first);
        let _ = tx.send(first);
        let _ = std::io::copy(&mut r, &mut std::io::sink());
    });
    assert!(rx
        .recv_timeout(Duration::from_secs(10))
        .unwrap()
        .starts_with("READY"));
    let control = |op: &str| {
        let mut c = Command::new(BIN);
        c.args([op, "--service", &name]);
        c
    };
    wait_for(|| run(&mut control("status"), 0), r#""watching":true"#);
    assert!(run(&mut control("status"), 0).contains(r#""completed":0,"last_exit":null"#));
    run(&mut start(), 1); // an active listener is never replaced
    let local = check(&root, &dir.join("local.json"), None, 0);
    let remote = check(&root, &dir.join("first.json"), Some(&name), 0);
    parity(&local, &remote);
    assert!(remote.contains(r#""clean":20"#) && remote.contains(r#""reports":[]"#));
    let warm = check(&root, &dir.join("warm.json"), Some(&name), 0);
    parity(&local, &warm);
    if dir.join("index/index-v1.tsv").exists() || std::env::var_os("GITHUB_ACTIONS").is_some() {
        assert!(warm.contains(r#""cache_hits":20"#));
    }
    let sample = root.join("note.txt");
    fs::write(&sample, [b'x'; 68]).unwrap();
    let m = sample.metadata().unwrap().modified().unwrap();
    check(&root, &dir.join("prime.json"), Some(&name), 0);
    fs::write(&sample, common::offline_vector()).unwrap();
    fs::OpenOptions::new()
        .write(true)
        .open(&sample)
        .unwrap()
        .set_modified(m)
        .unwrap();
    let direct = check(&root, &dir.join("mixed-local.json"), None, 2);
    let mixed = check(&root, &dir.join("mixed-ipc.json"), Some(&name), 2);
    parity(&direct, &mixed);
    assert!(
        mixed.contains(r#""confirmed":1"#)
            && mixed.contains(odina_core::g1_reference::EICAR_SHA256)
    );
    let outside = dir.join("other.json");
    run(
        Command::new(BIN)
            .arg("check")
            .arg(&sample)
            .args(["--service", &name, "--out"])
            .arg(&outside),
        1,
    );
    assert!(!outside.exists());
    assert!(run(&mut control("status"), 0).contains(r#""completed":4,"last_exit":2"#));
    // From here no scanner request is made. All new verdicts come from native events.
    let verdict = |rel: &str| {
        run(
            Command::new(BIN).args(["verdict", rel, "--service", &name]),
            0,
        )
    };
    let full = number(&run(&mut control("status"), 0), "full_scans");
    let untouched = fs::read_dir(&root)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.file_name().into_string().unwrap())
        .find(|n| n != "note.txt")
        .unwrap();
    let before = verdict(&untouched);
    fs::write(&sample, [b'X'; 68]).unwrap();
    let clean = wait_for(|| verdict("note.txt"), r#""verdict":"clean""#);
    assert!(clean.contains(r#""origin":"event""#));
    fs::write(&sample, common::offline_vector()).unwrap();
    fs::OpenOptions::new()
        .write(true)
        .open(&sample)
        .unwrap()
        .set_modified(m)
        .unwrap();
    let confirmed = wait_for(|| verdict("note.txt"), r#""verdict":"confirmed""#);
    assert!(number(&confirmed, "generation") > number(&clean, "generation"));
    assert!(
        number(&confirmed, "received_ns") <= number(&confirmed, "invalidated_ns")
            && number(&confirmed, "invalidated_ns") <= number(&confirmed, "published_ns")
    );
    fs::rename(&sample, root.join("moved.txt")).unwrap();
    wait_for(|| verdict("note.txt"), r#""state":"deleted""#);
    wait_for(|| verdict("moved.txt"), r#""verdict":"confirmed""#);
    fs::remove_file(root.join("moved.txt")).unwrap();
    wait_for(|| verdict("moved.txt"), r#""state":"deleted""#);
    let sub = dir.join("incoming");
    fs::create_dir(&sub).unwrap();
    fs::write(sub.join("child.txt"), [b'x'; 68]).unwrap();
    fs::rename(sub, root.join("nested")).unwrap();
    wait_for(|| verdict("nested/child.txt"), r#""verdict":"clean""#);
    // Atomic same-length replacement in a dynamically watched subtree.
    let replacement = dir.join("replacement");
    fs::write(&replacement, common::offline_vector()).unwrap();
    fs::rename(replacement, root.join("nested/child.txt")).unwrap();
    wait_for(|| verdict("nested/child.txt"), r#""verdict":"confirmed""#);
    assert_eq!(verdict(&untouched), before);
    let status = run(&mut control("status"), 0);
    assert_eq!(number(&status, "full_scans"), full);
    assert_eq!(number(&status, "loss_incidents"), 0);
    run(
        Command::new(BIN).args(["verdict", "../outside", "--service", &name]),
        1,
    );
    run(&mut control("stop"), 0);
    assert!(guard.0.wait().unwrap().success());
    run(&mut control("status"), 1);
}
