// Copyright (c) 2026 bifa90 — Odina
// SPDX-License-Identifier: AGPL-3.0-or-later
//! What the window is allowed to say, and where each word comes from.
//!
//! The window draws; it does not decide. Every string here is a key resolved
//! against the embedded `.ftl` files, and every state comes from the resident
//! over the local pipe. When the pipe does not answer the state is
//! `Disconnected`, which reads «لم يُتحقق منه بعد» — never «أنت محمي» (K5).
//!
//! A build with no release month shows the version and no date: the month is
//! injected at release time and is never guessed from a file's timestamp.

use odina_core::l10n;
use odina_core::messages;
use odina_core::release_age::{self, ReleaseAge, ReleaseMonth};
use odina_core::ui_state::{ServiceHealth, TextDirection, UiState};
use serde::Serialize;

/// The one screen the shell knows: what the guard is, how old it is, and
/// whether a newer release exists. It carries no story of its own.
#[derive(Debug, Clone, Serialize)]
pub struct Home {
    pub lang: String,
    pub dir: String,
    pub title: String,
    pub chip: Option<String>,
    pub month: Option<String>,
    pub version: String,
    /// Why the screen looks the way it does, when the state line cannot say it:
    /// the service is not running, or this build carries no release month.
    pub detail: Option<String>,
    pub stale: bool,
    pub health: String,
}

fn health_word(health: ServiceHealth) -> &'static str {
    match health {
        ServiceHealth::Active => "active",
        ServiceHealth::Suspended => "suspended",
        ServiceHealth::Stopped => "stopped",
        ServiceHealth::Disconnected => "disconnected",
    }
}

/// The state from a run: the language the window asked for, the health it
/// measured, the age of the build, and the release month the build carries.
pub fn view(
    lang: &str,
    health: ServiceHealth,
    age: Option<(ReleaseMonth, ReleaseAge)>,
    release_month: Option<&str>,
) -> Home {
    let lang = if l10n::source(lang).is_some() {
        lang.to_string()
    } else {
        String::from("ar")
    };
    let state = UiState::new(health, lang.clone());
    let (month, stale) = match age {
        Some((released, age)) => (
            Some(format!("{:04}-{:02}", released.year, released.month)),
            age == ReleaseAge::NeedsUpdate,
        ),
        None => (None, false),
    };

    // The state line wins over the age line: a stopped guard is never renamed
    // into a stale one, because "your threat list is old" implies it still runs.
    let mut title_key = state.state_key();
    if stale && title_key == messages::STATE_PROTECTED {
        title_key = messages::STATE_STALE;
    }
    let title = l10n::translate(&lang, title_key).unwrap_or_else(|| title_key.to_string());
    let version_args = [("v", odina_core::VERSION)];
    let version = l10n::translate_args(&lang, messages::HOME_VERSION, &version_args)
        .unwrap_or_else(|| odina_core::VERSION.to_string());
    let detail_key = if health == ServiceHealth::Disconnected {
        Some(messages::HOME_SERVICE_OFF)
    } else if age.is_none() && release_month.is_none() {
        Some(messages::HOME_DEV_BUILD)
    } else {
        None
    };
    let detail = detail_key.and_then(|key| l10n::translate(&lang, key));
    let dir = match state.direction() {
        TextDirection::Rtl => "rtl",
        TextDirection::Ltr => "ltr",
    };

    Home {
        lang,
        dir: dir.to_string(),
        title,
        // The chip is drawn only when a check found a newer release; nothing
        // here may invent one.
        chip: None,
        month,
        version,
        detail,
        stale,
        health: health_word(health).to_string(),
    }
}

/// Ask the resident for its state. Silence is `Disconnected`, not `Active`.
pub fn measured_health() -> ServiceHealth {
    let reply = odina_resident::request("main", odina_resident::Command::Status);
    match reply {
        Ok(reply) if reply.code == 0 => {
            if reply.body.contains("\"phase\":\"suspended\"") {
                ServiceHealth::Suspended
            } else {
                ServiceHealth::Active
            }
        }
        _ => ServiceHealth::Disconnected,
    }
}

#[tauri::command]
fn home(lang: String) -> Home {
    let age = release_age::current_month().and_then(release_age::age_from_build);
    view(&lang, measured_health(), age, release_age::RELEASE_MONTH)
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![home])
        .run(tauri::generate_context!())
        .expect("the window did not start");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn month(text: &str) -> ReleaseMonth {
        ReleaseMonth::parse(text).unwrap()
    }

    #[test]
    fn a_stopped_guard_is_never_renamed_into_a_stale_one() {
        let age = Some((month("2025-01"), ReleaseAge::NeedsUpdate));
        let home = view("ar", ServiceHealth::Stopped, age, Some("2025-01"));
        assert!(home.stale, "the age is still reported");
        assert_eq!(
            home.title,
            l10n::translate("ar", messages::STATE_UNPROTECTED).unwrap()
        );
    }

    #[test]
    fn a_running_old_guard_says_its_list_is_old() {
        let age = Some((month("2025-01"), ReleaseAge::NeedsUpdate));
        let home = view("ar", ServiceHealth::Active, age, Some("2025-01"));
        assert_eq!(
            home.title,
            l10n::translate("ar", messages::STATE_STALE).unwrap()
        );
    }

    #[test]
    fn no_release_month_means_no_date_rather_than_a_guess() {
        let home = view("ar", ServiceHealth::Active, None, Some("2026-11"));
        assert_eq!(home.month, None);
        assert!(!home.stale);
        assert_eq!(home.detail, None, "a dated build explains nothing away");
        assert_eq!(
            home.title,
            l10n::translate("ar", messages::STATE_PROTECTED).unwrap()
        );
    }

    #[test]
    fn the_three_lines_come_from_the_locale_files_in_every_language() {
        for &lang in l10n::LANGS {
            let home = view(lang, ServiceHealth::Active, None, Some("2026-11"));
            assert_eq!(home.lang, lang);
            assert!(!home.title.is_empty(), "{lang} title");
            assert!(home.version.contains(odina_core::VERSION), "{lang} version");
            assert_eq!(home.chip, None);
        }
    }

    #[test]
    fn the_screen_says_why_the_service_is_silent_and_why_a_month_is_missing() {
        let off = view("ar", ServiceHealth::Disconnected, None, Some("2026-11"));
        assert_eq!(
            off.detail,
            l10n::translate("ar", messages::HOME_SERVICE_OFF)
        );
        let dev = view("ar", ServiceHealth::Active, None, None);
        assert_eq!(dev.detail, l10n::translate("ar", messages::HOME_DEV_BUILD));
    }

    #[test]
    fn an_unknown_language_falls_back_to_arabic_and_the_direction_follows_it() {
        let home = view("xx", ServiceHealth::Active, None, Some("2026-11"));
        assert_eq!(home.lang, "ar");
        assert_eq!(home.dir, "rtl");
        assert_eq!(
            view("en", ServiceHealth::Active, None, Some("2026-11")).dir,
            "ltr"
        );
    }
}
