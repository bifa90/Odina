// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! Explicit user-mode prototype. Never spawn a daemon or fall back to TCP.
use super::check;
use odina_cache::Scanner;
use odina_instrument::Meter;
use odina_resident::{request, serve, Command, FileVerdict, Inventory, Scans, MAX_REPLY};
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
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
struct Session<'a> {
    scanner: Scanner,
    root: &'a Path,
    scope: PathBuf,
    single: bool,
}
fn value(a: &odina_core::assessment::Assessment) -> FileVerdict {
    use odina_core::Verdict::*;
    FileVerdict {
        bytes: a.hashes.size,
        band: match a.score.verdict {
            Clean => "clean",
            Monitor => "monitor",
            Suspicious => "suspicious",
            HighRisk => "high_risk",
            Confirmed => "confirmed",
        },
    }
}
impl Session<'_> {
    fn read_change(&mut self, rel: &Path) -> io::Result<FileVerdict> {
        let named = self.named(rel);
        let expected = if self.single {
            self.scope.clone()
        } else {
            self.scope.join(rel)
        };
        let confined = || -> io::Result<()> {
            if named.canonicalize()? == expected {
                Ok(())
            } else {
                Err(bad("changed path redirected"))
            }
        };
        // Canonical name for confinement, original spelling for all scoring. Forget on
        // failure as well: a newly recorded private hint isn't a successful publication.
        let result = (|| {
            confined()?;
            let a = self.scanner.scan_changed(&named)?;
            confined()?;
            Ok(value(&a))
        })();
        if result.is_err() {
            self.scanner.forget(&named);
        }
        result
    }
    fn named(&self, rel: &Path) -> PathBuf {
        if self.single {
            self.root.to_owned()
        } else {
            self.root.join(rel)
        }
    }
    fn check_root(&self) -> io::Result<()> {
        data_target(self.root)?;
        if self.root.canonicalize()? != self.scope || self.root.metadata()?.is_file() != self.single
        {
            return Err(bad("configured root changed"));
        }
        Ok(())
    }
}
impl Scans for Session<'_> {
    fn inventory(&mut self, seen: &mut dyn FnMut(&Path, FileVerdict)) -> io::Result<Inventory> {
        self.check_root()?;
        let (root, scope, single) = (self.root, &self.scope, self.single);
        check::run_pass(
            &mut self.scanner,
            root,
            Meter::now(),
            Instant::now(),
            MAX_REPLY,
            &mut |a| {
                let rel = if single {
                    Path::new(scope.file_name().unwrap())
                } else {
                    a.path
                        .strip_prefix(root)
                        .expect("inventory inside configured root")
                };
                seen(rel, value(a));
            },
        )
    }
    fn forget(&mut self, rel: &Path) {
        self.scanner.forget(&self.named(rel));
    }
    fn changed(&mut self, rel: &Path) -> io::Result<FileVerdict> {
        // RDCW can notify before a writer releases an exclusive handle. Retry only
        // sharing/lock violations, with a finite budget, never a timer during idle.
        #[cfg(windows)]
        for ms in [2, 8, 32, 128] {
            match self.read_change(rel) {
                Err(e) if matches!(e.raw_os_error(), Some(32 | 33)) => {
                    std::thread::sleep(std::time::Duration::from_millis(ms))
                }
                r => return r,
            }
        }
        self.read_change(rel)
    }
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
        let scanner = Scanner::resident(root, dir)?;
        let session = Session {
            scanner,
            root,
            scope: scope.clone(),
            single: scope.is_file(),
        };
        serve(
            name,
            &scope,
            || println!("READY odina-resident/v2 (check status for coverage)"),
            session,
        )?;
        return Ok(0);
    }
    if let ("verdict", [p, flag, name]) = (cmd, args) {
        if flag != "--service" {
            return Err(bad("verdict <relative-path> --service NAME"));
        }
        let r = request(name, Command::Verdict(Path::new(p)))?;
        println!("{}", r.body);
        return Ok(r.code);
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
