// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! Explicit user-mode prototype. Never spawn a daemon or fall back to TCP.
use super::check;
use odina_cache::Scanner;
use odina_instrument::Meter;
use odina_resident::{request, serve, Command, MAX_REPLY};
use std::{
    fs,
    io::{self, Write},
    path::Path,
    process::ExitCode,
    time::Instant,
};

fn code(r: io::Result<u8>) -> ExitCode {
    match r {
        Ok(c) => ExitCode::from(c),
        Err(e) => {
            eprintln!("odina: {e}");
            ExitCode::FAILURE
        }
    }
}
fn bad(s: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, s)
}
fn data_target(p: &Path) -> io::Result<()> {
    let m = fs::symlink_metadata(p)?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if m.file_attributes() & 0x400 != 0 {
            return Err(bad("a reparse point is not a service root"));
        }
    }
    if !m.is_dir() && !m.is_file() {
        return Err(bad("expected an ordinary file or directory"));
    }
    Ok(())
}
pub fn command(cmd: &str, args: &[String]) -> ExitCode {
    code(run(cmd, args))
}
fn run(cmd: &str, args: &[String]) -> io::Result<u8> {
    if cmd == "serve" {
        let (root, name, dir) = match args {
            [p, flag, n] if flag == "--service" => (p, n, None),
            [p, flag, n, cache, d] if flag == "--service" && cache == "--cache-dir" => {
                (p, n, Some(Path::new(d)))
            }
            _ => return Err(bad("serve <path> --service NAME [--cache-dir DIR]")),
        };
        // Canonicalize for scope binding only. Feeding Windows' verbatim path
        // prefix (or a resolved alias) to the scorer/exporter would change the
        // reported path pattern relative to a one-shot scan of the same input.
        let root = Path::new(root);
        data_target(root)?;
        let scope = root.canonicalize()?;
        let mut scanner = Scanner::resident(root, dir)?;
        serve(
            name,
            &scope,
            || println!("READY odina-resident/v1 (manual scans; no watcher)"),
            || {
                data_target(root)?;
                if root.canonicalize()? != scope {
                    return Err(bad("configured root changed"));
                }
                check::run_pass(&mut scanner, root, Meter::now(), Instant::now(), MAX_REPLY)
            },
        )?;
        return Ok(0);
    }
    let [flag, name] = args else {
        return Err(bad("expected --service NAME"));
    };
    if flag != "--service" {
        return Err(bad("expected --service NAME"));
    }
    let op = match cmd {
        "status" => Command::Status,
        "scan-now" => Command::ScanNow,
        "stop" => Command::Stop,
        _ => return Err(bad("unknown command")),
    };
    let r = request(name, op)?;
    println!("{}", r.body);
    Ok(r.code)
}

pub fn client_check(name: &str, target: &Path, out: &Path) -> ExitCode {
    code((|| {
        let target = target.canonicalize()?;
        let abs = std::env::current_dir()?.join(out);
        let dest = abs
            .parent()
            .ok_or_else(|| bad("expected a report filename"))?
            .canonicalize()?
            .join(
                abs.file_name()
                    .ok_or_else(|| bad("expected report filename"))?,
            );
        if dest.starts_with(&target) || fs::symlink_metadata(&dest).is_ok() {
            return Err(bad("IPC --out must be a new file outside the corpus"));
        }
        let r = request(name, Command::Check(&target))?;
        if r.code == 1 {
            eprintln!("odina: {}", r.body);
            return Ok(1);
        }
        // Only the client writes its destination. No output path crosses IPC, and
        // create_new cannot truncate the target of an existing link or hardlink.
        let mut opts = fs::OpenOptions::new();
        opts.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        opts.open(dest)?.write_all(r.body.as_bytes())?;
        println!("الملف: {}", out.display());
        Ok(r.code)
    })())
}
