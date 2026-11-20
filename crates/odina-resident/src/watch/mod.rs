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
        let m = match fs::symlink_metadata(&dir) {
            Err(e) if !rel.as_os_str().is_empty() && e.kind() == io::ErrorKind::NotFound => {
                continue
            }
            r => r?,
        };
        if !ordinary_dir(&m) || dir.canonicalize()? != dir {
            return Err(invalid("watch directory changed or redirected"));
        }
        if !install(&rel)? || scope.only.is_some() {
            continue;
        }
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            let child = rel.join(entry.file_name());
            if kind.is_dir() {
                #[cfg(windows)]
                if !ordinary_dir(&entry.metadata()?) {
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
