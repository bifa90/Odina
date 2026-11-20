// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
use odina_resident::{local_selection, request, Command};
use serde::{Deserialize, Serialize};
use std::{
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

const LIMIT: usize = 32 * 1024;
// No message, page, environment variable or filename gets to choose a service/command.
const SERVICE: &str = "browser";
#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "lowercase", deny_unknown_fields)]
enum Input {
    Inspect { url: String },
    Deep { url: String, ticket: String },
}
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum ResultKind {
    Clean,
    Unknown,
    Suspicious,
    Threat,
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
struct Outcome {
    result: ResultKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    ticket: Option<String>,
}
impl Outcome {
    fn unknown() -> Self {
        Self {
            result: ResultKind::Unknown,
            ticket: None,
        }
    }
}
fn receipt(s: &str) -> Option<u64> {
    if s.len() != 16 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    u64::from_str_radix(s, 16).ok()
}

/// Deliberately NOT a general URL resolver. Decode each escape exactly once;
/// don't consult the filesystem, DNS, a download history or the browser cache.
fn file_url(url: &str) -> Option<PathBuf> {
    let raw = url.strip_prefix("file://")?;
    if !raw.starts_with('/')
        || raw.starts_with("//")
        || raw.contains(['?', '#', '\\'])
        || raw.len() > LIMIT
    {
        return None;
    }
    let mut data = Vec::with_capacity(raw.len());
    let mut it = raw.bytes();
    while let Some(c) = it.next() {
        if c != b'%' {
            data.push(c);
            continue;
        }
        let hi = (it.next()? as char).to_digit(16)?;
        let lo = (it.next()? as char).to_digit(16)?;
        let c = (hi * 16 + lo) as u8;
        if matches!(c, b'/' | b'\\') {
            return None;
        }
        data.push(c);
    }
    let path = std::str::from_utf8(&data).ok()?;
    #[cfg(windows)]
    let path = path.strip_prefix('/')?;
    local_selection(path)
}

fn query(path: &Path, ticket: Option<u64>) -> Outcome {
    let op = ticket.map_or(Command::Inspect(path), |t| Command::Verify(path, t));
    let Some(out) = request(SERVICE, op)
        .ok()
        .filter(|r| r.code == 0 && r.body.len() <= 128)
        .and_then(|r| serde_json::from_str::<Outcome>(&r.body).ok())
    else {
        return Outcome::unknown();
    };
    match (&out.result, &out.ticket, ticket) {
        (ResultKind::Suspicious, Some(t), None) if receipt(t).is_some() => out,
        (ResultKind::Suspicious, None, Some(_)) => out,
        (ResultKind::Clean | ResultKind::Threat | ResultKind::Unknown, None, _) => out,
        _ => Outcome::unknown(),
    }
}
fn confirm(path: &Path, ticket: &str) -> Outcome {
    receipt(ticket).map_or_else(Outcome::unknown, |t| query(path, Some(t)))
}
fn handle(bytes: &[u8]) -> Outcome {
    match serde_json::from_slice(bytes) {
        Ok(Input::Inspect { url }) => {
            file_url(&url).map_or_else(Outcome::unknown, |p| query(&p, None))
        }
        Ok(Input::Deep { url, ticket }) => {
            file_url(&url).map_or_else(Outcome::unknown, |p| confirm(&p, &ticket))
        }
        Err(_) => Outcome::unknown(),
    }
}
/// sendNativeMessage: one process per request, u32 native endian + UTF-8 JSON on both pipes.
/// Validate length BEFORE allocation. Stdout has no banners, logs, filenames or trailing newline.
pub fn once(input: &mut impl Read, output: &mut impl Write) -> io::Result<()> {
    let mut len = [0; 4];
    input.read_exact(&mut len)?;
    let n = u32::from_ne_bytes(len) as usize;
    let result = if n == 0 || n > LIMIT {
        Outcome::unknown()
    } else {
        let mut bytes = vec![0; n];
        input.read_exact(&mut bytes)?;
        handle(&bytes)
    };
    let reply = serde_json::to_vec(&result)?;
    output.write_all(&(reply.len() as u32).to_ne_bytes())?;
    output.write_all(&reply)?;
    output.flush()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn remote_malformed_and_ambiguous_urls_never_become_paths() {
        for u in [
            "https://example.invalid/x",
            "file://server/share/a",
            "file:////server/share/a",
            "file:///C:/a%2fb",
            "file:///tmp/a%5Cb",
            "file:///tmp/../x",
            "file:///C:/a/%2e%2e/b",
            "file:///C:/a%00b",
            "file:///tmp/a?secret",
            "file:///tmp/#fragment",
            "file:///bad%",
            "file:///bad%ff",
            "data:,a",
        ] {
            assert!(file_url(u).is_none(), "{u}");
        }
        let u = if cfg!(windows) {
            "file:///C:/x/%D8%A8%20a%2520"
        } else {
            "file:///x/%D8%A8%20a%2520"
        };
        assert_eq!(file_url(u).unwrap().file_name().unwrap(), "ب a%20");
    }
    #[test]
    fn reject_invalid_json_extra_fields_and_oversize_before_body() {
        for data in [
            &b"{"[..],
            br#"{"op":"stop"}"#,
            br#"{"op":"inspect","url":"file:///x","service":"other"}"#,
            br#"{"op":"inspect","url":"x","url":"y"}"#,
        ] {
            assert!(serde_json::from_slice::<Input>(data).is_err());
        }
        let mut out = Vec::new();
        once(&mut &u32::MAX.to_ne_bytes()[..], &mut out).unwrap();
        assert_eq!(
            u32::from_ne_bytes(out[..4].try_into().unwrap()) as usize,
            out.len() - 4
        );
        assert_eq!(&out[4..], br#"{"result":"unknown"}"#);
        assert!(once(&mut &b"\x07\0\0\0{"[..], &mut out).is_err());
    }
}
