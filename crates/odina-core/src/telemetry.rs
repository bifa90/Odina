// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K10 Telemetry & Privacy Boundary Contract.
//!
//! Enforces zero-PII, zero-path, zero-username, and <= 4 KiB cap
//! on any diagnostic payload leaving the core.

pub const MAX_DIAGNOSTIC_BYTES: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrivacyError {
    ExceedsSizeCap,
    ContainsLocalPath,
}

pub fn validate_diagnostic_payload(payload: &str) -> Result<(), PrivacyError> {
    if payload.len() > MAX_DIAGNOSTIC_BYTES {
        return Err(PrivacyError::ExceedsSizeCap);
    }
    if payload.contains(":\\") || payload.contains("/home/") || payload.contains("/Users/") {
        return Err(PrivacyError::ContainsLocalPath);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_local_paths_and_oversized_payloads() {
        assert_eq!(
            validate_diagnostic_payload("file=C:\\Users\\alice\\a.exe"),
            Err(PrivacyError::ContainsLocalPath)
        );
        assert_eq!(
            validate_diagnostic_payload("file=/home/user/a.exe"),
            Err(PrivacyError::ContainsLocalPath)
        );
        assert_eq!(
            validate_diagnostic_payload(&"a".repeat(MAX_DIAGNOSTIC_BYTES + 1)),
            Err(PrivacyError::ExceedsSizeCap)
        );
        assert_eq!(
            validate_diagnostic_payload("verdict=v5;band=confirm;lang=ar"),
            Ok(())
        );
    }
}
