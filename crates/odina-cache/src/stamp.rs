// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! Handle metadata is an invalidation hint, not an attestation of content.
//! Even a matching stamp cannot authorize a hit on a candidate length.

use std::fs::File;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct Stamp {
    pub(super) len: u64,
    identity: [u64; 3],
    times: [i64; 4],
}

#[cfg(target_os = "linux")]
pub(super) fn stamp(file: &File) -> Option<Stamp> {
    use std::{mem::MaybeUninit, os::unix::io::AsRawFd};
    let mut fs = MaybeUninit::<libc::statfs>::uninit();
    // SAFETY: live fd and space for the platform's statfs, read only on success.
    let fs = unsafe {
        if libc::fstatfs(file.as_raw_fd(), fs.as_mut_ptr()) != 0 {
            return None;
        }
        fs.assume_init()
    };
    // Native ordinary data files. Unknown types (including proc/sysfs, FUSE
    // and network mounts) stream every time; st_size there need not be EOF.
    if !matches!(
        fs.f_type,
        libc::EXT4_SUPER_MAGIC
            | libc::BTRFS_SUPER_MAGIC
            | libc::XFS_SUPER_MAGIC
            | libc::TMPFS_MAGIC
            | libc::MSDOS_SUPER_MAGIC
            | 0x2011_bab0 // Linux EXFAT_SUPER_MAGIC
            | 0x7366_746e // kernel ntfs3, not ntfs-3g/FUSE
    ) {
        return None;
    }
    state(file)
}
#[cfg(target_os = "linux")]
pub(super) fn state(file: &File) -> Option<Stamp> {
    use std::os::unix::fs::MetadataExt;
    let m = file.metadata().ok()?;
    m.is_file().then(|| Stamp {
        len: m.len(),
        identity: [m.dev(), m.ino(), 0],
        times: [m.mtime(), m.mtime_nsec(), m.ctime(), m.ctime_nsec()],
    })
}

#[cfg(windows)]
pub(super) fn stamp(file: &File) -> Option<Stamp> {
    windows_state(file, true)
}
#[cfg(windows)]
pub(super) fn state(file: &File) -> Option<Stamp> {
    windows_state(file, false)
}
#[cfg(windows)]
fn windows_state(file: &File, require_cacheable_volume: bool) -> Option<Stamp> {
    use std::{
        ffi::c_void,
        mem::{size_of, MaybeUninit},
        os::windows::io::AsRawHandle,
        ptr::null_mut,
    };
    #[repr(C)]
    struct Basic {
        created: i64,
        accessed: i64,
        written: i64,
        changed: i64,
        attributes: u32,
    }
    #[repr(C)]
    struct Id {
        volume: u64,
        id: [u8; 16],
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GetFileInformationByHandleEx(
            h: *mut c_void,
            class: i32,
            out: *mut c_void,
            len: u32,
        ) -> i32;
        fn GetVolumeInformationByHandleW(
            h: *mut c_void,
            name: *mut u16,
            n: u32,
            serial: *mut u32,
            max: *mut u32,
            flags: *mut u32,
            fs: *mut u16,
            len: u32,
        ) -> i32;
        fn GetFinalPathNameByHandleW(h: *mut c_void, out: *mut u16, len: u32, flags: u32) -> u32;
    }
    let h = file.as_raw_handle();
    let (mut basic, mut id) = (MaybeUninit::<Basic>::uninit(), MaybeUninit::<Id>::uninit());
    let mut fs = [0u16; 32];
    // SAFETY: all sizes match the two documented Win32 structures. Null
    // volume fields are optional. A local volume GUID is required (not UNC).
    let (basic, id) = unsafe {
        if (require_cacheable_volume
            && (GetFinalPathNameByHandleW(h, null_mut(), 0, 1) == 0
                || GetVolumeInformationByHandleW(
                    h,
                    null_mut(),
                    0,
                    null_mut(),
                    null_mut(),
                    null_mut(),
                    fs.as_mut_ptr(),
                    fs.len() as u32,
                ) == 0))
            || GetFileInformationByHandleEx(
                h,
                0,
                basic.as_mut_ptr().cast(),
                size_of::<Basic>() as u32,
            ) == 0
            || GetFileInformationByHandleEx(h, 18, id.as_mut_ptr().cast(), size_of::<Id>() as u32)
                == 0
        {
            return None;
        }
        (basic.assume_init(), id.assume_init())
    };
    let m = file.metadata().ok()?;
    let fs = &fs[..fs.iter().position(|&c| c == 0)?];
    if !m.is_file()
        || basic.attributes & 0x400 != 0
        || (require_cacheable_volume
            && !(fs.iter().copied().eq("NTFS".encode_utf16())
                || fs.iter().copied().eq("ReFS".encode_utf16())))
    {
        return None;
    }
    Some(Stamp {
        len: m.len(),
        identity: [
            id.volume,
            u64::from_le_bytes(id.id[..8].try_into().ok()?),
            u64::from_le_bytes(id.id[8..].try_into().ok()?),
        ],
        times: [basic.written, basic.changed, basic.created, 0],
    })
}

#[cfg(not(any(target_os = "linux", windows)))]
pub(super) fn stamp(_: &File) -> Option<Stamp> {
    None
}

#[cfg(not(any(target_os = "linux", windows)))]
pub(super) fn state(_: &File) -> Option<Stamp> {
    None
}
