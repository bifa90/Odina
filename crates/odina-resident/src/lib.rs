// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! Native filesystem watcher and local per-user control plane. The operator selects the scope at startup.
//! Nothing here knows how to score a file, start itself, or open a TCP socket.
use std::{io, path::Path};

#[cfg(any(target_os = "linux", windows))]
mod monitor;
#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod native;
#[cfg(windows)]
#[path = "windows.rs"]
mod native;
#[cfg(any(target_os = "linux", windows))]
mod server;
#[cfg(any(target_os = "linux", windows))]
mod watch;
#[cfg(any(target_os = "linux", windows))]
mod wire;
#[cfg(any(target_os = "linux", windows))]
pub use server::{request, serve};

mod selection;
pub use selection::local_selection;

pub const MAX_REPLY: usize = 8 * 1024 * 1024;
pub enum Command<'a> {
    Status,
    ScanNow,
    /// Already canonical on the client. Compared as opaque bytes, never opened by the server.
    Check(&'a Path),
    Stop,
    Suspend,
    Resume,
    Drain,
    /// Relative, read-only lookup. Never interpreted as a scan request.
    Verdict(&'a Path),
    /// Read-only summary of an absolute selection under the operator's fixed scope.
    Inspect(&'a Path),
    /// Explicitly verify the *previously selected* suspect, never a folder inventory.
    Verify(&'a Path, u64),
}
pub struct Reply {
    pub code: u8,
    pub body: String,
}
#[derive(Clone, Copy, Debug)]
pub struct FileVerdict {
    pub band: &'static str,
    pub bytes: u64,
}
pub struct Inventory {
    pub code: u8,
    pub report: String,
    /// False if enumeration was incomplete. An unknown tree cannot become current.
    pub enumerated: bool,
}
pub trait Scans: Send {
    fn inventory(&mut self, seen: &mut dyn FnMut(&Path, FileVerdict)) -> io::Result<Inventory>;
    fn changed(&mut self, relative: &Path) -> io::Result<FileVerdict>;
    fn forget(&mut self, relative: &Path);
    fn verify(&mut self, relative: &Path) -> io::Result<FileVerdict>;
}
fn invalid(s: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, s)
}
fn validate_name(name: &str) -> io::Result<()> {
    if !(1..=32).contains(&name.len())
        || !name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_'))
    {
        return Err(invalid(
            "instance: expected 1–32 ASCII letters, digits, '-' or '_'",
        ));
    }
    Ok(())
}
#[cfg(not(any(target_os = "linux", windows)))]
pub fn request(name: &str, _: Command<'_>) -> io::Result<Reply> {
    validate_name(name)?;
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "no local IPC adapter",
    ))
}
#[cfg(not(any(target_os = "linux", windows)))]
pub fn serve(name: &str, _: &Path, _: impl FnOnce(), _: impl Scans) -> io::Result<()> {
    request(name, Command::Status).map(|_| ())
}
