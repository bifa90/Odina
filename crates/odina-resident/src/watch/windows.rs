// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
use super::*;
use std::{
    collections::HashSet,
    ffi::OsString,
    mem::size_of_val,
    os::windows::{
        ffi::OsStringExt,
        fs::OpenOptionsExt,
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
    },
    ptr::{null, null_mut},
    sync::OnceLock,
};
use windows_sys::Win32::{
    Foundation::{ERROR_IO_PENDING, ERROR_NOTIFY_ENUM_DIR, WAIT_OBJECT_0, WAIT_TIMEOUT},
    Storage::FileSystem::*,
    System::{
        Performance::{QueryPerformanceCounter, QueryPerformanceFrequency},
        Threading::{CreateEventW, ResetEvent, SetEvent, WaitForMultipleObjects, INFINITE},
        IO::{CancelIoEx, GetOverlappedResult, OVERLAPPED},
    },
};

pub(crate) fn now_ns() -> u64 {
    static FREQ: OnceLock<u64> = OnceLock::new();
    let freq = FREQ.get_or_init(|| unsafe {
        let mut f = 0;
        assert_ne!(QueryPerformanceFrequency(&mut f), 0);
        f as u64
    });
    let mut counter = 0;
    unsafe {
        assert_ne!(QueryPerformanceCounter(&mut counter), 0);
    }
    (counter as u128 * 1_000_000_000 / *freq as u128) as u64
}
fn event(manual: bool) -> io::Result<OwnedHandle> {
    let h = unsafe { CreateEventW(null(), manual.into(), 0, null()) };
    if h.is_null() {
        return Err(io::Error::last_os_error());
    }
    Ok(unsafe { OwnedHandle::from_raw_handle(h) })
}
pub(crate) struct Cancel(OwnedHandle);
impl Cancel {
    pub fn new() -> io::Result<Self> {
        event(false).map(Self)
    }
    pub fn wake(&self) {
        unsafe {
            SetEvent(self.0.as_raw_handle());
        }
    }
}
// The OVERLAPPED and DWORD-aligned data have stable addresses. On EVERY exit,
// including installation errors and cancellation, drain before freeing either.
struct Subscription {
    dir: fs::File,
    ready: OwnedHandle,
    op: Box<OVERLAPPED>,
    words: Box<[u32]>,
    pending: bool,
}
impl Drop for Subscription {
    fn drop(&mut self) {
        if self.pending {
            unsafe {
                CancelIoEx(self.dir.as_raw_handle(), &*self.op);
                GetOverlappedResult(self.dir.as_raw_handle(), &*self.op, &mut 0, 1);
            }
        }
    }
}
impl Subscription {
    fn open(path: &Path) -> io::Result<Self> {
        // Pin the subscription root against rename/removal while the handle lives.
        let dir = fs::OpenOptions::new()
            .access_mode(FILE_LIST_DIRECTORY)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(
                FILE_FLAG_OVERLAPPED | FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
            )
            .open(path)?;
        if !ordinary_dir(&dir.metadata()?) {
            return Err(invalid("not an ordinary watch directory"));
        }
        Ok(Self {
            dir,
            ready: event(true)?,
            op: Box::default(),
            words: vec![0; 16 * 1024].into_boxed_slice(),
            pending: false,
        })
    }
    fn arm(&mut self, recursive: bool) -> io::Result<()> {
        unsafe {
            if ResetEvent(self.ready.as_raw_handle()) == 0 {
                return Err(io::Error::last_os_error());
            }
            *self.op = OVERLAPPED {
                hEvent: self.ready.as_raw_handle(),
                ..Default::default()
            };
            if ReadDirectoryChangesW(
                self.dir.as_raw_handle(),
                self.words.as_mut_ptr().cast(),
                size_of_val(&*self.words) as u32,
                recursive.into(),
                FILE_NOTIFY_CHANGE_FILE_NAME
                    | FILE_NOTIFY_CHANGE_DIR_NAME
                    | FILE_NOTIFY_CHANGE_SIZE
                    | FILE_NOTIFY_CHANGE_LAST_WRITE
                    | FILE_NOTIFY_CHANGE_ATTRIBUTES,
                null_mut(),
                &mut *self.op,
                None,
            ) == 0
            {
                let err = io::Error::last_os_error();
                if err.raw_os_error() != Some(ERROR_IO_PENDING as i32) {
                    return Err(err);
                }
            }
        }
        self.pending = true;
        Ok(())
    }
    fn listen(&mut self, scope: &Scope<'_>, shared: &Shared, cancel: &Cancel) -> io::Result<End> {
        let mut dirs = HashSet::<PathBuf>::new();
        let recursive = scope.only.is_none();
        self.arm(recursive)?; // before enumeration: kernel buffers all subsequent changes
        let add = |dirs: &mut HashSet<PathBuf>, rel: &Path| {
            if dirs.len() >= DIR_LIMIT {
                return Err(invalid("watch directory limit"));
            }
            Ok(dirs.insert(rel.to_owned()))
        };
        tree(scope, Path::new(""), None, |rel| add(&mut dirs, rel))?;
        shared.arm();
        let handles = [cancel.0.as_raw_handle(), self.ready.as_raw_handle()];
        let mut saved = Vec::new();
        let mut ticket = None;
        loop {
            let rc = unsafe {
                WaitForMultipleObjects(
                    2,
                    handles.as_ptr(),
                    0,
                    if ticket.is_some() { 0 } else { INFINITE },
                )
            };
            if rc == WAIT_TIMEOUT {
                shared.drained(ticket.take().unwrap());
                continue;
            }
            if rc == WAIT_OBJECT_0 {
                if shared.state.lock().unwrap().stopping {
                    return Ok(End::Stop);
                }
                ticket = Some(shared.ticket());
                continue;
            }
            if rc != WAIT_OBJECT_0 + 1 {
                return Err(io::Error::last_os_error());
            }
            let received = now_ns();
            let mut n = 0;
            let ok = unsafe { GetOverlappedResult(self.dir.as_raw_handle(), &*self.op, &mut n, 1) };
            self.pending = false;
            if ok == 0 {
                let e = io::Error::last_os_error();
                if e.raw_os_error() == Some(ERROR_NOTIFY_ENUM_DIR as i32) {
                    return Ok(End::Gap);
                }
                return Err(e);
            }
            if n == 0 || n as usize > size_of_val(&*self.words) {
                return Ok(End::Gap);
            }
            saved.clear();
            // SAFETY: op completed and reported n initialized bytes, bounded above.
            saved.extend_from_slice(unsafe {
                std::slice::from_raw_parts(self.words.as_ptr().cast::<u8>(), n as usize)
            });
            self.arm(recursive)?; // do not leave the stream unarmed during parsing/subtree catch-up
            let mut batch = &saved[..];
            loop {
                let Some((next, action, rel)) = record(batch) else {
                    return Ok(End::Gap);
                };
                if scope.accepts(&rel) {
                    match action {
                        FILE_ACTION_REMOVED | FILE_ACTION_RENAMED_OLD_NAME => {
                            if dirs.contains(&rel) {
                                return Ok(End::Gap);
                            }
                            shared.event(&rel, true, received);
                        }
                        FILE_ACTION_ADDED | FILE_ACTION_RENAMED_NEW_NAME | FILE_ACTION_MODIFIED => {
                            match fs::symlink_metadata(scope.base.join(&rel)) {
                                Ok(m) if ordinary_dir(&m) => {
                                    if scope.only.is_some() {
                                        shared.event(&rel, true, received);
                                    } else if !dirs.contains(&rel) {
                                        tree(scope, &rel, Some(shared), |r| add(&mut dirs, r))?;
                                    }
                                }
                                _ => shared.event(&rel, false, received),
                            }
                        }
                        _ => return Ok(End::Gap),
                    }
                }
                if next == 0 {
                    break;
                }
                batch = &batch[next..];
            }
        }
    }
}
fn record(b: &[u8]) -> Option<(usize, u32, PathBuf)> {
    let dword = |i| Some(u32::from_le_bytes(b.get(i..i + 4)?.try_into().ok()?));
    let next = dword(0)? as usize;
    let len = dword(8)? as usize;
    let end = 12usize.checked_add(len)?;
    if len % 2 != 0 || (next != 0 && (next < end || next % 4 != 0 || next >= b.len())) {
        return None;
    }
    let name: Vec<u16> = b
        .get(12..end)?
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    if name.contains(&0) {
        return None;
    }
    let path = PathBuf::from(OsString::from_wide(&name));
    relative(&path).then_some((next, dword(4)?, path))
}
pub(crate) fn run(root: &Path, single: bool, shared: &Shared, cancel: &Cancel) -> io::Result<()> {
    let scope = Scope::new(root, single)?;
    loop {
        let mut stream = Subscription::open(scope.base)?;
        match stream.listen(&scope, shared, cancel) {
            Ok(End::Stop) => return Ok(()),
            Ok(End::Gap) => shared.loss(),
            Err(e) if e.raw_os_error() == Some(ERROR_NOTIFY_ENUM_DIR as i32) => shared.loss(),
            Err(e) => return Err(e),
        }
    }
}
