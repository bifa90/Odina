// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! How old this build is, said out loud (D113).
//!
//! The owner cancelled the 30/60/90 ladder: there is **no strict mode, no
//! acknowledgement button and no behaviour change** with age. A build that is
//! six months old still matches every hash it shipped with, so turning its
//! scanning into a prompt storm would punish the user for our release cadence.
//!
//! What remains is honesty in two places, both of them information:
//!
//! * the home page prints the release month in plain sight (`2026-04`), so the
//!   age is visible without a nagging dialog;
//! * from the sixth month on, the state title switches to the locale key
//!   `state.stale` («قاعدتك قديمة») and the update chip keeps its place beside
//!   it (D122). The wording lives in `locales/*/odina.ftl`, not here.
//!
//! The month is **compiled in from the release**, never read from a file's
//! modification time: mtime is one API call away from being forged, and the
//! whole point of the badge is that it cannot be moved without a new build.
//! Time arithmetic is month-number arithmetic — no day counting, no timezone,
//! no leap-year edge case. The caller supplies "now"; nothing here reads a
//! clock, so the whole module is a pure function that can be tested.

/// Months after which the badge appears. Six months, as decided.
pub const NEEDS_UPDATE_AFTER_MONTHS: u32 = 6;

/// The release month baked into this build by CI (`ODINA_RELEASE_MONTH=YYYY-MM`).
/// `None` means the build did not carry one; the UI then shows the version
/// instead of inventing a date it cannot know.
pub const RELEASE_MONTH: Option<&str> = option_env!("ODINA_RELEASE_MONTH");

/// A `YYYY-MM` stamp. Months are 1..=12; a year below 2000 is a typo or a
/// forged string, and both are refused rather than displayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReleaseMonth {
    pub year: u32,
    pub month: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonthReject {
    Shape,
    Year,
    Month,
}

impl ReleaseMonth {
    pub fn parse(text: &str) -> Result<Self, MonthReject> {
        let mut parts = text.split('-');
        let (Some(year), Some(month), None) = (parts.next(), parts.next(), parts.next()) else {
            return Err(MonthReject::Shape);
        };
        if year.len() != 4 || month.len() != 2 {
            return Err(MonthReject::Shape);
        }
        if !year.bytes().all(|b| b.is_ascii_digit()) || !month.bytes().all(|b| b.is_ascii_digit()) {
            return Err(MonthReject::Shape);
        }
        let year = year.parse::<u32>().map_err(|_| MonthReject::Year)?;
        let month = month.parse::<u32>().map_err(|_| MonthReject::Month)?;
        if year < 2000 {
            return Err(MonthReject::Year);
        }
        if !(1..=12).contains(&month) {
            return Err(MonthReject::Month);
        }
        Ok(Self { year, month })
    }

    /// Whole months from `self` (the release) to `now`. A clock that claims to
    /// be earlier than the build is not a reason to show a negative age: it
    /// returns `None` and the caller shows the month alone.
    pub fn months_until(self, now: Self) -> Option<u32> {
        let months =
            (now.year as i64 * 12 + now.month as i64) - (self.year as i64 * 12 + self.month as i64);
        u32::try_from(months).ok()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseAge {
    /// Under six months: nothing is shown beside «محمي».
    Current,
    /// Six months or older: the title switches to `state.stale`; the scanning
    /// result and the behaviour do not change.
    NeedsUpdate,
}

/// The message key that replaces the state title, or nothing. The text itself
/// is in the locale files (`state.stale`), so the screen has one wording in ten
/// languages and no Arabic string is compiled into the binary (18.2.9).
pub fn title_key(age: ReleaseAge) -> Option<&'static str> {
    match age {
        ReleaseAge::Current => None,
        ReleaseAge::NeedsUpdate => Some("state.stale"),
    }
}

pub fn release_age(months: u32) -> ReleaseAge {
    if months >= NEEDS_UPDATE_AFTER_MONTHS {
        ReleaseAge::NeedsUpdate
    } else {
        ReleaseAge::Current
    }
}

/// Convenience for the UI: the month to print, and whether the badge applies.
/// `None` when the build carries no month or the clock is behind the build —
/// in both cases the caller prints the version and shows no badge.
pub fn age_from_build(now: ReleaseMonth) -> Option<(ReleaseMonth, ReleaseAge)> {
    let released = ReleaseMonth::parse(RELEASE_MONTH?).ok()?;
    let months = released.months_until(now)?;
    Some((released, release_age(months)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(text: &str) -> ReleaseMonth {
        ReleaseMonth::parse(text).unwrap()
    }

    #[test]
    fn a_month_stamp_is_four_digits_two_digits_and_nothing_else() {
        assert_eq!(
            m("2026-04"),
            ReleaseMonth {
                year: 2026,
                month: 4
            }
        );
        for bad in [
            "",
            "2026",
            "2026-4",
            "26-04",
            "2026-13",
            "2026-00",
            "2026-04-01",
            "1999-12",
        ] {
            assert!(ReleaseMonth::parse(bad).is_err(), "{bad}");
        }
        assert_eq!(ReleaseMonth::parse("2026-4x"), Err(MonthReject::Shape));
        assert_eq!(ReleaseMonth::parse("1999-12"), Err(MonthReject::Year));
        assert_eq!(ReleaseMonth::parse("2026-13"), Err(MonthReject::Month));
    }

    #[test]
    fn months_are_counted_across_year_boundaries_without_days() {
        assert_eq!(m("2026-11").months_until(m("2026-11")), Some(0));
        assert_eq!(m("2026-11").months_until(m("2027-01")), Some(2));
        assert_eq!(m("2026-11").months_until(m("2027-05")), Some(6));
        assert_eq!(m("2025-01").months_until(m("2026-11")), Some(22));
        // A clock behind the build is not a negative age.
        assert_eq!(m("2026-11").months_until(m("2026-10")), None);
    }

    #[test]
    fn the_title_switches_at_six_months_and_changes_nothing_else() {
        assert_eq!(release_age(0), ReleaseAge::Current);
        assert_eq!(release_age(5), ReleaseAge::Current);
        assert_eq!(release_age(6), ReleaseAge::NeedsUpdate);
        assert_eq!(release_age(60), ReleaseAge::NeedsUpdate);
        assert_eq!(title_key(ReleaseAge::Current), None);
        assert_eq!(title_key(ReleaseAge::NeedsUpdate), Some("state.stale"));
        // There is no snooze and no strict state to fall into: the only two
        // answers are "current" and "still working, needs update".
        assert_eq!(NEEDS_UPDATE_AFTER_MONTHS, 6);
    }

    #[test]
    fn a_build_without_a_month_shows_nothing_rather_than_a_guess() {
        // RELEASE_MONTH is set by CI for release builds; this build may not
        // carry one, and both answers must be safe.
        match age_from_build(m("2027-01")) {
            Some((released, age)) => {
                assert_eq!(
                    released,
                    ReleaseMonth::parse(RELEASE_MONTH.unwrap()).unwrap()
                );
                assert_eq!(
                    age,
                    release_age(released.months_until(m("2027-01")).unwrap())
                );
            }
            None => assert!(
                RELEASE_MONTH.is_none() || ReleaseMonth::parse(RELEASE_MONTH.unwrap()).is_err()
            ),
        }
    }
}
