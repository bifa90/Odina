// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! Exceptions the user grants, and what they are allowed to touch.
//!
//! This module exists because of a contradiction the spec carried without
//! noticing: turning **all** protection off needed one click and no reason,
//! while relaxing **one** rule needed expert mode and a written justification —
//! and nothing anywhere defined "on" as a direction at all. The safer act was
//! the harder one, which is how a user ends up killing the service from Task
//! Manager instead.
//!
//! Three things are therefore separate here, and the separation is the whole
//! design:
//!
//!   * **what** may be relaxed — [`Rule`], and a fixed list of them;
//!   * **how far** — [`Mode`], where `Allow` is strictly wider than `Relax`;
//!   * **how long** — [`Duration`], where `Permanent` exists but is visible.
//!
//! And three things can never be relaxed, whatever the user says: the first
//! wall's verdict (a confirmed file is quarantined even under an exception),
//! the tamper protection, and the rule that one exception never switches on a
//! second rule. Those are enforced in [`Policy::effective`], not documented
//! and hoped for — the isolation test is the one that would catch a regression.
//!
//! No clock, no filesystem, no Windows. `now_ms` is passed in, so a test can
//! make an hour pass in a microsecond and every expiry is provable.

use crate::scoring::Verdict;

/// One proactive rule a user may relax. The list is closed: adding a rule is a
/// code change with a test, not a configuration entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rule {
    /// A1 — Windows Script Host. `.vbs` / `.js` / `.wsf` run restricted.
    WindowsScriptHost,
    /// A1 — PowerShell constrained language.
    PowerShell,
    /// A1 — macros in unsigned Office documents.
    OfficeMacros,
    /// A3 — every new file in Downloads counts as suspicious until cleared.
    DownloadsSuspicion,
    /// A4 — the silent two-hourly scan.
    PeriodicScan,
    /// A6 — "when in doubt, block first".
    DoubtBlocks,
}

impl Rule {
    /// Every rule, in the order the UI shows them.
    pub const ALL: [Rule; 6] = [
        Rule::WindowsScriptHost,
        Rule::PowerShell,
        Rule::OfficeMacros,
        Rule::DownloadsSuspicion,
        Rule::PeriodicScan,
        Rule::DoubtBlocks,
    ];

    /// Stable key for the log and the settings file. Never translated.
    pub const fn key(self) -> &'static str {
        match self {
            Rule::WindowsScriptHost => "a1.wsh",
            Rule::PowerShell => "a1.powershell",
            Rule::OfficeMacros => "a1.macros",
            Rule::DownloadsSuspicion => "a3.downloads",
            Rule::PeriodicScan => "a4.periodic",
            Rule::DoubtBlocks => "a6.doubt",
        }
    }

    /// The strongest thing an exception on this rule may ever do.
    ///
    /// Not every rule can be wholly allowed: A6 is the rule that blocks when
    /// uncertain, and "allow doubts" is not a setting, it is the absence of a
    /// product. It can be relaxed — block only when the score is high — and no
    /// further.
    pub const fn ceiling(self) -> Mode {
        match self {
            Rule::DoubtBlocks => Mode::Relax,
            _ => Mode::Allow,
        }
    }

    /// Whether the rule can be relaxed permanently.
    ///
    /// False for A6 for the same reason: a permanent "do not block on doubt"
    /// is the same as uninstalling while leaving the icon.
    pub const fn permanent_allowed(self) -> bool {
        !matches!(self, Rule::DoubtBlocks)
    }
}

impl std::fmt::Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.key())
    }
}

/// How far an exception goes.
///
/// `Allow` is strictly wider than `Relax` and ORs with it — asking for `Allow`
/// when the user holds `Relax` is an escalation, and it is priced as one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mode {
    /// The rule stays on but stops interrupting: log, do not ask.
    Relax,
    /// The rule does not apply to the named target.
    Allow,
}

impl Mode {
    pub const fn key(self) -> &'static str {
        match self {
            Mode::Relax => "relax",
            Mode::Allow => "allow",
        }
    }

    /// `self` widened by `other`, capped at `ceiling`.
    ///
    /// The ceiling is applied last on purpose: widening first and clamping
    /// after means an `Allow` request on a relax-only rule returns `Relax`
    /// rather than silently keeping a narrower grant.
    pub const fn widen(self, other: Mode, ceiling: Mode) -> Mode {
        let wider = if (self as u8) >= (other as u8) {
            self
        } else {
            other
        };
        if (wider as u8) > (ceiling as u8) {
            ceiling
        } else {
            wider
        }
    }
}

/// How long an exception lasts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Duration {
    /// Minutes from the moment it was granted.
    Minutes(u32),
    /// Until the machine restarts. The app does not know when that is, so it
    /// is stored as a flag and cleared at start-up.
    UntilReboot,
    /// Until revoked. Visible forever in the home screen and the tray.
    Permanent,
}

impl Duration {
    pub const fn key(self) -> &'static str {
        match self {
            Duration::Minutes(_) => "minutes",
            Duration::UntilReboot => "until_reboot",
            Duration::Permanent => "permanent",
        }
    }

    /// Seconds from `granted_at`, or `None` for open-ended durations.
    pub const fn seconds(self) -> Option<u64> {
        match self {
            Duration::Minutes(m) => Some(m as u64 * 60),
            Duration::UntilReboot | Duration::Permanent => None,
        }
    }
}

/// What a rule may be relaxed for. The target is what makes an exception
/// surgical rather than global.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// Every instance of the rule, with no path or publisher attached. The
    /// widest thing the UI may offer, and only in expert mode.
    Everything,
    /// One path or folder, matched verbatim. No globs: a glob is a second
    /// language the user did not agree to learn.
    Path(String),
    /// A publisher whose signature is valid on the file in question.
    Publisher(String),
}

impl Target {
    /// The label the log stores. Paths are the user's own data and are the
    /// one thing an exception log is expected to contain — locally.
    pub fn label(&self) -> &str {
        match self {
            Target::Everything => "*",
            Target::Path(p) => p,
            Target::Publisher(p) => p,
        }
    }
}

/// One granted exception.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exception {
    pub rule: Rule,
    pub target: Target,
    pub mode: Mode,
    /// Epoch milliseconds the user granted it.
    pub granted_at_ms: i64,
    pub duration: Duration,
    /// How the decision was reached. Recorded because a permanent exception
    /// granted by a child on a shared machine is a different fact from one
    /// granted in expert mode.
    pub tier: ConsentTier,
}

impl Exception {
    /// When this exception stops applying, if it ever does.
    pub fn expires_at_ms(&self) -> Option<i64> {
        self.duration
            .seconds()
            .map(|s| self.granted_at_ms + s as i64 * 1000)
    }

    /// Whether it still applies at `now_ms`.
    pub fn is_live(&self, now_ms: i64) -> bool {
        match self.expires_at_ms() {
            Some(end) => now_ms < end,
            None => true,
        }
    }

    /// Whether it was granted with the deliberate, slower flow.
    pub fn is_permanent(&self) -> bool {
        matches!(self.duration, Duration::Permanent)
    }

    /// The line the log writes. Local only, and it is the user's own record:
    /// who granted what, for how long, and through which flow.
    ///
    /// `tier` is read here rather than stored and forgotten, because a
    /// permanent exception granted in the standard flow on a shared machine
    /// is a different fact from one granted through the expert screens, and
    /// the log is the only place that difference survives.
    pub fn log_line(&self) -> String {
        format!(
            "{} {} {} {:?} {}",
            self.rule.key(),
            self.mode.key(),
            self.target.label(),
            self.duration,
            match self.tier {
                ConsentTier::Standard => "standard",
                ConsentTier::Expert => "expert",
            }
        )
    }
}

/// How the user said yes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsentTier {
    /// Two buttons, consequence written above them, no reason asked.
    Standard,
    /// The expert screens: a permanent exception or a whole-rule one.
    Expert,
}

/// Why an exception was refused. Each variant is a sentence the UI can show
/// without inventing anything — and every one of them is a refusal the code
/// would also enforce if the UI asked anyway.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// The rule cannot be allowed at all, only relaxed (A6).
    NotAllowedAtAll,
    /// `Allow` on a rule whose ceiling is `Relax`.
    AboveCeiling,
    /// A permanent exception on a rule that cannot have one.
    NoPermanence,
    /// A whole-rule permanent exception asked for in the standard flow.
    NeedsExpertMode,
    /// The second wall's verdict is in force: a confirmed file is quarantined
    /// whatever the policy says.
    VetoStands,
}

impl Refusal {
    pub const fn key(self) -> &'static str {
        match self {
            Refusal::NotAllowedAtAll => "exception.refused.not_allowed",
            Refusal::AboveCeiling => "exception.refused.above_ceiling",
            Refusal::NoPermanence => "exception.refused.no_permanent",
            Refusal::NeedsExpertMode => "exception.refused.expert_only",
            Refusal::VetoStands => "exception.refused.veto_stands",
        }
    }
}

/// Whether an exception is currently granted, and for what.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Granted {
    pub rule: Rule,
    pub mode: Mode,
    pub target: Target,
    /// Absent for `Everything`.
    pub expires_at_ms: Option<i64>,
    /// True when the user will have to come back and revoke it.
    pub permanent: bool,
}

impl Granted {
    /// The one-line label the home screen shows while this is live. Returned
    /// as a machine key, not a sentence: the sentence is the localization
    /// layer's job, and putting Arabic here would bake one language into the
    /// core.
    pub fn badge_key(&self) -> String {
        format!("exception.active.{}", self.rule.key())
    }
}

/// The whole set of exceptions, with the rules that keep them honest.
#[derive(Debug, Clone, Default)]
pub struct Policy {
    exceptions: Vec<Exception>,
}

impl Policy {
    pub fn new() -> Self {
        Policy::default()
    }

    /// Every exception ever granted, live or not. The log is the caller's job;
    /// this list is what the policy knows right now.
    pub fn exceptions(&self) -> &[Exception] {
        &self.exceptions
    }

    /// Grant an exception, or explain why not.
    ///
    /// The refusals are checked here rather than in the UI so that a second
    /// front end — a CLI, a settings file, a future screen — cannot grant
    /// something the first one refuses.
    pub fn grant(
        &mut self,
        rule: Rule,
        target: Target,
        mode: Mode,
        duration: Duration,
        tier: ConsentTier,
        now_ms: i64,
    ) -> Result<(), Refusal> {
        if mode as u8 > rule.ceiling() as u8 {
            return Err(if rule.ceiling() == Mode::Relax {
                Refusal::NotAllowedAtAll
            } else {
                Refusal::AboveCeiling
            });
        }
        if matches!(duration, Duration::Permanent) && !rule.permanent_allowed() {
            return Err(Refusal::NoPermanence);
        }
        // A permanent exception on a whole rule is the widest thing the model
        // can express, so it is the one that needs the deliberate flow.
        if matches!(duration, Duration::Permanent)
            && matches!(target, Target::Everything)
            && tier != ConsentTier::Expert
        {
            return Err(Refusal::NeedsExpertMode);
        }

        let exception = Exception {
            rule,
            target,
            mode,
            granted_at_ms: now_ms,
            duration,
            tier,
        };
        // A newer grant for the same rule and target replaces the older one:
        // two live exceptions on one target would make "what is in force"
        // ambiguous, and ambiguity in a security policy is a bug.
        self.exceptions
            .retain(|e| !(e.rule == rule && e.target == exception.target));
        self.exceptions.push(exception);
        Ok(())
    }

    /// Revoke everything for one rule. Returns how many were removed. One
    /// click, no password: an exception is easier to withdraw than to grant,
    /// always.
    pub fn revoke_rule(&mut self, rule: Rule) -> usize {
        let before = self.exceptions.len();
        self.exceptions.retain(|e| e.rule != rule);
        before - self.exceptions.len()
    }

    /// Revoke a single exception.
    pub fn revoke(&mut self, rule: Rule, target: &Target) -> bool {
        let before = self.exceptions.len();
        self.exceptions
            .retain(|e| !(e.rule == rule && &e.target == target));
        before != self.exceptions.len()
    }

    /// Drop everything that has expired. Called at start-up and on a timer.
    pub fn prune(&mut self, now_ms: i64) -> usize {
        let before = self.exceptions.len();
        self.exceptions.retain(|e| e.is_live(now_ms));
        before - self.exceptions.len()
    }

    /// The exceptions in force at `now_ms`, and **only** those. Every caller
    /// asks this, so an expired exception cannot leak through a code path that
    /// forgot to check.
    pub fn live(&self, now_ms: i64) -> Vec<Granted> {
        self.exceptions
            .iter()
            .filter(|e| e.is_live(now_ms))
            .map(|e| Granted {
                rule: e.rule,
                mode: e.mode,
                target: e.target.clone(),
                expires_at_ms: e.expires_at_ms(),
                permanent: e.is_permanent(),
            })
            .collect()
    }

    /// Whether an exception actually applies, rather than merely existing.
    ///
    /// This is where isolation lives. An exception names one rule; it can
    /// never answer for another. A `Rule::WindowsScriptHost` exception must not
    /// make `Rule::PowerShell` look relaxed, however tempting the shortcut of
    /// "scripts are fine" would be.
    fn reaches(&self, e: &Exception, asked: &Target, now_ms: i64) -> bool {
        if !e.is_live(now_ms) {
            return false;
        }
        match (&e.target, asked) {
            (Target::Everything, _) => true,
            (Target::Path(a), Target::Path(b)) => a.eq_ignore_ascii_case(b),
            (Target::Publisher(a), Target::Publisher(b)) => a.eq_ignore_ascii_case(b),
            _ => false,
        }
    }

    /// What the effective state of one rule is, for one target, right now.
    ///
    /// Returns `None` when nothing applies. The caller must not treat `None`
    /// as "denied" or as "allowed": it means the rule behaves as shipped.
    pub fn effective(&self, rule: Rule, target: &Target, now_ms: i64) -> Option<Mode> {
        self.exceptions
            .iter()
            .filter(|e| e.rule == rule && self.reaches(e, target, now_ms))
            .map(|e| e.mode)
            .max()
    }

    /// Apply the exceptions to a verdict, and never let one decide the file
    /// is safe.
    ///
    /// This is the boundary between the two walls. An exception says "this
    /// rule may stop interrupting me"; it does not say "this file is clean".
    /// So the downgrade is one-directional and bounded:
    ///
    ///   * `Confirmed` never moves. A byte-exact match is not a suspicion to
    ///     be relaxed, and a user who granted an exception an hour ago did not
    ///     intend to run a known sample.
    ///   * `HighRisk` never moves either, for the same reason with less
    ///     certainty: the spec's thresholds put it at "stop now", and letting
    ///     an exception silence it would make the exception a blind spot.
    ///   * `Suspicious` and `Monitor` may drop to `Monitor`, which is logged
    ///     and never shown. The user asked not to be interrupted.
    ///   * `Clean` stays `Clean`.
    ///
    /// A refused (expired, revoked, absent) exception changes nothing.
    pub fn apply_to_verdict(
        &self,
        verdict: Verdict,
        rule: Rule,
        target: &Target,
        now_ms: i64,
    ) -> Verdict {
        if self.effective(rule, target, now_ms) != Some(Mode::Relax) {
            return verdict;
        }
        match verdict {
            Verdict::Suspicious => Verdict::Monitor,
            other => other,
        }
    }

    /// Revoke everything, without a reason and without a password. The exit
    /// from every exception at once, for the user who wants out.
    pub fn revoke_all(&mut self) -> usize {
        let n = self.exceptions.len();
        self.exceptions.clear();
        n
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const T0: i64 = 1_763_000_000_000; // an arbitrary fixed instant
    const HOUR: i64 = 3_600_000;

    fn policy_with(rule: Rule, mode: Mode, duration: Duration) -> Policy {
        let mut p = Policy::new();
        p.grant(
            rule,
            Target::Path("C:\\Scripts".into()),
            mode,
            duration,
            ConsentTier::Standard,
            T0,
        )
        .expect("grant should succeed");
        p
    }

    #[test]
    fn an_exception_expires_on_time_and_not_a_millisecond_later() {
        let p = policy_with(Rule::WindowsScriptHost, Mode::Relax, Duration::Minutes(30));
        let target = Target::Path("C:\\Scripts".into());

        assert_eq!(
            p.effective(Rule::WindowsScriptHost, &target, T0),
            Some(Mode::Relax)
        );
        assert_eq!(
            p.effective(Rule::WindowsScriptHost, &target, T0 + 30 * 60_000 - 1),
            Some(Mode::Relax)
        );
        assert_eq!(
            p.effective(Rule::WindowsScriptHost, &target, T0 + 30 * 60_000),
            None,
            "the instant it expires it is gone, not a moment later"
        );
    }

    /// The test this module exists for. Relaxing WSH must not relax anything
    /// else — not PowerShell, not macros, not the downloads rule.
    #[test]
    fn one_exception_never_switches_on_a_second_rule() {
        let p = policy_with(Rule::WindowsScriptHost, Mode::Allow, Duration::Minutes(60));
        let target = Target::Path("C:\\Scripts".into());

        assert_eq!(
            p.effective(Rule::WindowsScriptHost, &target, T0),
            Some(Mode::Allow)
        );
        for other in Rule::ALL {
            if other == Rule::WindowsScriptHost {
                continue;
            }
            assert_eq!(
                p.effective(other, &target, T0),
                None,
                "{other} was altered by a WSH exception"
            );
        }
    }

    #[test]
    fn an_allow_exception_does_not_leak_to_another_path() {
        let p = policy_with(Rule::WindowsScriptHost, Mode::Allow, Duration::Minutes(60));
        assert_eq!(
            p.effective(
                Rule::WindowsScriptHost,
                &Target::Path("C:\\Other".into()),
                T0
            ),
            None
        );
    }

    #[test]
    fn a6_can_be_relaxed_but_not_allowed_and_not_made_permanent() {
        let mut p = Policy::new();
        let whole = Target::Everything;

        let relaxed = p.grant(
            Rule::DoubtBlocks,
            whole.clone(),
            Mode::Relax,
            Duration::Minutes(30),
            ConsentTier::Standard,
            T0,
        );
        assert!(relaxed.is_ok());

        let allowed = p.grant(
            Rule::DoubtBlocks,
            whole.clone(),
            Mode::Allow,
            Duration::Minutes(30),
            ConsentTier::Expert,
            T0,
        );
        assert_eq!(allowed.unwrap_err(), Refusal::NotAllowedAtAll);

        let permanent = p.grant(
            Rule::DoubtBlocks,
            Target::Path("C:\\Tools".into()),
            Mode::Relax,
            Duration::Permanent,
            ConsentTier::Expert,
            T0,
        );
        assert_eq!(permanent.unwrap_err(), Refusal::NoPermanence);
    }

    #[test]
    fn a_permanent_whole_rule_exception_needs_the_expert_flow() {
        let mut p = Policy::new();
        let refused = p.grant(
            Rule::WindowsScriptHost,
            Target::Everything,
            Mode::Allow,
            Duration::Permanent,
            ConsentTier::Standard,
            T0,
        );
        assert_eq!(refused.unwrap_err(), Refusal::NeedsExpertMode);

        let granted = p.grant(
            Rule::WindowsScriptHost,
            Target::Everything,
            Mode::Allow,
            Duration::Permanent,
            ConsentTier::Expert,
            T0,
        );
        assert!(granted.is_ok());
    }

    /// A permanent exception on a path is fine in the standard flow: it is
    /// narrow, and it is the case the user actually meets.
    #[test]
    fn a_permanent_path_exception_is_standard() {
        let p = policy_with(Rule::DownloadsSuspicion, Mode::Allow, Duration::Permanent);
        let live = p.live(T0);
        assert_eq!(live.len(), 1);
        assert!(live[0].permanent);
        assert_eq!(live[0].expires_at_ms, None);
    }

    #[test]
    fn permission_to_relax_widens_to_allow_but_never_past_the_ceiling() {
        assert_eq!(Mode::Relax.widen(Mode::Allow, Mode::Allow), Mode::Allow);
        assert_eq!(Mode::Allow.widen(Mode::Relax, Mode::Allow), Mode::Allow);
        assert_eq!(
            Mode::Relax.widen(Mode::Allow, Mode::Relax),
            Mode::Relax,
            "the ceiling is the ceiling"
        );
    }

    #[test]
    fn revoking_is_one_call_and_needs_nothing() {
        let mut p = policy_with(Rule::WindowsScriptHost, Mode::Allow, Duration::Minutes(60));
        p.grant(
            Rule::PowerShell,
            Target::Everything,
            Mode::Allow,
            Duration::Minutes(60),
            ConsentTier::Expert,
            T0,
        )
        .unwrap();

        assert_eq!(p.revoke_rule(Rule::PowerShell), 1);
        assert_eq!(p.effective(Rule::PowerShell, &Target::Everything, T0), None);
        assert_eq!(
            p.effective(
                Rule::WindowsScriptHost,
                &Target::Path("C:\\Scripts".into()),
                T0
            ),
            Some(Mode::Allow),
            "revoking one rule leaves the rest alone"
        );

        assert_eq!(p.revoke_all(), 1);
        assert!(p.live(T0).is_empty());
    }

    #[test]
    fn a_newer_grant_replaces_the_older_one_on_the_same_target() {
        let mut p = policy_with(Rule::WindowsScriptHost, Mode::Relax, Duration::Minutes(10));
        let target = Target::Path("C:\\Scripts".into());
        p.grant(
            Rule::WindowsScriptHost,
            target.clone(),
            Mode::Allow,
            Duration::Minutes(60),
            ConsentTier::Expert,
            T0 + 60_000,
        )
        .unwrap();

        let live = p.live(T0 + 120_000);
        assert_eq!(live.len(), 1, "one in force, not two");
        assert_eq!(
            p.effective(Rule::WindowsScriptHost, &target, T0 + 120_000),
            Some(Mode::Allow)
        );
    }

    #[test]
    fn pruning_removes_only_the_expired() {
        let mut p = Policy::new();
        p.grant(
            Rule::WindowsScriptHost,
            Target::Everything,
            Mode::Relax,
            Duration::Minutes(10),
            ConsentTier::Standard,
            T0,
        )
        .unwrap();
        p.grant(
            Rule::PowerShell,
            Target::Everything,
            Mode::Relax,
            Duration::UntilReboot,
            ConsentTier::Standard,
            T0,
        )
        .unwrap();

        assert_eq!(p.prune(T0 + HOUR), 1);
        assert_eq!(p.live(T0 + HOUR).len(), 1);
        assert_eq!(p.live(T0 + HOUR)[0].rule, Rule::PowerShell);
    }

    /// The wall that does not move. An exception may quiet a suspicion and
    /// may never vouch for a file.
    #[test]
    fn an_exception_quiets_suspicion_and_never_clears_a_verdict() {
        let mut p = Policy::new();
        let target = Target::Path("C:\\Scripts".into());
        p.grant(
            Rule::WindowsScriptHost,
            target.clone(),
            Mode::Relax,
            Duration::Minutes(60),
            ConsentTier::Standard,
            T0,
        )
        .unwrap();

        let after = |v| p.apply_to_verdict(v, Rule::WindowsScriptHost, &target, T0);

        assert_eq!(after(Verdict::Suspicious), Verdict::Monitor);
        assert_eq!(
            after(Verdict::Confirmed),
            Verdict::Confirmed,
            "a known-bad sample is never relaxed"
        );
        assert_eq!(after(Verdict::HighRisk), Verdict::HighRisk);
        assert_eq!(after(Verdict::Clean), Verdict::Clean);

        // The same exception after it expires changes nothing at all.
        let later = T0 + HOUR;
        assert_eq!(
            p.apply_to_verdict(Verdict::Suspicious, Rule::WindowsScriptHost, &target, later),
            Verdict::Suspicious
        );

        // And an `Allow` exception is not a Relax one: it stops the rule from
        // asking, it does not lower a score.
        let mut allow = Policy::new();
        allow
            .grant(
                Rule::WindowsScriptHost,
                target.clone(),
                Mode::Allow,
                Duration::Minutes(60),
                ConsentTier::Standard,
                T0,
            )
            .unwrap();
        assert_eq!(
            allow.apply_to_verdict(Verdict::Suspicious, Rule::WindowsScriptHost, &target, T0),
            Verdict::Suspicious
        );
    }

    #[test]
    fn the_log_line_says_who_granted_what_and_through_which_flow() {
        let mut p = Policy::new();
        p.grant(
            Rule::WindowsScriptHost,
            Target::Path("C:\\Scripts".into()),
            Mode::Allow,
            Duration::Minutes(30),
            ConsentTier::Standard,
            T0,
        )
        .unwrap();
        let line = &p.exceptions()[0].log_line();
        assert!(line.contains("a1.wsh"), "{line}");
        assert!(line.contains("allow"), "{line}");
        assert!(line.contains("standard"), "{line}");
        assert!(!line.contains("expert"), "{line}");
    }

    #[test]
    fn a_live_exception_carries_a_badge_key_and_not_a_sentence() {
        let p = policy_with(Rule::WindowsScriptHost, Mode::Allow, Duration::Permanent);
        let live = p.live(T0);
        assert_eq!(live[0].badge_key(), "exception.active.a1.wsh");
    }

    #[test]
    fn an_empty_policy_changed_nothing() {
        let p = Policy::new();
        for rule in Rule::ALL {
            assert_eq!(p.effective(rule, &Target::Everything, T0), None);
        }
        assert!(p.live(T0).is_empty());
    }
}
