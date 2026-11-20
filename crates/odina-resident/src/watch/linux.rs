// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
use super::*;
use std::{
    collections::HashMap,
    ffi::{CString, OsStr},
    io::{Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::{ffi::OsStrExt, net::UnixStream},
    },
};

pub(crate) fn now_ns() -> u64 {
    // Same clock/time origin as Python time.monotonic_ns() on Linux.
    unsafe {
        let mut t = std::mem::zeroed();
        assert_eq!(libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut t), 0);
        t.tv_sec as u64 * 1_000_000_000 + t.tv_nsec as u64
    }
}
pub(crate) struct Cancel(UnixStream, UnixStream);
impl Cancel {
    pub fn new() -> io::Result<Self> {
        let (a, b) = UnixStream::pair()?;
        a.set_nonblocking(true)?;
        Ok(Self(a, b))
    }
    pub fn wake(&self) {
        let _ = (&self.0).write_all(&[1]);
    }
}
const MASK: u32 = libc::IN_CREATE
    | libc::IN_MODIFY
    | libc::IN_CLOSE_WRITE
    | libc::IN_ATTRIB
    | libc::IN_DELETE
    | libc::IN_MOVED_FROM
    | libc::IN_MOVED_TO
    | libc::IN_MOVE_SELF
    | libc::IN_DELETE_SELF
    | libc::IN_ONLYDIR
    | libc::IN_DONT_FOLLOW;
struct Stream {
    fd: fs::File,
    dirs: HashMap<i32, PathBuf>,
}
impl Stream {
    fn open() -> io::Result<Self> {
        let raw = unsafe { libc::inotify_init1(libc::IN_CLOEXEC) };
        if raw < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self {
            fd: unsafe { fs::File::from_raw_fd(raw) },
            dirs: HashMap::new(),
        })
    }
    fn add(
        &mut self,
        scope: &Scope<'_>,
        start: &Path,
        catch_up: Option<&Shared>,
    ) -> io::Result<()> {
        tree(scope, start, catch_up, |rel| {
            let dir = CString::new(scope.base.join(rel).as_os_str().as_bytes())?;
            let wd = unsafe { libc::inotify_add_watch(self.fd.as_raw_fd(), dir.as_ptr(), MASK) };
            if wd < 0 {
                return Err(io::Error::last_os_error());
            }
            if let Some(old) = self.dirs.get(&wd) {
                if old != rel {
                    return Err(invalid("ambiguous directory identity"));
                }
                return Ok(false);
            }
            if self.dirs.len() == DIR_LIMIT {
                return Err(invalid("watch directory limit"));
            }
            self.dirs.insert(wd, rel.to_owned());
            Ok(true)
        })
    }
    fn listen(&mut self, scope: &Scope<'_>, shared: &Shared, cancel: &Cancel) -> io::Result<End> {
        let mut buf = [0u8; 64 * 1024];
        let mut ticket = None;
        loop {
            let mut fds = [self.fd.as_raw_fd(), cancel.1.as_raw_fd()].map(|fd| libc::pollfd {
                fd,
                events: libc::POLLIN,
                revents: 0,
            });
            let n =
                unsafe { libc::poll(fds.as_mut_ptr(), 2, if ticket.is_some() { 0 } else { -1 }) };
            if n == 0 {
                shared.drained(ticket.take().unwrap());
                continue;
            }
            if n < 0 {
                let e = io::Error::last_os_error();
                if e.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(e);
            }
            if fds[1].revents != 0 {
                let _ = (&cancel.1).read(&mut [0; 64])?;
                if shared.state.lock().unwrap().stopping {
                    return Ok(End::Stop);
                }
                ticket = Some(shared.ticket());
                continue;
            }
            let n = self.fd.read(&mut buf)?;
            let received = now_ns();
            if n == 0 {
                return Err(io::Error::other("watch stream ended"));
            }
            let mut batch = &buf[..n];
            while !batch.is_empty() {
                let header = std::mem::size_of::<libc::inotify_event>();
                if batch.len() < header {
                    return Ok(End::Gap);
                }
                // SAFETY: bounds checked; buffer alignment is not assumed.
                let e = unsafe {
                    std::ptr::read_unaligned(batch.as_ptr().cast::<libc::inotify_event>())
                };
                let Some(field) = batch.get(header..header + e.len as usize) else {
                    return Ok(End::Gap);
                };
                batch = &batch[header + field.len()..];
                if e.mask & libc::IN_Q_OVERFLOW != 0 {
                    return Ok(End::Gap);
                }
                let Some(dir) = self.dirs.get(&e.wd) else {
                    return Ok(End::Gap);
                };
                if e.mask
                    & (libc::IN_IGNORED
                        | libc::IN_DELETE_SELF
                        | libc::IN_MOVE_SELF
                        | libc::IN_UNMOUNT)
                    != 0
                {
                    if dir.as_os_str().is_empty() {
                        return Err(io::Error::other("watched root detached"));
                    }
                    return Ok(End::Gap);
                }
                if field.is_empty() {
                    continue;
                } // attribute change on the directory itself
                let name = OsStr::from_bytes(field.split(|c| *c == 0).next().unwrap());
                let rel = dir.join(name);
                if !relative(&rel) {
                    return Ok(End::Gap);
                }
                if !scope.accepts(&rel) {
                    continue;
                }
                let remove = e.mask & (libc::IN_DELETE | libc::IN_MOVED_FROM) != 0;
                if e.mask & libc::IN_ISDIR != 0 {
                    if scope.only.is_some() {
                        shared.event(&rel, true, received);
                        continue;
                    }
                    if remove {
                        return Ok(End::Gap);
                    } // old descendant path mappings are ambiguous
                    if e.mask & (libc::IN_CREATE | libc::IN_MOVED_TO) != 0 {
                        self.add(scope, &rel, Some(shared))?;
                    }
                } else {
                    shared.event(&rel, remove, received);
                }
            }
        }
    }
}
pub(crate) fn run(root: &Path, single: bool, shared: &Shared, cancel: &Cancel) -> io::Result<()> {
    let scope = Scope::new(root, single)?;
    loop {
        let mut stream = Stream::open()?;
        stream.add(&scope, Path::new(""), None)?;
        shared.arm();
        match stream.listen(&scope, shared, cancel)? {
            End::Stop => return Ok(()),
            End::Gap => shared.loss(),
        }
    }
}
