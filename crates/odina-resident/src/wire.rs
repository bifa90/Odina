// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
use super::{invalid, native::Peer, Reply, MAX_REPLY};
use std::{
    io::{self, Read, Write},
    time::{Duration, Instant},
};

pub const LIMIT: usize = 64 * 1024; // canonical scope bytes; other requests have no body
pub const SHORT: Duration = Duration::from_secs(3);
pub const SCAN: Duration = Duration::from_secs(3600);
pub struct Channel {
    peer: Peer,
    deadline: Instant,
}
impl Channel {
    pub fn new(peer: Peer) -> Self {
        Self {
            peer,
            deadline: Instant::now() + SHORT,
        }
    }
    pub fn timeout(&mut self, time: Duration) {
        self.deadline = Instant::now() + time;
    }
    fn remaining(&self) -> io::Result<Duration> {
        self.deadline
            .checked_duration_since(Instant::now())
            .filter(|d| !d.is_zero())
            .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "frame deadline"))
    }
    pub fn reply(&mut self, r: Reply) -> io::Result<()> {
        self.timeout(SHORT);
        write_frame(self, r.code, r.body.as_bytes(), MAX_REPLY)?;
        // The pipe server must not disconnect before delivery, nor block forever
        // in FlushFileBuffers on a dead client. One bounded receipt on both OSes.
        let mut ack = [0];
        self.read_exact(&mut ack)?;
        if ack != [6] {
            return Err(invalid("bad receipt"));
        }
        Ok(())
    }
}
impl Read for Channel {
    fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
        self.peer.recv(b, self.remaining()?)
    }
}
impl Write for Channel {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        self.peer.send(b, self.remaining()?)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
pub fn read_frame(r: &mut impl Read, limit: usize) -> io::Result<(u8, Vec<u8>)> {
    let mut h = [0; 9];
    r.read_exact(&mut h)?;
    let len = u32::from_be_bytes(h[5..].try_into().unwrap()) as usize;
    if &h[..4] != b"ODN1" || len > limit {
        return Err(invalid("bad version or frame length"));
    }
    let mut body = vec![0; len];
    r.read_exact(&mut body)?;
    Ok((h[4], body))
}
pub fn write_frame(w: &mut impl Write, tag: u8, body: &[u8], limit: usize) -> io::Result<()> {
    if body.len() > limit {
        return Err(invalid("frame too large"));
    }
    let mut h = *b"ODN1\0\0\0\0\0";
    h[4] = tag;
    h[5..].copy_from_slice(&(body.len() as u32).to_be_bytes());
    w.write_all(&h)?;
    w.write_all(body)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_bad_version_truncation_and_oversize_before_payload() {
        for b in [
            &b"ODN2\x01\0\0\0\0"[..],
            b"ODN1\x01\0\0\0\x04a",
            b"ODN1\x01\xff\xff\xff\xff",
        ] {
            assert!(read_frame(&mut &b[..], LIMIT).is_err());
        }
    }
}
