// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! Local pipe, per-user DACL, mutual ownership check. No service account.
use std::{
    ffi::c_void,
    fs::OpenOptions,
    io,
    mem::{replace, size_of},
    os::windows::{
        fs::OpenOptionsExt,
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
    },
    ptr::{null, null_mut},
    time::Duration,
};
use windows_sys::Win32::{
    Foundation::{
        LocalFree, ERROR_IO_PENDING, ERROR_NO_DATA, ERROR_PIPE_BUSY, ERROR_PIPE_CONNECTED,
        ERROR_SUCCESS, HANDLE, INVALID_HANDLE_VALUE,
    },
    Security::{
        Authorization::{
            ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
            GetSecurityInfo, SDDL_REVISION_1, SE_KERNEL_OBJECT,
        },
        GetTokenInformation, TokenUser, OWNER_SECURITY_INFORMATION, PSID, SECURITY_ATTRIBUTES,
        TOKEN_QUERY, TOKEN_USER,
    },
    Storage::FileSystem::{
        ReadFile, WriteFile, FILE_FLAG_FIRST_PIPE_INSTANCE, FILE_FLAG_OVERLAPPED,
        PIPE_ACCESS_DUPLEX, SECURITY_IDENTIFICATION,
    },
    System::{
        Pipes::{ConnectNamedPipe, CreateNamedPipeW, WaitNamedPipeW, PIPE_REJECT_REMOTE_CLIENTS},
        Threading::{CreateEventW, GetCurrentProcess, OpenProcessToken, INFINITE},
        IO::{CancelIoEx, GetOverlappedResult, GetOverlappedResultEx, OVERLAPPED},
    },
};
fn ok(b: i32) -> io::Result<()> {
    if b != 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}
fn owned(h: HANDLE) -> io::Result<OwnedHandle> {
    if h.is_null() || h == INVALID_HANDLE_VALUE {
        Err(io::Error::last_os_error())
    }
    // SAFETY: callers transfer a newly acquired kernel handle exactly once.
    else {
        Ok(unsafe { OwnedHandle::from_raw_handle(h) })
    }
}
struct Local(*mut c_void);
impl Drop for Local {
    fn drop(&mut self) {
        unsafe {
            LocalFree(self.0);
        }
    }
}
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain([0]).collect()
}
/// SAFETY: SID is live for this call; the API owns the size of the resulting NUL-terminated text.
unsafe fn sid_text(sid: PSID) -> io::Result<String> {
    let mut p = null_mut();
    ok(ConvertSidToStringSidW(sid, &mut p))?;
    let _free = Local(p.cast());
    let mut n = 0;
    while *p.add(n) != 0 {
        n += 1;
    }
    String::from_utf16(std::slice::from_raw_parts(p, n)).map_err(|_| super::invalid("invalid SID"))
}
fn user() -> io::Result<String> {
    // SAFETY: token buffer is sized by the OS, aligned for TOKEN_USER, and live during conversion.
    unsafe {
        let mut t = null_mut();
        ok(OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut t))?;
        let token = owned(t)?;
        let mut bytes = 0;
        GetTokenInformation(token.as_raw_handle(), TokenUser, null_mut(), 0, &mut bytes);
        if (bytes as usize) < size_of::<TOKEN_USER>() {
            return Err(io::Error::last_os_error());
        }
        let mut buf = vec![0usize; (bytes as usize).div_ceil(size_of::<usize>())];
        ok(GetTokenInformation(
            token.as_raw_handle(),
            TokenUser,
            buf.as_mut_ptr().cast(),
            bytes,
            &mut bytes,
        ))?;
        sid_text((*buf.as_ptr().cast::<TOKEN_USER>()).User.Sid)
    }
}
fn path(name: &str, user: &str) -> String {
    format!(r"\\.\pipe\Odina.v1.{user}.{name}")
}
fn check_owner(h: &OwnedHandle, expected: &str) -> io::Result<()> {
    // SAFETY: GetSecurityInfo owns the descriptor, owner points inside it.
    unsafe {
        let (mut owner, mut sd) = (null_mut(), null_mut());
        let e = GetSecurityInfo(
            h.as_raw_handle(),
            SE_KERNEL_OBJECT,
            OWNER_SECURITY_INFORMATION,
            &mut owner,
            null_mut(),
            null_mut(),
            null_mut(),
            &mut sd,
        );
        if e != ERROR_SUCCESS {
            return Err(io::Error::from_raw_os_error(e as i32));
        }
        let _free = Local(sd);
        if owner.is_null() || sid_text(owner)? != expected {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "different pipe owner",
            ));
        }
    }
    Ok(())
}

/// Finish or cancel **and drain** every pending operation before returning the
/// caller's buffer or dropping OVERLAPPED/event. INFINITE is for accept only.
fn operation(
    h: &OwnedHandle,
    ms: u32,
    begin: impl FnOnce(HANDLE, *mut OVERLAPPED) -> i32,
) -> io::Result<usize> {
    unsafe {
        let event = owned(CreateEventW(null(), 1, 0, null()))?;
        let mut op = OVERLAPPED {
            hEvent: event.as_raw_handle(),
            ..Default::default()
        };
        let mut n = 0;
        let raw = h.as_raw_handle();
        if begin(raw, &mut op) == 0 {
            let e = io::Error::last_os_error();
            if e.raw_os_error() != Some(ERROR_IO_PENDING as i32) {
                return Err(e);
            }
            if GetOverlappedResultEx(raw, &op, &mut n, ms, 0) == 0 {
                let e = io::Error::last_os_error();
                CancelIoEx(raw, &op);
                GetOverlappedResult(raw, &op, &mut n, 1);
                return Err(e);
            }
        } else {
            ok(GetOverlappedResult(raw, &op, &mut n, 0))?;
        }
        Ok(n as usize)
    }
}
struct Address {
    name: Vec<u16>,
    sd: Local,
}
impl Address {
    fn create(&self, first: bool) -> io::Result<OwnedHandle> {
        let sa = SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: self.sd.0,
            bInheritHandle: 0,
        };
        // SAFETY: NUL-terminated local name and descriptor remain live; byte-mode pipe.
        owned(unsafe {
            CreateNamedPipeW(
                self.name.as_ptr(),
                PIPE_ACCESS_DUPLEX
                    | FILE_FLAG_OVERLAPPED
                    | if first {
                        FILE_FLAG_FIRST_PIPE_INSTANCE
                    } else {
                        0
                    },
                PIPE_REJECT_REMOTE_CLIENTS,
                4,
                65536,
                4096,
                3000,
                &sa,
            )
        })
    }
}
pub(super) struct Listener {
    addr: Address,
    next: OwnedHandle,
}
impl Listener {
    pub fn bind(name: &str) -> io::Result<Self> {
        let user = user()?;
        let mut sd = null_mut();
        let sddl = wide(&format!("O:{user}D:P(A;;GA;;;{user})"));
        // SAFETY: allocate a protected DACL for precisely this token user; no World ACE.
        unsafe {
            ok(ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl.as_ptr(),
                SDDL_REVISION_1,
                &mut sd,
                null_mut(),
            ))?;
        }
        let addr = Address {
            name: wide(&path(name, &user)),
            sd: Local(sd),
        };
        let next = addr.create(true)?;
        Ok(Self { addr, next })
    }
    pub fn accept(&mut self) -> io::Result<Peer> {
        let result = operation(&self.next, INFINITE, |h, o| unsafe {
            ConnectNamedPipe(h, o)
        });
        // Keep the name continuously occupied, including while a worker replies.
        let connected = replace(&mut self.next, self.addr.create(false)?);
        match result {
            Ok(_) => Ok(Peer(connected)),
            Err(e) if e.raw_os_error() == Some(ERROR_PIPE_CONNECTED as i32) => Ok(Peer(connected)),
            Err(e) if e.raw_os_error() == Some(ERROR_NO_DATA as i32) => {
                Err(io::ErrorKind::Interrupted.into())
            }
            Err(e) => Err(e),
        }
    }
}
pub(super) struct Peer(OwnedHandle);
impl Peer {
    pub fn connect(name: &str) -> io::Result<Self> {
        let sid = user()?;
        let p = path(name, &sid);
        // SQOS limits any impersonation by a squatter, even before checking owner.
        let open = || {
            OpenOptions::new()
                .read(true)
                .write(true)
                .custom_flags(FILE_FLAG_OVERLAPPED)
                .security_qos_flags(SECURITY_IDENTIFICATION)
                .open(&p)
        };
        let file = match open() {
            Err(e) if e.raw_os_error() == Some(ERROR_PIPE_BUSY as i32) => {
                unsafe {
                    ok(WaitNamedPipeW(wide(&p).as_ptr(), 3000))?;
                }
                open()?
            }
            r => r?,
        };
        let h = file.into();
        check_owner(&h, &sid)?;
        Ok(Self(h))
    }
    pub fn recv(&mut self, b: &mut [u8], t: Duration) -> io::Result<usize> {
        operation(&self.0, t.as_millis().max(1) as u32, |h, o| unsafe {
            ReadFile(h, b.as_mut_ptr(), b.len() as u32, null_mut(), o)
        })
    }
    pub fn send(&mut self, b: &[u8], t: Duration) -> io::Result<usize> {
        operation(&self.0, t.as_millis().max(1) as u32, |h, o| unsafe {
            WriteFile(h, b.as_ptr(), b.len() as u32, null_mut(), o)
        })
    }
}
