// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! Where a file sat, with nobody's name in it (D121).
//!
//! A report is worth more when it says *where* the threat hid: `%TEMP%` and
//! `%USERPROFILE%\Downloads` are the two places that matter most, and knowing
//! them changes what we warn about. A raw path is also the most personal thing
//! on the disk — it starts with a person's account name and often ends with a
//! document title.
//!
//! So the rule is neither "send the path" nor "send nothing": it is
//! **normalise, then mask the last piece**.
//!
//! * `C:\Users\ali\Downloads\invoice-2026.pdf` → `%USERPROFILE%\Downloads\*.pdf`
//! * `C:\Users\ali\AppData\Local\Temp\x.dll` → `%TEMP%\*.dll`
//! * `C:\$Recycle.Bin\S-1-5-21-…\a.exe` → `%SYSTEMDRIVE%\$Recycle.Bin\*.exe`
//!
//! The account name is removed by construction (the home prefix becomes a
//! token), the file name becomes `*` with its extension kept, and anything that
//! could still identify a person — an e-mail address, a Windows SID, a UNC
//! share naming a company server — is **refused**: a value we cannot prove
//! anonymous is not sent in a slightly cleaned form.
//!
//! This is why the field is named `path_pattern` and not `path`: what leaves
//! the machine is a location, never a place.

/// Longest path we will even look at. Windows' own limit is 260 for the classic
/// API, and anything longer is a malformed or hostile value rather than a
/// location worth reporting.
pub const MAX_PATH: usize = 260;

pub const HOME: &str = "%USERPROFILE%";
pub const SYSTEMROOT: &str = "%SYSTEMROOT%";
pub const PROGRAMFILES: &str = "%PROGRAMFILES%";
pub const PROGRAMFILES_X86: &str = "%PROGRAMFILES(X86)%";
pub const PROGRAMDATA: &str = "%PROGRAMDATA%";
pub const SYSTEMDRIVE: &str = "%SYSTEMDRIVE%";
pub const TEMP: &str = "%TEMP%";
pub const LOCALAPPDATA: &str = "%LOCALAPPDATA%";
pub const APPDATA: &str = "%APPDATA%";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathReject {
    Empty,
    TooLong,
    /// A relative path does not locate anything, so it carries no value.
    NotAbsolute,
    /// `\\server\share` names somebody's server.
    Unc,
    /// An e-mail address inside a folder name.
    Email,
    /// A Windows SID is a user identifier, not a location.
    Sid,
}

fn has_email(segment: &str) -> bool {
    segment.contains('@')
}

fn has_sid(segment: &str) -> bool {
    let upper = segment.to_ascii_uppercase();
    upper.starts_with("S-1-") && segment.bytes().filter(|b| *b == b'-').count() >= 3
}

/// The last piece becomes `*` plus a short extension. `report.exe` keeps
/// `.exe`; `invoice-ali-2026.pdf` loses everything except `.pdf`.
fn mask_leaf(leaf: &str) -> Option<String> {
    if leaf.is_empty() {
        return None;
    }
    let extension = leaf
        .rsplit_once('.')
        .map(|(_, ext)| ext)
        .filter(|ext| (2..=5).contains(&ext.len()) && ext.bytes().all(|b| b.is_ascii_alphanumeric()));
    Some(match extension {
        Some(ext) => format!("*.{}", ext.to_ascii_lowercase()),
        None => "*".to_string(),
    })
}

fn token_for(segment: &str) -> Option<&'static str> {
    let lower = segment.to_ascii_lowercase();
    match lower.as_str() {
        "windows" => Some(SYSTEMROOT),
        "program files" => Some(PROGRAMFILES),
        "program files (x86)" => Some(PROGRAMFILES_X86),
        "programdata" => Some(PROGRAMDATA),
        _ => None,
    }
}

/// The normal form. Returns a value that starts with a `%TOKEN%` and ends with
/// a masked leaf, or a reason why this path will not be sent at all.
pub fn normalize(raw: &str) -> Result<String, PathReject> {
    let path = raw.trim();
    if path.is_empty() {
        return Err(PathReject::Empty);
    }
    if path.len() > MAX_PATH {
        return Err(PathReject::TooLong);
    }
    if path.starts_with("\\\\") || path.starts_with("//") {
        return Err(PathReject::Unc);
    }
    if path.split(['\\', '/']).any(has_email) {
        return Err(PathReject::Email);
    }
    if path.split(['\\', '/']).any(has_sid) {
        return Err(PathReject::Sid);
    }

    let unified = path.replace('/', "\\");
    let mut segments: Vec<&str> = unified.split('\\').filter(|s| !s.is_empty()).collect();
    if segments.is_empty() {
        return Err(PathReject::Empty);
    }

    let mut head = SYSTEMDRIVE;
    let mut rest: &[&str] = &segments[1..];
    if segments.len() == 1 {
        // A bare drive ("C:") locates nothing worth reporting.
        return Err(PathReject::NotAbsolute);
    }
    let drive_like = segments[0].len() == 2
        && segments[0].as_bytes()[1] == b':'
        && segments[0].as_bytes()[0].is_ascii_alphabetic();
    if !drive_like {
        // Already-tokenised input (`%USERPROFILE%\...`) keeps its first token.
        if segments[0].starts_with('%') {
            head = "";
            rest = &segments[..];
        } else {
            return Err(PathReject::NotAbsolute);
        }
    }

    let mut kept: Vec<String> = Vec::new();
    if !head.is_empty() {
        kept.push(head.to_string());
    }

    let mut index = 0;
    // Home first: everything under `\Users\<name>` becomes `%USERPROFILE%`, and
    // the account name never enters the output.
    if rest.len() >= 2 {
        let first = rest[0].to_ascii_lowercase();
        let is_home = first == "users" || first == "documents and settings";
        if is_home {
            kept = vec![HOME.to_string()];
            index = 2;
            // The three well-known sub-trees get their own token, because
            // "%TEMP%\*.exe" is a more useful sentence than the full tail.
            let tail: Vec<&str> = rest[index..].to_vec();
            let lower: Vec<String> = tail.iter().map(|s| s.to_ascii_lowercase()).collect();
            if lower.len() >= 3 && lower[0] == "appdata" && lower[1] == "local" && lower[2] == "temp" {
                kept = vec![TEMP.to_string()];
                index += 3;
            } else if lower.len() >= 2 && lower[0] == "appdata" && lower[1] == "local" {
                kept = vec![LOCALAPPDATA.to_string()];
                index += 2;
            } else if lower.len() >= 2 && lower[0] == "appdata" && lower[1] == "roaming" {
                kept = vec![APPDATA.to_string()];
                index += 2;
            }
        }
    }
    if kept.is_empty() {
        if let Some(token) = token_for(rest[0]) {
            kept.push(token.to_string());
            index = 1;
        } else if rest[0].starts_with('%') {
            kept.push(rest[0].to_string());
            index = 1;
        } else if rest[0].eq_ignore_ascii_case("$recycle.bin") {
            kept.push("$Recycle.Bin".to_string());
            index = 1;
        } else {
            kept.push(SYSTEMDRIVE.to_string());
        }
    }

    // A SID directly after $Recycle.Bin is dropped, not masked: it identifies
    // the account that deleted the file.
    let mut survivors: Vec<String> = rest[index..]
        .iter()
        .map(|segment| segment.to_string())
        .collect();
    while survivors.first().is_some_and(|s| has_sid(s)) {
        survivors.remove(0);
    }
    for segment in &survivors {
        if has_email(segment) || has_sid(segment) {
            return Err(if has_email(segment) {
                PathReject::Email
            } else {
                PathReject::Sid
            });
        }
    }

    if survivors.is_empty() {
        // Nothing below the token: report the directory itself as the location.
        kept.push("*".to_string());
    } else {
        let leaf = survivors.pop().unwrap_or_default();
        kept.extend(survivors);
        kept.push(mask_leaf(&leaf).unwrap_or_else(|| "*".to_string()));
    }
    segments.clear();
    Ok(kept.join("\\"))
}

/// Whether a value already carries the normal form, which is what may be sent.
/// The check is deliberately strict: token first, masked leaf last, and no
/// identifying shape anywhere inside.
pub fn is_normalized(value: &str) -> bool {
    if value.is_empty() || value.len() > MAX_PATH || !value.starts_with('%') {
        return false;
    }
    if value.starts_with("\\\\") || value.contains('/') {
        return false;
    }
    let segments: Vec<&str> = value.split('\\').filter(|s| !s.is_empty()).collect();
    let Some((first, rest)) = segments.split_first() else {
        return false;
    };
    if !(first.starts_with('%') && first.ends_with('%')) {
        return false;
    }
    if rest.is_empty() {
        return false;
    }
    if rest.iter().any(|s| has_email(s) || has_sid(s)) {
        return false;
    }
    let leaf = rest[rest.len() - 1];
    leaf.starts_with('*')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_account_name_never_survives_the_normal_form() {
        assert_eq!(
            normalize(r"C:\Users\ali\Downloads\invoice-2026.pdf"),
            Ok(format!("{HOME}\\Downloads\\*.pdf"))
        );
        assert_eq!(
            normalize(r"C:\Users\ali\AppData\Local\Temp\x.dll"),
            Ok(format!("{TEMP}\\*.dll"))
        );
        assert_eq!(
            normalize(r"C:\Users\Ali\AppData\Roaming\AnyDesk\a.exe"),
            Ok(format!("{APPDATA}\\AnyDesk\\*.exe"))
        );
        assert_eq!(
            normalize(r"C:\Users\ali\AppData\Local\Google\Chrome\x.crx"),
            Ok(format!("{LOCALAPPDATA}\\Google\\Chrome\\*.crx"))
        );
        assert_eq!(
            normalize(r"C:\Users\ali\Desktop\a.exe"),
            Ok(format!("{HOME}\\Desktop\\*.exe"))
        );
        for value in [
            normalize(r"C:\Users\ali\Downloads\invoice-2026.pdf").unwrap(),
            normalize(r"C:\Users\ali\AppData\Local\Temp\x.dll").unwrap(),
        ] {
            assert!(!value.to_ascii_lowercase().contains("ali"), "{value}");
            assert!(is_normalized(&value), "{value}");
        }
    }

    #[test]
    fn the_location_is_kept_wherever_the_file_hides() {
        assert_eq!(
            normalize(r"C:\Windows\System32\bad.exe"),
            Ok(format!("{SYSTEMROOT}\\System32\\*.exe"))
        );
        assert_eq!(
            normalize(r"D:\Program Files\App\evil.exe"),
            Ok(format!("{PROGRAMFILES}\\App\\*.exe"))
        );
        assert_eq!(
            normalize(r"C:\Program Files (x86)\Old\a.dll"),
            Ok(format!("{PROGRAMFILES_X86}\\Old\\*.dll"))
        );
        assert_eq!(
            normalize(r"C:\ProgramData\Vendor\agent.exe"),
            Ok(format!("{PROGRAMDATA}\\Vendor\\*.exe"))
        );
        assert_eq!(
            normalize(r"C:\$Recycle.Bin\S-1-5-21-111-222-333-1001\a.exe"),
            Ok(format!("{SYSTEMDRIVE}\\$Recycle.Bin\\*.exe"))
        );
        // A path that is already a token is kept as a token.
        assert_eq!(
            normalize("%APPDATA%\\Vendor\\a.exe"),
            Ok(format!("{APPDATA}\\Vendor\\*.exe"))
        );
    }

    #[test]
    fn the_last_piece_is_masked_and_the_extension_is_all_that_stays() {
        assert!(normalize(r"C:\Users\ali\Desktop\my-salaries.xlsx")
            .unwrap()
            .ends_with("\\*.xlsx"));
        assert!(normalize(r"C:\Users\ali\Desktop\README")
            .unwrap()
            .ends_with("\\*"));
        // A long "extension" is not an extension; it goes with the file name.
        assert!(normalize(r"C:\Users\ali\Desktop\a.verylongthing")
            .unwrap()
            .ends_with("\\*"));
        // A folder with nothing under it still reports its own location.
        assert_eq!(normalize(r"C:\Users\ali\Downloads"), Ok(format!("{HOME}\\*")));
    }

    #[test]
    fn values_that_cannot_be_proven_anonymous_are_refused_not_cleaned() {
        assert_eq!(normalize(""), Err(PathReject::Empty));
        assert_eq!(normalize(r"\\fileserver\share\a.exe"), Err(PathReject::Unc));
        assert_eq!(normalize(r"C:\Users\ali@example.com\a.exe"), Err(PathReject::Email));
        assert_eq!(
            normalize(r"C:\Temp\S-1-5-21-111-222-333-1001\a.exe"),
            Err(PathReject::Sid)
        );
        assert_eq!(normalize("Downloads\\a.exe"), Err(PathReject::NotAbsolute));
        assert_eq!(normalize("C:"), Err(PathReject::NotAbsolute));
        assert_eq!(normalize(&format!("C:\\{}", "a".repeat(MAX_PATH))), Err(PathReject::TooLong));
    }

    #[test]
    fn what_counts_as_normal_form_is_exact() {
        let good = format!("{HOME}\\Downloads\\*.exe");
        assert!(is_normalized(&good));
        assert!(is_normalized(&format!("{TEMP}\\*")));
        // Not tokenised, not masked, or carrying an identity: all refused.
        assert!(!is_normalized(r"C:\Users\ali\Downloads\a.exe"));
        assert!(!is_normalized(&format!("{HOME}\\Downloads\\a.exe")));
        assert!(!is_normalized(&format!("{HOME}\\ali@example.com\\*")));
        assert!(!is_normalized(&format!("{HOME}\\S-1-5-21-1-2-3-4\\*")));
        assert!(!is_normalized(&format!("{HOME}")));
        assert!(!is_normalized("*.exe"));
        assert!(!is_normalized(""));
    }

    #[test]
    fn every_environment_token_is_reachable_from_a_real_windows_path() {
        let pairs = [
            (r"C:\Windows\a.exe", SYSTEMROOT),
            (r"C:\Program Files\a.exe", PROGRAMFILES),
            (r"C:\Program Files (x86)\a.exe", PROGRAMFILES_X86),
            (r"C:\ProgramData\a.exe", PROGRAMDATA),
            (r"C:\Users\ali\a.exe", HOME),
            (r"C:\Users\ali\AppData\Local\Temp\a.exe", TEMP),
            (r"C:\Users\ali\AppData\Local\a.exe", LOCALAPPDATA),
            (r"C:\Users\ali\AppData\Roaming\a.exe", APPDATA),
            (r"D:\Vendor\a.exe", SYSTEMDRIVE),
        ];
        for (raw, token) in pairs {
            let normalized = normalize(raw).unwrap();
            assert!(normalized.starts_with(token), "{raw} -> {normalized}");
        }
    }
}
