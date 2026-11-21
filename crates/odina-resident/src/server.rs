// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
use super::{
    invalid,
    monitor::{Shared, Task},
    native::{Listener, Peer},
    validate_name, watch,
    wire::{self, Channel},
    Command, Reply, Scans, MAX_REPLY,
};
use std::{
    io::{self, Write},
    path::Path,
    thread,
};

const STATUS: u8 = 1;
const CHECK: u8 = 2;
const SCAN_NOW: u8 = 3;
const STOP: u8 = 4;
const VERDICT: u8 = 5;
const INSPECT: u8 = 6;
const VERIFY: u8 = 7;
const SUSPEND: u8 = 8;
const RESUME: u8 = 9;
pub(super) const UNKNOWN: &str = r#"{"result":"unknown"}"#;

pub fn request(name: &str, cmd: Command<'_>) -> io::Result<Reply> {
    validate_name(name)?;
    let mut encoded = Vec::new();
    let (op, scope) = match cmd {
        Command::Status => (STATUS, &[][..]),
        Command::ScanNow => (SCAN_NOW, &[][..]),
        Command::Stop => (STOP, &[][..]),
        Command::Suspend => (SUSPEND, &[][..]),
        Command::Resume => (RESUME, &[][..]),
        Command::Verdict(p) => (
            VERDICT,
            p.to_str()
                .ok_or_else(|| invalid("lookup expects UTF-8"))?
                .as_bytes(),
        ),
        Command::Inspect(p) => (
            INSPECT,
            p.to_str()
                .ok_or_else(|| invalid("selection requires UTF-8"))?
                .as_bytes(),
        ),
        Command::Verify(p, t) => {
            encoded.extend(t.to_be_bytes());
            encoded.extend(
                p.to_str()
                    .ok_or_else(|| invalid("selection requires UTF-8"))?
                    .as_bytes(),
            );
            (VERIFY, encoded.as_slice())
        }
        Command::Check(p) => (CHECK, p.as_os_str().as_encoded_bytes()),
    };
    let mut ch = Channel::new(Peer::connect(name)?);
    wire::write_frame(&mut ch, op, scope, wire::LIMIT)?;
    ch.timeout(if matches!(op, CHECK | VERIFY) {
        wire::SCAN
    } else {
        wire::SHORT
    });
    let reply_limit = if matches!(op, INSPECT | VERIFY) {
        128
    } else {
        MAX_REPLY
    };
    let (code, body) = wire::read_frame(&mut ch, reply_limit)?;
    if !matches!(code, 0..=2) {
        return Err(invalid("invalid reply code"));
    }
    let body = String::from_utf8(body).map_err(|_| invalid("non-UTF8 reply"))?;
    ch.timeout(wire::SHORT);
    ch.write_all(&[6])?;
    Ok(Reply { code, body })
}

fn reply(ch: &mut Channel, code: u8, body: impl Into<String>) {
    let _ = ch.reply(Reply {
        code,
        body: body.into(),
    });
}
fn worker(shared: &Shared, cancel: &watch::Cancel, mut scans: impl Scans) {
    loop {
        let mut s = shared.state.lock().unwrap();
        let task = loop {
            if s.stopping {
                return;
            }
            if let Some(work) = s.next() {
                break work;
            }
            s = shared.wake.wait(s).unwrap();
        };
        drop(s);
        match task {
            Task::Verify(key, revision, mut ch) => {
                let current = || shared.state.lock().unwrap().still_suspect(&key, revision);
                let verdict = if current() {
                    scans.verify(&key).ok()
                } else {
                    None
                };
                let mut s = shared.state.lock().unwrap();
                let band = if s.still_suspect(&key, revision) {
                    verdict.map(|v| v.band)
                } else {
                    None
                };
                s.verify_busy = false;
                drop(s);
                // A verified-but-still-suspicious result has no new consent receipt.
                let label = match band {
                    Some("confirmed") => "threat",
                    Some("suspicious" | "high_risk") => "suspicious",
                    Some("clean") => "clean",
                    _ => "unknown",
                };
                reply(&mut ch, 0, format!(r#"{{"result":"{label}"}}"#));
            }
            Task::One(epoch, path, e) => {
                let v = if e.remove {
                    scans.forget(&path);
                    None
                } else {
                    scans.changed(&path).ok()
                };
                shared.state.lock().unwrap().publish(epoch, &path, e, v);
            }
            Task::Full(epoch, manual) => {
                let mut fits = true;
                let result = scans.inventory(&mut |path, v| {
                    fits &= shared.state.lock().unwrap().seed(epoch, path, v);
                });
                let (code, body, enumerated) = match result {
                    Ok(r) if matches!(r.code, 0 | 2) && r.report.len() <= MAX_REPLY => {
                        (r.code, r.report, r.enumerated && fits)
                    }
                    _ => (1, r#"{"error":"scan_or_export_failed"}"#.into(), false),
                };
                let mut s = shared.state.lock().unwrap();
                s.end_full(epoch, enumerated);
                if manual.is_some() {
                    s.manual_busy = false;
                    s.completed += 1;
                    s.last = Some(code);
                }
                drop(s);
                cancel.wake();
                if let Some(Some(mut ch)) = manual {
                    reply(&mut ch, code, body);
                }
            }
        }
    }
}
fn control(mut listener: Listener, root: &Path, single: bool, shared: &Shared) -> io::Result<()> {
    loop {
        let peer = match listener.accept() {
            Ok(s) => s,
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::Interrupted | io::ErrorKind::PermissionDenied
                ) =>
            {
                continue
            }
            Err(e) => return Err(e),
        };
        let mut ch = Channel::new(peer);
        let (op, arg) = match wire::read_frame(&mut ch, wire::LIMIT) {
            Ok(f) => f,
            Err(_) => continue,
        };
        let mut s = shared.state.lock().unwrap();
        let (code, body) = match op {
            STATUS if arg.is_empty() => (0, s.json()),
            SUSPEND if arg.is_empty() => {
                s.suspended = true;
                (0, r#"{"suspended":true}"#.into())
            }
            RESUME if arg.is_empty() => {
                s.suspended = false;
                (0, r#"{"suspended":false}"#.into())
            }
            VERDICT => match std::str::from_utf8(&arg)
                .ok()
                .map(Path::new)
                .filter(|p| watch::relative(p))
            {
                Some(p) => (0, s.lookup(p)),
                None => (1, r#"{"error":"invalid_relative_key"}"#.into()),
            },
            INSPECT => {
                let key = crate::selection::scope_key(root, single, &arg);
                (0, key.map_or_else(|| UNKNOWN.into(), |key| s.inspect(&key)))
            }
            VERIFY => {
                if !s.manual_busy && !s.full_busy && !s.verify_busy && arg.len() > 8 {
                    let (ticket, path) = arg.split_at(8);
                    let job = crate::selection::scope_key(root, single, path).and_then(|key| {
                        s.consume(&key, u64::from_be_bytes(ticket.try_into().unwrap()))
                    });
                    if let Some((key, revision)) = job {
                        s.verify_busy = true;
                        s.verification = Some((key, revision, ch));
                        shared.wake.notify_one();
                        continue;
                    }
                }
                (0, UNKNOWN.into())
            }
            CHECK if arg != root.as_os_str().as_encoded_bytes() => {
                (1, r#"{"error":"invalid_scope"}"#.into())
            }
            _ if op != CHECK && !arg.is_empty() => (1, r#"{"error":"unexpected_payload"}"#.into()),
            CHECK | SCAN_NOW | STOP if s.manual_busy || s.full_busy || s.verify_busy => {
                (1, r#"{"error":"busy"}"#.into())
            }
            CHECK | SCAN_NOW if s.suspended => (1, r#"{"error":"suspended"}"#.into()),
            STOP => {
                drop(s);
                reply(&mut ch, 0, r#"{"stopping":true}"#);
                return Ok(());
            }
            CHECK | SCAN_NOW => {
                s.manual_busy = true;
                if op == CHECK {
                    s.manual = Some(Some(ch));
                    shared.wake.notify_one();
                    continue;
                }
                s.manual = Some(None);
                shared.wake.notify_one();
                (0, format!(r#"{{"accepted":{}}}"#, s.completed + 1))
            }
            _ => (1, r#"{"error":"unknown_command"}"#.into()),
        };
        drop(s); // never do IPC, filesystem I/O or scoring under the publication lock
        reply(&mut ch, code, body);
    }
}
pub fn serve(name: &str, root: &Path, ready: impl FnOnce(), scans: impl Scans) -> io::Result<()> {
    validate_name(name)?;
    let listener = Listener::bind(name)?;
    let single = root.metadata()?.is_file();
    let shared = Shared::default();
    let cancel = watch::Cancel::new()?;
    // READY is endpoint readiness only; watching:true requires subscription + reconciliation.
    ready();
    thread::scope(|threads| {
        let shared = &shared;
        threads.spawn(|| {
            if let Err(e) = watch::run(root, single, shared, &cancel) {
                shared.offline();
                eprintln!("odina: watcher offline; coverage unknown: {e}");
            }
        });
        let signal = &cancel;
        threads.spawn(move || worker(shared, signal, scans));
        let result = control(listener, root, single, shared);
        shared.state.lock().unwrap().stopping = true;
        shared.wake.notify_one();
        cancel.wake();
        result
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FileVerdict, Inventory};
    use std::{
        sync::mpsc,
        time::{Duration, Instant},
    };
    struct BlockAfterBootstrap {
        count: usize,
        wait: mpsc::Receiver<()>,
    }
    impl Scans for BlockAfterBootstrap {
        fn inventory(&mut self, _: &mut dyn FnMut(&Path, FileVerdict)) -> io::Result<Inventory> {
            if self.count != 0 {
                self.wait
                    .recv_timeout(Duration::from_secs(10))
                    .map_err(io::Error::other)?;
            }
            self.count += 1;
            Ok(Inventory {
                code: 2,
                report: "{}".into(),
                enumerated: true,
            })
        }
        fn changed(&mut self, _: &Path) -> io::Result<FileVerdict> {
            unreachable!()
        }
        fn forget(&mut self, _: &Path) {}
        fn verify(&mut self, _: &Path) -> io::Result<FileVerdict> {
            unreachable!()
        }
    }
    #[test]
    fn status_remains_live_during_inventory_and_busy_never_means_clean() {
        let name = format!("worker-{}", std::process::id());
        let n = name.clone();
        let dir = std::env::temp_dir().join(&name);
        std::fs::create_dir(&dir).unwrap();
        let root = dir.canonicalize().unwrap();
        let scope = root.clone();
        let (ready, started) = mpsc::channel();
        let (go, wait) = mpsc::channel();
        let srv = thread::spawn(move || {
            serve(
                &n,
                &root,
                || ready.send(()).unwrap(),
                BlockAfterBootstrap { count: 0, wait },
            )
        });
        started.recv_timeout(Duration::from_secs(10)).unwrap();
        let ask = |c| request(&name, c).unwrap();
        let until = Instant::now() + Duration::from_secs(10);
        while !ask(Command::Status).body.contains(r#""watching":true"#) {
            assert!(Instant::now() < until);
            thread::yield_now();
        }
        assert_eq!(ask(Command::ScanNow).body, r#"{"accepted":1}"#);
        assert!(ask(Command::Status).body.contains(r#""phase":"scanning""#));
        for op in [Command::ScanNow, Command::Stop, Command::Check(&scope)] {
            assert_eq!(ask(op).code, 1);
        }
        go.send(()).unwrap();
        while !ask(Command::Status).body.contains(r#""last_exit":2"#) {
            assert!(Instant::now() < until);
            thread::yield_now();
        }
        assert_eq!(ask(Command::Suspend).body, r#"{"suspended":true}"#);
        assert!(ask(Command::Status).body.contains(r#""phase":"suspended""#));
        assert_eq!(ask(Command::ScanNow).body, r#"{"error":"suspended"}"#);
        assert_eq!(ask(Command::Resume).body, r#"{"suspended":false}"#);
        assert_eq!(ask(Command::Stop).code, 0);
        srv.join().unwrap().unwrap();
        std::fs::remove_dir(dir).unwrap();
    }
}
