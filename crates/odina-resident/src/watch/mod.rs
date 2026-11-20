// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
use crate::{invalid, monitor::Shared};
use std::{
    fs, io,
    path::{Component, Path, PathBuf},
};
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub(crate) use linux::{now_ns, run, Cancel};
#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub(crate) use windows::{now_ns, run, Cancel};

#[cfg(windows)]
pub(crate) const BACKEND: &str = "ReadDirectoryChangesW";
#[cfg(target_os = "linux")]
pub(crate) const BACKEND: &str = "inotify";

pub(crate) fn relative(path: &Path) -> bool {
    !path.as_os_str().is_empty() && path.components().all(|p| matches!(p, Component::Normal(_)))
}
struct Scope<'a> {
    base: &'a Path,
    only: Option<&'a Path>,
}
impl<'a> Scope<'a> {
    fn new(root: &'a Path, single: bool) -> io::Result<Self> {
        if single {
            Ok(Self {
                base: root.parent().ok_or_else(|| invalid("no parent"))?,
                only: Some(Path::new(
                    root.file_name().ok_or_else(|| invalid("no file name"))?,
                )),
            })
        } else {
            Ok(Self {
                base: root,
                only: None,
            })
        }
    }
    fn accepts(&self, rel: &Path) -> bool {
        self.only.map_or(true, |p| rel == p)
    }
}
fn ordinary_dir(m: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if m.file_attributes() & 0x400 != 0 {
            return false;
        }
    }
    m.is_dir()
}
// A nested entry that vanished between readdir/stat/add_watch is normal churn;
// its parent is already subscribed. Never swallow permission or root errors.
fn still_there<T>(result: io::Result<T>, nested: bool) -> io::Result<Option<T>> {
    match result {
        Err(e) if nested && e.kind() == io::ErrorKind::NotFound => Ok(None),
        r => r.map(Some),
    }
}
// Install before enumeration. For a newly inserted subtree, feed existing regular files
// to the same per-file queue, closing the interval before the child watches were armed.
fn tree(
    scope: &Scope<'_>,
    start: &Path,
    catch_up: Option<&Shared>,
    mut install: impl FnMut(&Path) -> io::Result<bool>,
) -> io::Result<()> {
    let mut stack = vec![start.to_owned()];
    while let Some(rel) = stack.pop() {
        let dir = scope.base.join(&rel);
        let nested = !rel.as_os_str().is_empty();
        let Some(m) = still_there(fs::symlink_metadata(&dir), nested)? else {
            continue;
        };
        if !ordinary_dir(&m) {
            if nested {
                continue;
            }
            return Err(invalid("watch root changed type"));
        }
        let Some(canonical) = still_there(dir.canonicalize(), nested)? else {
            continue;
        };
        if canonical != dir {
            return Err(invalid("watch directory redirected"));
        }
        if still_there(install(&rel), nested)? != Some(true) || scope.only.is_some() {
            continue;
        }
        let Some(entries) = still_there(fs::read_dir(dir), nested)? else {
            continue;
        };
        for entry in entries {
            let Some(entry) = still_there(entry, true)? else {
                continue;
            };
            let Some(kind) = still_there(entry.file_type(), true)? else {
                continue;
            };
            let child = rel.join(entry.file_name());
            if kind.is_dir() {
                #[cfg(windows)]
                if !still_there(entry.metadata(), true)?
                    .as_ref()
                    .is_some_and(ordinary_dir)
                {
                    continue;
                }
                stack.push(child);
            } else if kind.is_file() {
                if let Some(shared) = catch_up {
                    shared.event(&child, false, now_ns());
                }
            }
        }
    }
    Ok(())
}
const DIR_LIMIT: usize = 200_000;
// A loss return requires a NEW kernel stream before reconciliation. A fatal error leaves
// the instance queryable but unknown; it never silently falls back to polling.
#[derive(PartialEq)]
enum End {
    Stop,
    Gap,
}
