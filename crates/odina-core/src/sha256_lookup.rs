// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! Layer 3 (0-SAR): Free browser-only SHA-256 lookup on explicit user request.
//! Never uploads files, never queries automatically, and never sends local paths.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LookupError {
    ConsentRequired,
    InvalidSha256,
}

/// Validates a 64-char hex SHA-256 digest and builds a free VirusTotal GUI URL
/// only when `user_requested` is true.
pub fn build_free_sha256_lookup_url(
    sha256_hex: &str,
    user_requested: bool,
) -> Result<String, LookupError> {
    if !user_requested {
        return Err(LookupError::ConsentRequired);
    }
    let normalized = sha256_hex.trim();
    if normalized.len() != 64 || !normalized.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(LookupError::InvalidSha256);
    }
    Ok(format!(
        "https://www.virustotal.com/gui/file/{}",
        normalized.to_ascii_lowercase()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const EICAR_SHA256: &str = "275A021BBFB6489E54D471899F7DB9D1663FC695EC2FE2A2C4538AABF651FD0F";

    #[test]
    fn requires_explicit_user_request_and_valid_sha256() {
        assert_eq!(
            build_free_sha256_lookup_url(EICAR_SHA256, false),
            Err(LookupError::ConsentRequired)
        );
        assert_eq!(
            build_free_sha256_lookup_url("not-a-sha256", true),
            Err(LookupError::InvalidSha256)
        );
        assert_eq!(
            build_free_sha256_lookup_url(EICAR_SHA256, true),
            Ok(format!(
                "https://www.virustotal.com/gui/file/{}",
                EICAR_SHA256.to_ascii_lowercase()
            ))
        );
    }
}
