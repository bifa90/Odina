// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! Local per-user control plane. The operator selects the scope at startup.
//! Nothing here knows how to score a file, start itself, or open a TCP socket.
use std::{io, path::Path};

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod native;
#[cfg(windows)]
#[path = "windows.rs"]
mod native;
#[cfg(any(target_os = "linux", windows))]
mod server;
#[cfg(any(target_os = "linux", windows))]
mod wire;
#[cfg(any(target_os = "linux", windows))]
pub use server::{request, serve};

pub const MAX_REPLY: usize = 8 * 1024 * 1024;
pub enum Command<'a> {
    Status,
    ScanNow,
    /// Already canonical on the client. Compared as opaque bytes, never opened by the server.
    Check(&'a Path),
    Stop,
}
pub struct Reply {
    pub code: u8,
    pub body: String,
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
pub fn serve(
    name: &str,
    _: &Path,
    _: impl FnOnce(),
    _: impl FnMut() -> io::Result<(u8, String)> + Send,
) -> io::Result<()> {
    request(name, Command::Status).map(|_| ())
}
