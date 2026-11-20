// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
use std::{
    io::{self, Read, Write},
    mem::MaybeUninit,
    os::{
        fd::AsRawFd,
        linux::net::SocketAddrExt,
        unix::net::{SocketAddr, UnixListener, UnixStream},
    },
    time::Duration,
};

fn uid() -> libc::uid_t {
    // SAFETY: no arguments or pointers.
    unsafe { libc::geteuid() }
}
fn address(name: &str) -> io::Result<SocketAddr> {
    // No stale file on crash; the abstract name is NOT authentication.
    SocketAddr::from_abstract_name(format!("odina.v1.{}.{name}", uid()))
}
fn verified(s: UnixStream) -> io::Result<Peer> {
    let mut cred = MaybeUninit::<libc::ucred>::uninit();
    let size = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    let mut n = size;
    // SAFETY: live socket, output struct of the requested length. Verify both peers.
    unsafe {
        if libc::getsockopt(
            s.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            cred.as_mut_ptr().cast(),
            &mut n,
        ) != 0
        {
            return Err(io::Error::last_os_error());
        }
        if n != size || cred.assume_init().uid != uid() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "different peer UID",
            ));
        }
    }
    Ok(Peer(s))
}
pub(super) struct Listener(UnixListener);
impl Listener {
    pub fn bind(name: &str) -> io::Result<Self> {
        UnixListener::bind_addr(&address(name)?).map(Self)
    }
    pub fn accept(&mut self) -> io::Result<Peer> {
        verified(self.0.accept()?.0)
    }
}
pub(super) struct Peer(UnixStream);
impl Peer {
    pub fn connect(name: &str) -> io::Result<Self> {
        verified(UnixStream::connect_addr(&address(name)?)?)
    }
    pub fn recv(&mut self, buf: &mut [u8], timeout: Duration) -> io::Result<usize> {
        self.0.set_read_timeout(Some(timeout))?;
        self.0.read(buf)
    }
    pub fn send(&mut self, buf: &[u8], timeout: Duration) -> io::Result<usize> {
        self.0.set_write_timeout(Some(timeout))?;
        self.0.write(buf)
    }
}
