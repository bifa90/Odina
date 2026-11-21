// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K19 Archive Bomb & Depth/Ratio Guard Contract.

pub const MAX_ARCHIVE_DEPTH: u8 = 8;
pub const MAX_EXPANSION_RATIO: u64 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveVerdict {
    SafeToInspect,
    AbortedBombOrDepth(&'static str),
}

pub fn check_archive_limits(depth: u8, compressed_bytes: u64, uncompressed_bytes: u64) -> ArchiveVerdict {
    if depth > MAX_ARCHIVE_DEPTH {
        return ArchiveVerdict::AbortedBombOrDepth("max_depth_exceeded");
    }
    if compressed_bytes == 0 && uncompressed_bytes > 0 {
        return ArchiveVerdict::AbortedBombOrDepth("zero_compressed_ratio");
    }
    if compressed_bytes > 0 && (uncompressed_bytes / compressed_bytes) > MAX_EXPANSION_RATIO {
        return ArchiveVerdict::AbortedBombOrDepth("expansion_ratio_exceeded");
    }
    ArchiveVerdict::SafeToInspect
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aborts_deep_or_high_ratio_archives() {
        assert_eq!(
            check_archive_limits(9, 100, 200),
            ArchiveVerdict::AbortedBombOrDepth("max_depth_exceeded")
        );
        assert_eq!(
            check_archive_limits(2, 10, 50_000),
            ArchiveVerdict::AbortedBombOrDepth("expansion_ratio_exceeded")
        );
        assert_eq!(check_archive_limits(2, 1000, 5000), ArchiveVerdict::SafeToInspect);
    }
}
