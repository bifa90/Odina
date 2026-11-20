// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
use super::{
    invalid,
    native::{Listener, Peer},
    validate_name,
    wire::{self, Channel},
    Command, Reply, MAX_REPLY,
};
use std::{
    io::{self, Write},
    path::Path,
    sync::{mpsc, Mutex},
    thread,
};

const STATUS: u8 = 1;
const CHECK: u8 = 2;
const SCAN_NOW: u8 = 3;
const STOP: u8 = 4;

pub fn request(name: &str, cmd: Command<'_>) -> io::Result<Reply> {
    validate_name(name)?;
    let (op, scope) = match cmd {
        Command::Status => (STATUS, &[][..]),
        Command::ScanNow => (SCAN_NOW, &[][..]),
        Command::Stop => (STOP, &[][..]),
        Command::Check(p) => (CHECK, p.as_os_str().as_encoded_bytes()),
    };
    let mut ch = Channel::new(Peer::connect(name)?);
    wire::write_frame(&mut ch, op, scope, wire::LIMIT)?;
    ch.timeout(if op == CHECK { wire::SCAN } else { wire::SHORT });
    let (code, body) = wire::read_frame(&mut ch, MAX_REPLY)?;
    if !matches!(code, 0..=2) {
        return Err(invalid("invalid reply code"));
    }
    let body = String::from_utf8(body).map_err(|_| invalid("non-UTF8 reply"))?;
    ch.timeout(wire::SHORT);
    ch.write_all(&[6])?;
    Ok(Reply { code, body })
}

#[derive(Default)]
struct State {
    busy: bool,
    completed: u64,
    last: Option<u8>,
}
impl State {
    fn json(&self) -> String {
        let phase = if self.busy { "scanning" } else { "idle" };
        let last = self.last.map_or_else(|| "null".into(), |c| c.to_string());
        format!(
            r#"{{"schema":"odina-resident/v1","phase":"{phase}","watching":false,"completed":{},"last_exit":{last}}}"#,
            self.completed
        )
    }
}
fn reply(ch: &mut Channel, code: u8, body: impl Into<String>) {
    let _ = ch.reply(Reply {
        code,
        body: body.into(),
    });
}

/// One fixed canonical scope, one worker, no periodic wakeups or jobs lost in
/// an unbounded queue. The service stays live if a client exits during a scan.
pub fn serve(
    name: &str,
    root: &Path,
    ready: impl FnOnce(),
    mut scan: impl FnMut() -> io::Result<(u8, String)> + Send,
) -> io::Result<()> {
    validate_name(name)?;
    let mut listener = Listener::bind(name)?;
    let state = Mutex::new(State::default());
    thread::scope(|threads| {
        let (jobs, rx) = mpsc::sync_channel::<Option<Channel>>(1);
        let shared = &state;
        threads.spawn(move || {
            for client in rx {
                let (code, body) = match scan() {
                    Ok((c, b)) if matches!(c, 0 | 2) && b.len() <= MAX_REPLY => (c, b),
                    _ => (1, r#"{"error":"scan_or_export_failed"}"#.into()),
                };
                {
                    let mut s = shared.lock().unwrap();
                    s.completed += 1;
                    s.last = Some(code);
                    s.busy = false;
                }
                if let Some(mut ch) = client {
                    reply(&mut ch, code, body);
                }
            }
        });
        ready();
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
            if (op == CHECK && arg != root.as_os_str().as_encoded_bytes())
                || (op != CHECK && !arg.is_empty())
            {
                reply(&mut ch, 1, r#"{"error":"invalid_scope"}"#);
                continue;
            }
            let mut s = state.lock().unwrap();
            if op == STATUS {
                let json = s.json();
                drop(s);
                reply(&mut ch, 0, json);
                continue;
            }
            if !matches!(op, CHECK | SCAN_NOW | STOP) {
                drop(s);
                reply(&mut ch, 1, r#"{"error":"unknown_command"}"#);
                continue;
            }
            if s.busy {
                drop(s);
                reply(&mut ch, 1, r#"{"error":"busy"}"#);
                continue;
            }
            if op == STOP {
                drop(s);
                reply(&mut ch, 0, r#"{"stopping":true}"#);
                break;
            }
            s.busy = true;
            let id = s.completed + 1;
            drop(s);
            if op == CHECK {
                jobs.send(Some(ch))
            } else {
                let result = jobs.send(None);
                if result.is_ok() {
                    reply(&mut ch, 0, format!(r#"{{"accepted":{id}}}"#));
                }
                result
            }
            .map_err(|_| io::Error::other("scan worker stopped"))?;
        }
        Ok(()) // drop sender, wake and join worker, release endpoint
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    #[test]
    fn status_remains_live_while_a_job_runs_and_busy_never_means_clean() {
        let name = format!("worker-{}", std::process::id());
        let n = name.clone();
        let (ready, started) = mpsc::channel();
        let (go, wait) = mpsc::channel();
        let service = thread::spawn(move || {
            serve(
                &n,
                Path::new("fixed"),
                || ready.send(()).unwrap(),
                move || {
                    wait.recv_timeout(Duration::from_secs(10))
                        .map_err(io::Error::other)?;
                    Ok((2, "{}".into()))
                },
            )
        });
        started.recv_timeout(Duration::from_secs(10)).unwrap();
        let ask = |c| request(&name, c).unwrap();
        assert_eq!(ask(Command::ScanNow).body, r#"{"accepted":1}"#);
        assert!(ask(Command::Status).body.contains(r#""phase":"scanning""#));
        for op in [
            Command::ScanNow,
            Command::Stop,
            Command::Check(Path::new("fixed")),
        ] {
            assert_eq!(ask(op).code, 1);
        }
        go.send(()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !ask(Command::Status).body.contains(r#""last_exit":2"#) {
            assert!(Instant::now() < deadline);
            thread::yield_now();
        }
        assert_eq!(ask(Command::Stop).code, 0);
        service.join().unwrap().unwrap();
    }
}
