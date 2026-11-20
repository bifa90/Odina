// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! Local browser selections. Mapping a name to a published key does NO filesystem I/O.
//! In particular, do not canonicalize a path supplied by a web client (UNC/redirectors).
#[cfg(any(target_os = "linux", windows))]
use std::path::Path;
use std::path::PathBuf;

/// Ordinary absolute UTF-8 path; never a URL, UNC share, device, alternate stream or traversal.
pub fn local_selection(text: &str) -> Option<PathBuf> {
    if text.is_empty() || text.len() > 16 * 1024 || text.chars().any(char::is_control) {
        return None;
    }
    #[cfg(windows)]
    let text = text.replace('\\', "/");
    #[cfg(windows)]
    let tail = {
        let b = text.as_bytes();
        if !b.first()?.is_ascii_alphabetic() || b.get(1..3) != Some(b":/") {
            return None;
        }
        &text[3..]
    };
    #[cfg(not(windows))]
    let tail = {
        if text.contains('\\') {
            return None;
        }
        text.strip_prefix('/')?
    };
    if tail.starts_with('/') || tail.contains("//") {
        return None;
    }
    for part in tail
        .trim_end_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
    {
        if matches!(part, "." | "..") {
            return None;
        }
        #[cfg(windows)]
        {
            let stem = part
                .split('.')
                .next()?
                .trim_end_matches(' ')
                .to_ascii_uppercase();
            if part.contains([':', '*', '?', '"', '<', '>', '|'])
                || part.ends_with([' ', '.'])
                || matches!(
                    stem.as_str(),
                    "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
                )
                || ["COM", "LPT"].iter().any(|p| {
                    stem.strip_prefix(p).is_some_and(|n| {
                        matches!(
                            n,
                            "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
                        )
                    })
                })
            {
                return None;
            }
        }
    }
    Some(PathBuf::from(text))
}

#[cfg(any(target_os = "linux", windows))]
pub(crate) fn scope_key(root: &Path, single: bool, arg: &[u8]) -> Option<PathBuf> {
    let path = local_selection(std::str::from_utf8(arg).ok()?)?;
    // The service uses canonical verbatim disk paths on Windows, not user-supplied prefixes.
    #[cfg(windows)]
    let path = PathBuf::from(format!(r"\\?\{}", path.to_str()?.replace('/', r"\")));
    let rel = path.strip_prefix(root).ok()?;
    if single {
        return rel
            .as_os_str()
            .is_empty()
            .then(|| root.file_name().map(PathBuf::from))
            .flatten();
    }
    Some(rel.to_owned()) // empty = selected scope directory/drive root
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn remote_relative_and_dot_components_are_not_local_selections() {
        for bad in [
            "relative",
            "file:///tmp/a",
            "//host/share/a",
            r"\\host\share\a",
            r"\\?\C:\x",
            "/tmp/../x",
            "C:/a/../b",
            "C:/a:x",
            "C:/NUL.txt",
            "C:/x//y",
            "/tmp/x\0y",
        ] {
            assert!(local_selection(bad).is_none(), "{bad:?}");
        }
    }
}
