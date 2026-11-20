// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! The current catalog produces a V5 positive, not a heuristic suspect. Seed ONLY
//! the quick suspect row here; the host, frames, resident and full digest are real.
#![cfg(any(target_os = "linux", windows))]
use odina_resident::{request, serve, Command, FileVerdict, Inventory, Scans};
use serde_json::{json, Value};
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command as Process, Stdio},
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc, Arc,
    },
    thread,
    time::{Duration, Instant},
};
struct Fixture {
    root: PathBuf,
    calls: Arc<AtomicUsize>,
}
impl Scans for Fixture {
    fn inventory(&mut self, seen: &mut dyn FnMut(&Path, FileVerdict)) -> io::Result<Inventory> {
        seen(
            Path::new("sample.bin"),
            FileVerdict {
                band: "suspicious",
                bytes: 68,
            },
        );
        Ok(Inventory {
            code: 0,
            report: "{}".into(),
            enumerated: true,
        })
    }
    fn changed(&mut self, _: &Path) -> io::Result<FileVerdict> {
        unreachable!()
    }
    fn forget(&mut self, _: &Path) {}
    fn verify(&mut self, p: &Path) -> io::Result<FileVerdict> {
        assert_eq!(p, Path::new("sample.bin"));
        self.calls.fetch_add(1, Ordering::SeqCst);
        let a =
            odina_cache::Scanner::resident(&self.root, None)?.verify_file(&self.root.join(p))?;
        assert!(a.known_bad);
        Ok(FileVerdict {
            band: "confirmed",
            bytes: a.hashes.size,
        })
    }
}
fn native(input: Value) -> Value {
    let host = env!("CARGO_BIN_EXE_odina-browser-host");
    let m: Value = serde_json::from_str(include_str!(
        "../../../browser-extension/hosts/chromium.json"
    ))
    .unwrap();
    let mut child = Process::new(host)
        .arg(m["allowed_origins"][0].as_str().unwrap())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let bytes = input.to_string();
    let mut stdin = child.stdin.take().unwrap();
    stdin
        .write_all(&(bytes.len() as u32).to_ne_bytes())
        .unwrap();
    stdin.write_all(bytes.as_bytes()).unwrap();
    drop(stdin);
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let out = out.stdout;
    assert_eq!(
        u32::from_ne_bytes(out[..4].try_into().unwrap()) as usize,
        out.len() - 4
    );
    serde_json::from_slice(&out[4..]).unwrap()
}
#[test]
fn suspect_requires_its_own_unconsumed_receipt_and_only_one_file_is_verified() {
    let base = std::env::temp_dir().join(format!("odina-consent-{}", std::process::id()));
    fs::create_dir(&base).unwrap();
    let root = base.canonicalize().unwrap();
    let bytes = [
        "X5O!P%@AP[4\\PZX54(P^)7CC)7}$",
        "EICAR-STANDARD-ANTIVIRUS-",
        "TEST-FILE!$H+H*",
    ]
    .concat();
    fs::write(root.join("sample.bin"), &bytes).unwrap();
    let (tx, rx) = mpsc::channel();
    let calls = Arc::new(AtomicUsize::new(0));
    let fixture = Fixture {
        root: root.clone(),
        calls: calls.clone(),
    };
    let scope = root.clone();
    let t = thread::spawn(move || serve("browser", &scope, || tx.send(()).unwrap(), fixture));
    // If a real instance owns this endpoint, fail before making any request to it.
    rx.recv_timeout(Duration::from_secs(10))
        .expect("requires an unused browser instance");
    let by = Instant::now() + Duration::from_secs(10);
    while !request("browser", Command::Status)
        .unwrap()
        .body
        .contains(r#""watching":true"#)
    {
        assert!(Instant::now() < by);
        thread::yield_now();
    }
    // Browsers select the long, ordinary spelling; TEMP may be an 8.3 alias on Windows.
    let path = root.to_str().unwrap();
    let path = path
        .strip_prefix(r"\\?\")
        .unwrap_or(path)
        .replace('\\', "/");
    let url = format!(
        "file://{}/{}",
        if cfg!(windows) {
            format!("/{path}")
        } else {
            path
        },
        "sample.bin"
    );
    let dir = url.strip_suffix("sample.bin").unwrap();
    let reply = native(json!({"op":"inspect","url":dir}));
    assert_eq!(reply["result"], "suspicious");
    assert_eq!(
        native(json!({"op":"deep","url":url,"ticket":reply["ticket"]})),
        json!({"result":"unknown"})
    );
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let verify = json!({"op":"deep","url":dir,"ticket":reply["ticket"]});
    assert_eq!(native(verify.clone()), json!({"result":"threat"}));
    assert_eq!(native(verify), json!({"result":"unknown"}));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(fs::read(root.join("sample.bin")).unwrap(), bytes.as_bytes());
    assert!(request("browser", Command::Status)
        .unwrap()
        .body
        .contains(r#""full_scans":1"#));
    assert_eq!(request("browser", Command::Stop).unwrap().code, 0);
    t.join().unwrap().unwrap();
    fs::remove_dir_all(base).unwrap();
}
