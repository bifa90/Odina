// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K25 IPC Framing & Size-Limit Contract.

pub const IPC_VERSION: u8 = 1;
pub const MAX_IPC_FRAME_BYTES: usize = 64 * 1024;

pub fn validate_ipc_header(version: u8, payload_len: usize) -> bool {
    version == IPC_VERSION && payload_len <= MAX_IPC_FRAME_BYTES
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_bad_version_or_oversized_frame() {
        assert!(validate_ipc_header(1, 1024));
        assert!(!validate_ipc_header(2, 1024));
        assert!(!validate_ipc_header(1, MAX_IPC_FRAME_BYTES + 1));
    }
}
