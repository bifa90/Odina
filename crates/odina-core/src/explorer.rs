// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K6 Windows Explorer Context Menu Contract.
//!
//! Enforces:
//! - "Scan with Odina" targets only the selected local file/folder/drive within scope.
//! - Rejects remote/UNC/out-of-scope selections.
//! - Never reports "clean" when the resident service is stopped.
//! - Supports Arabic and Unicode filenames identically to ASCII.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplorerAction {
    pub path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplorerVerdict {
    ServiceStopped,
    OutOfScope,
    Unknown,
}

pub fn validate_explorer_selection(
    path: &str,
    scope_root: &str,
    service_running: bool,
) -> Result<ExplorerAction, ExplorerVerdict> {
    let trimmed = path.trim();
    if trimmed.is_empty()
        || trimmed.starts_with("\\\\")
        || trimmed.starts_with("//")
        || trimmed.contains("://")
    {
        return Err(ExplorerVerdict::OutOfScope);
    }
    if !trimmed.starts_with(scope_root) {
        return Err(ExplorerVerdict::OutOfScope);
    }
    if !service_running {
        return Err(ExplorerVerdict::ServiceStopped);
    }
    Ok(ExplorerAction {
        path: trimmed.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arabic_filename_in_scope_is_accepted_when_service_runs() {
        let res = validate_explorer_selection(
            "C:\\Users\\user\\ملف_تجريبي.exe",
            "C:\\Users\\user",
            true,
        );
        assert_eq!(
            res,
            Ok(ExplorerAction {
                path: "C:\\Users\\user\\ملف_تجريبي.exe".to_string()
            })
        );
    }

    #[test]
    fn stopped_service_and_out_of_scope_never_claim_clean() {
        assert_eq!(
            validate_explorer_selection("C:\\Users\\user\\a.txt", "C:\\Users\\user", false),
            Err(ExplorerVerdict::ServiceStopped)
        );
        assert_eq!(
            validate_explorer_selection("\\\\server\\share\\a.txt", "C:\\Users\\user", true),
            Err(ExplorerVerdict::OutOfScope)
        );
        assert_eq!(
            validate_explorer_selection("D:\\Other\\a.txt", "C:\\Users\\user", true),
            Err(ExplorerVerdict::OutOfScope)
        );
    }
}
