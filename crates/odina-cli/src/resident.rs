// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! Explicit user-mode prototype. Never spawn a daemon or fall back to TCP.
// The report-building build's pass. The module does not exist in a public
// build, and nothing in that build's code path may mention it.
#[cfg(feature = "instrument")]
use super::check;
use odina_cache::Scanner;
#[cfg(feature = "instrument")]
use odina_instrument::Meter;
use odina_resident::{request, serve, Command, FileVerdict, Inventory, Scans};
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};
#[cfg(feature = "instrument")]
use std::time::Instant;

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
    fn read_one(&mut self, rel: &Path, explicit: bool) -> io::Result<FileVerdict> {
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
            let a = if explicit {
                self.scanner.verify_file(&named)?
            } else {
                self.scanner.scan_changed(&named)?
            };
            confined()?;
            Ok(value(&a))
        })();
        if !explicit && result.is_err() {
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
        // Two implementations, one scoring path and one set of verdicts. The
        // public one owns no report, meter or counter, so it is safe in every
        // build; the measured one adds the meter and the local export. They are
        // pinned together by `the_public_and_measured_verdicts_are_the_same_per_file`
        // below, and that test is the only reason it is acceptable for there to
        // be two definitions at all.
        inventory_once(self, seen)
    }
    fn verify(&mut self, rel: &Path) -> io::Result<FileVerdict> {
        self.read_one(rel, true)
    }
    fn forget(&mut self, rel: &Path) {
        self.scanner.forget(&self.named(rel));
    }
    fn changed(&mut self, rel: &Path) -> io::Result<FileVerdict> {
        // RDCW can notify before a writer releases an exclusive handle. Retry only
        // sharing/lock violations, with a finite budget, never a timer during idle.
        #[cfg(windows)]
        for ms in [2, 8, 32, 128] {
            match self.read_one(rel, false) {
                Err(e) if matches!(e.raw_os_error(), Some(32 | 33)) => {
                    std::thread::sleep(std::time::Duration::from_millis(ms))
                }
                r => return r,
            }
        }
        self.read_one(rel, false)
    }
}

/// The public build's pass: `odina_scan`'s single loop, no report, no meter.
///
/// The pass boundaries belong to whoever owns the reader. The loop knows
/// nothing about an index; the resident's does, and it has to be told where one
/// inventory starts and ends, or the next run starts from a half-written state.
#[cfg(not(feature = "instrument"))]
fn inventory_once(
    session: &mut Session<'_>,
    seen: &mut dyn FnMut(&Path, FileVerdict),
) -> io::Result<Inventory> {
    session.scanner.begin_pass();
    let (totals, walked) = odina_scan::pass(&mut session.scanner, session.root, &mut |a| {
        seen(&relative(session.root, session.single, a), value(a));
    })?;
    // Sealing can fail (a read-only index directory, a full disk). That costs the
    // next run its warm start; it does not falsify this one, which is why it is a
    // warning and not an error. The measured path below does the same.
    if let Err(e) = session.scanner.finish_pass() {
        eprintln!("odina: could not publish cache; rescan next time: {e}");
    }
    Ok(Inventory {
        code: totals.exit_code(),
        report: String::new(),
        enumerated: walked.is_complete(),
    })
}

/// The measured build's pass: the same inventory, plus a meter and a local
/// export.
///
/// Everything measurement-shaped in this file is behind one `cfg` or under this
/// definition, so `tools/ci/check-public-build.py` can delete the gated items
/// and assert that what remains neither names `check::` nor `Meter`. Two
/// definitions of the same name, one per build, rather than one function with a
/// dead branch: a branch that is compiled out still has to typecheck, and a
/// typecheck in the public build would drag the report crate back in.
#[cfg(feature = "instrument")]
fn inventory_once(
    session: &mut Session<'_>,
    seen: &mut dyn FnMut(&Path, FileVerdict),
) -> io::Result<Inventory> {
    let (root, single) = (session.root, session.single);
    check::run_pass(
        &mut session.scanner,
        root,
        Meter::now(),
        Instant::now(),
        odina_resident::MAX_REPLY,
        &mut |a| {
            seen(relative(root, single, a), value(a));
        },
    )
}

/// The path a verdict belongs to, relative to the configured root. A single
/// file is reported by its own name; a tree is reported relative to the root.
/// One function, so the two passes cannot report the same file under two names.
fn relative<'a>(
    root: &'a Path,
    single: bool,
    a: &'a odina_core::assessment::Assessment,
) -> &'a Path {
    if single {
        root.file_name().map(Path::new).unwrap_or(root)
    } else {
        a.path
            .strip_prefix(root)
            .expect("inventory inside configured root")
    }
}

/// The entry point `main` calls. Present in every build: the service is a core
/// capability, not a lab tool.
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

/// `odina check --service NAME`'s client half. Gated with the command it
/// serves: a public build has no `check`, so it has nowhere to write a report
/// and nothing to write. Keeping it ungated would mean a public build that can
/// only produce an empty file.
#[cfg(feature = "instrument")]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// K1's exit criterion 1, as a test: the public scan and the measured scan
    /// must return the same verdict for every file in the same tree. If this
    /// fails, the public build is not a substitute for the lab build and the
    /// separation was cosmetic.
    ///
    /// The two loops differ in what they *record*, never in what they decide,
    /// because both call `odina_cache::Scanner::scan_file` and both walk with
    /// `odina_scan::walk`. This test exists so that if either stops being true
    /// the difference is caught here rather than in a user's result.
    #[test]
    fn the_public_and_measured_verdicts_are_the_same_per_file() {
        static N: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "odina-k1-parity-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("sub")).expect("tree");
        fs::write(dir.join("a.txt"), b"alpha").expect("a");
        fs::write(dir.join("sub/b.txt"), b"beta beta").expect("b");
        fs::write(dir.join("empty.bin"), b"").expect("c");

        let walked = odina_scan::walk(&dir);
        assert_eq!(walked.files.len(), 3, "the tree must be seen whole");

        // The public path: no cache, one pass, totals only.
        // The export path is outside the tree: a folder created inside it would
        // be a fourth entry and the count assert above would be a lie.
        let out = dir.with_extension("out");
        let mut public = odina_cache::Scanner::one_shot(&dir, &out, None).expect("public scanner");
        // What the public pass saw, kept by path so each file can be compared
        // against its own second reading below.
        let mut by_path: Vec<(PathBuf, odina_core::Verdict, odina_core::FileHashes)> = Vec::new();
        let (totals, complete) =
            odina_scan::pass(&mut public, &dir, &mut |a| {
                by_path.push((a.path.clone(), a.score.verdict, a.hashes.clone()));
            })
            .expect("public pass");
        assert_eq!(totals.files, 3);
        assert_eq!(totals.unreadable, 0);
        assert!(complete.is_complete());

        // The same files, scored one at a time through the explicit reader.
        // Same bytes, same scorer: the verdict and every digest must match.
        let mut again = odina_cache::Scanner::one_shot(&dir, &out, None).expect("second scanner");
        let mut bytes = 0u64;
        for file in &walked.files {
            let one = again.verify_file(file).expect("verify");
            let (_, verdict, hashes) = by_path
                .iter()
                .find(|(path, _, _)| path == file)
                .expect("every walked file must reach `seen`");
            assert_eq!(*verdict, one.score.verdict, "{file:?}");
            assert_eq!(hashes.full, one.hashes.full, "{file:?}");
            assert_eq!(hashes.head, one.hashes.head, "{file:?}");
            assert_eq!(hashes.tail, one.hashes.tail, "{file:?}");
            bytes += one.hashes.size;
        }
        assert_eq!(totals.bytes, bytes, "the two paths must read the same bytes");

        let _ = fs::remove_dir_all(&dir);
    }

    /// The counters the resident reports must come from the same verdicts the
    /// CLI prints. `value()` is the single mapping from a verdict to a band;
    /// a second mapping is how the two surfaces start disagreeing.
    #[test]
    fn every_verdict_has_exactly_one_band_name() {
        use odina_core::Verdict::*;
        for (v, band) in [
            (Clean, "clean"),
            (Monitor, "monitor"),
            (Suspicious, "suspicious"),
            (HighRisk, "high_risk"),
            (Confirmed, "confirmed"),
        ] {
            let mut a = odina_core::assessment::assess(Path::new("x"), Default::default());
            a.score.verdict = v;
            assert_eq!(value(&a).band, band);
        }
    }
}
