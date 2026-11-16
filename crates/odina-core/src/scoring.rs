// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! The scoring model, implemented exactly as `docs/spec/scoring-spec-v1.md`
//! states it — including the caps and the vetoes, which are the parts that
//! matter more than the weights.
//!
//! The one rule this file exists to enforce: **without a hash match, a file
//! cannot be called a threat** (cap S1). Most false positives come from the
//! non-hash indicators — name, path, size — and a false positive does not
//! fail because it is wrong, it fails because it teaches the user to ignore
//! alerts.

/// One of the ten indicators, each normalised to 0.0..=1.0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Indicator {
    /// 1 — exact byte size (10%).
    Size,
    /// 2 — file name structure (8%).
    NamePattern,
    /// 3 — path structure (8%).
    PathPattern,
    /// 4 — file attributes (7%).
    Attributes,
    /// 5 — SHA-256 of the first 4 KB (15%).
    HeaderHash,
    /// 6 — SHA-256 of the last 4 KB (8%).
    TailHash,
    /// 7 — PE imphash (12%).
    PeHash,
    /// 8 — TLSH fuzzy distance (22%).
    Fuzzy,
    /// 9 — absence of a valid signature (5%).
    Unsigned,
    /// 10 — declared behaviour fingerprint (5%).
    Behaviour,
}

impl Indicator {
    /// Every indicator, in spec order, so a test can assert the weights cover
    /// exactly 100% without the list being written twice.
    pub const ALL: [Indicator; 10] = [
        Indicator::Size,
        Indicator::NamePattern,
        Indicator::PathPattern,
        Indicator::Attributes,
        Indicator::HeaderHash,
        Indicator::TailHash,
        Indicator::PeHash,
        Indicator::Fuzzy,
        Indicator::Unsigned,
        Indicator::Behaviour,
    ];

    /// Weight in percent, from the spec table in §2.
    pub const fn weight_percent(self) -> u8 {
        match self {
            Indicator::Size => 10,
            Indicator::NamePattern => 8,
            Indicator::PathPattern => 8,
            Indicator::Attributes => 7,
            Indicator::HeaderHash => 15,
            Indicator::TailHash => 8,
            Indicator::PeHash => 12,
            Indicator::Fuzzy => 22,
            Indicator::Unsigned => 5,
            Indicator::Behaviour => 5,
        }
    }

    /// True for the indicators that are actual content fingerprints. Cap S1
    /// and the "at least two independent positive signals" rule key off this.
    pub fn is_hash_indicator(self) -> bool {
        matches!(
            self,
            Indicator::HeaderHash | Indicator::TailHash | Indicator::PeHash
        )
    }

    /// §2 — the fuzzy hash is weighted like a hash indicator even though its
    /// value is a distance, not an equality. S1 names it explicitly.
    pub fn is_fingerprint(self) -> bool {
        self.is_hash_indicator() || matches!(self, Indicator::Fuzzy)
    }
}

/// A reason for refusing to score at all. Any veto means the score is 0 and
/// the user is never told anything about this file beyond the log entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Veto {
    /// V1 — hash is on the signed allow-list.
    AllowListed,
    /// V2 — this is Odina itself or one of its signed components.
    OdinaItself,
    /// V3 — signed Microsoft system file inside System32 / WinSxS (S3).
    SystemFile,
    /// V4 — the user's own explicit exception: logged, not hidden, not
    /// forgotten. If the machine is later hit by an excluded file, the log
    /// says so plainly — and does not blame the user.
    UserException,
    /// V5 — **the other direction**: the file's SHA-256 is in the known-bad
    /// database. A byte-exact match is not evidence to weigh, it is the
    /// answer. Score becomes 1.0 and the verdict is Confirmed.
    ///
    /// This veto exists because writing the weights exposed a hole in them:
    /// a known-bad file whose name and path are unremarkable scores 15
    /// (header) + 8 (tail) + 10 (size) = **33%**, i.e. "monitor", i.e.
    /// **silence**. The caps in §4 push scores *down*, and nothing pushed a
    /// known sample *up*. V5 is that push.
    KnownBad,
}

/// The four verdicts the UI knows about. Below [`Verdict::Monitor`] the user is
/// never told anything: showing sub-threshold scores is noise, and noise costs
/// more trust than it buys safety.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Verdict {
    /// < 0.20 — pass silently.
    Clean,
    /// 0.20 .. 0.499 — log in the ring buffer, never shown.
    Monitor,
    /// 0.50 .. 0.799 — temporary block, deep scan, two buttons, 30 s.
    Suspicious,
    /// 0.80 .. 0.989 — stop now, mandatory behaviour check, 60 s.
    HighRisk,
    /// >= 0.99 — isolate, one button, cannot be dismissed.
    Confirmed,
}

impl Verdict {
    /// Whether this verdict reaches the user at all.
    pub fn is_user_visible(self) -> bool {
        self >= Verdict::Suspicious
    }

    /// What happens when the alert times out. Never "allow" — a timeout is an
    /// absence of a decision, and absence of a decision must not run the file.
    pub fn timeout_action(self) -> &'static str {
        match self {
            Verdict::Confirmed => "quarantine",
            Verdict::HighRisk | Verdict::Suspicious => "block",
            Verdict::Monitor | Verdict::Clean => "none",
        }
    }
}

/// Thresholds, in one place so the spec and the code cannot drift apart.
#[derive(Debug, Clone, Copy)]
pub struct Thresholds {
    pub monitor: f32,
    pub suspicious: f32,
    pub high_risk: f32,
    pub confirmed: f32,
    /// §6.1 — the caution band. Inside it the stricter verdict is applied.
    pub band_low: f32,
    /// §6.1 upper edge, exclusive.
    pub band_high: f32,
}

impl Default for Thresholds {
    fn default() -> Self {
        Thresholds {
            monitor: 0.20,
            suspicious: 0.50,
            high_risk: 0.80,
            confirmed: 0.99,
            band_low: 0.75,
            band_high: 0.85,
        }
    }
}

/// Which cap limited the result, so a log line can explain the number later.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cap {
    /// S1 — no fingerprint match anywhere.
    NoFingerprint,
    /// S2 — valid signature from a known publisher.
    SignedKnownPublisher,
    /// S4 — the file is in the clean corpus, so it is a known good.
    CleanCorpus,
}

impl Cap {
    pub fn label(self) -> &'static str {
        match self {
            Cap::NoFingerprint => "S1",
            Cap::SignedKnownPublisher => "S2",
            Cap::CleanCorpus => "S4",
        }
    }
}

/// The outcome of scoring: the number, the verdict, and why.
#[derive(Debug, Clone, PartialEq)]
pub struct Score {
    /// Final score after caps and vetoes.
    pub value: f32,
    pub verdict: Verdict,
    /// Set when a veto forced the score to zero.
    pub veto: Option<Veto>,
    /// Every cap that lowered the result, in spec order.
    pub caps: Vec<Cap>,
}

impl Score {
    /// True when the verdict may interrupt the user.
    pub fn interrupts(&self) -> bool {
        self.verdict.is_user_visible()
    }
}

/// Indicator values plus the three pieces of context that trigger caps.
#[derive(Debug, Clone, Default)]
pub struct Evidence {
    /// (indicator, value in 0.0..=1.0). A missing indicator counts as zero,
    /// which is the honest default: no evidence is not evidence.
    pub values: Vec<(Indicator, f32)>,
    /// A valid certificate from a publisher we already know.
    pub signed_by_known_publisher: bool,
    /// The hash appears in the clean corpus (Golden Set). Protects against a
    /// known false positive repeating (S4).
    pub in_clean_corpus: bool,
}

impl Evidence {
    pub fn new() -> Self {
        Evidence::default()
    }

    pub fn with(mut self, indicator: Indicator, value: f32) -> Self {
        self.set(indicator, value);
        self
    }

    pub fn set(&mut self, indicator: Indicator, value: f32) {
        let clamped = value.clamp(0.0, 1.0);
        if let Some(slot) = self.values.iter_mut().find(|(i, _)| *i == indicator) {
            slot.1 = clamped;
        } else {
            self.values.push((indicator, clamped));
        }
    }

    pub fn get(&self, indicator: Indicator) -> f32 {
        self.values
            .iter()
            .find(|(i, _)| *i == indicator)
            .map(|(_, v)| *v)
            .unwrap_or(0.0)
    }

    /// Any positive value on a fingerprint indicator.
    pub fn has_any_fingerprint_match(&self) -> bool {
        Indicator::ALL
            .iter()
            .filter(|i| i.is_fingerprint())
            .any(|i| self.get(*i) > 0.0)
    }

    /// §7 of the spec: a high verdict needs at least two independent positive
    /// signals, not one strong guess. Counts indicators above 0.5.
    pub fn positive_signal_count(&self) -> usize {
        self.values.iter().filter(|(_, v)| *v > 0.5).count()
    }
}

/// Score a file. Pure function: same evidence, same answer, always.
pub fn score(evidence: &Evidence, thresholds: &Thresholds, veto: Option<Veto>) -> Score {
    if let Some(v) = veto {
        return Score {
            value: 0.0,
            verdict: Verdict::Clean,
            veto: Some(v),
            caps: Vec::new(),
        };
    }

    let mut total = 0.0f32;
    for indicator in Indicator::ALL {
        total += evidence.get(indicator) * (indicator.weight_percent() as f32 / 100.0);
    }

    let mut caps = Vec::new();

    // S1 — the cap that matters. Without a fingerprint the score cannot pass
    // 0.49, so name/path/size alone never reach "suspicious".
    if !evidence.has_any_fingerprint_match() && total > 0.49 {
        total = 0.49;
        caps.push(Cap::NoFingerprint);
    }

    // S2 — a valid certificate from a known publisher caps the score; the
    // file is reviewed by a human, never decided by the engine.
    if evidence.signed_by_known_publisher && total > 0.30 {
        total = 0.30;
        caps.push(Cap::SignedKnownPublisher);
    }

    // S4 — a known good stays known good, even if this run scored it high.
    if evidence.in_clean_corpus && total > 0.20 {
        total = 0.20;
        caps.push(Cap::CleanCorpus);
    }

    Score {
        value: total,
        verdict: banded_verdict(total, thresholds),
        veto: None,
        caps,
    }
}

/// Score with the known-bad database consulted last, as it must be: an
/// allow-list veto (V1–V4) may clear a file, but a byte-exact match against a
/// known-bad sample always wins over a heuristic allow-list, because a signed
/// allow-list is a list of hashes and a hash is not a matter of opinion.
///
/// Conflicting lists are a configuration bug, not a judgement call — this
/// function resolves it in the safe direction and the caller is expected to
/// log it.
pub fn score_with_known_bad(
    evidence: &Evidence,
    thresholds: &Thresholds,
    veto: Option<Veto>,
    known_bad: bool,
) -> Score {
    if known_bad {
        return Score {
            value: 1.0,
            verdict: Verdict::Confirmed,
            veto: Some(Veto::KnownBad),
            caps: Vec::new(),
        };
    }
    score(evidence, thresholds, veto)
}

/// Map a score to its verdict, honouring the 0.75–0.85 caution band from
/// §6.1: inside the band the stricter behaviour wins, so an error falls toward
/// protection rather than toward letting something run.
pub fn banded_verdict(value: f32, thresholds: &Thresholds) -> Verdict {
    let in_band = value >= thresholds.band_low && value < thresholds.band_high;
    if value >= thresholds.confirmed {
        Verdict::Confirmed
    } else if value >= thresholds.high_risk || (in_band && value >= thresholds.suspicious) {
        Verdict::HighRisk
    } else if value >= thresholds.suspicious {
        Verdict::Suspicious
    } else if value >= thresholds.monitor {
        Verdict::Monitor
    } else {
        Verdict::Clean
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_maxed() -> Evidence {
        let mut e = Evidence::new();
        for i in Indicator::ALL {
            e.set(i, 1.0);
        }
        e
    }

    #[test]
    fn weights_sum_to_exactly_one_hundred() {
        let sum: u32 = Indicator::ALL
            .iter()
            .map(|i| i.weight_percent() as u32)
            .sum();
        assert_eq!(
            sum, 100,
            "the spec promises 100%; a change must redistribute"
        );
    }

    #[test]
    fn all_indicators_present_and_unique() {
        let mut seen = std::collections::HashSet::new();
        for i in Indicator::ALL {
            assert!(seen.insert(i), "duplicate indicator in ALL");
        }
        assert_eq!(seen.len(), 10);
    }

    #[test]
    fn no_evidence_scores_zero() {
        let s = score(&Evidence::new(), &Thresholds::default(), None);
        assert_eq!(s.value, 0.0);
        assert_eq!(s.verdict, Verdict::Clean);
        assert!(!s.interrupts());
    }

    #[test]
    fn everything_maxed_scores_one() {
        let s = score(&all_maxed(), &Thresholds::default(), None);
        assert!((s.value - 1.0).abs() < 1e-6);
        assert_eq!(s.verdict, Verdict::Confirmed);
        assert!(s.caps.is_empty());
    }

    #[test]
    fn a_name_only_verdict_can_never_reach_suspicious() {
        // Every non-fingerprint indicator maxed at once. This is the worst case
        // an attacker controls from the outside (name, path, size, attributes,
        // missing signature, declared behaviour) and it must stay below the
        // 0.50 line on its own.
        //
        // Note the arithmetic: 10 + 8 + 8 + 7 + 5 + 5 = 43%. So with today's
        // weights cap S1 can never fire — it is a *floor guarantee*, not a
        // clamp that saves us now. It is kept because it makes that property
        // hold through any future re-weighting, which is exactly when something
        // like it would be forgotten.
        let mut e = Evidence::new();
        e.set(Indicator::Size, 1.0);
        e.set(Indicator::NamePattern, 1.0);
        e.set(Indicator::PathPattern, 1.0);
        e.set(Indicator::Attributes, 1.0);
        e.set(Indicator::Unsigned, 1.0);
        e.set(Indicator::Behaviour, 1.0);
        let s = score(&e, &Thresholds::default(), None);
        assert!((s.value - 0.43).abs() < 1e-6, "got {}", s.value);
        assert!(s.value <= 0.49);
        assert!(s.caps.is_empty(), "S1 has nothing to clamp at 43%");
        assert!(!s.interrupts(), "no fingerprint means no alert");
    }

    #[test]
    fn fingerprints_alone_score_fifty_seven_percent() {
        // 15 (head) + 8 (tail) + 12 (imphash) + 22 (TLSH) = 57%.
        let mut e = all_maxed();
        for i in Indicator::ALL {
            if !i.is_fingerprint() {
                e.set(i, 0.0);
            }
        }
        let s = score(&e, &Thresholds::default(), None);
        assert!((s.value - 0.57).abs() < 1e-6, "got {}", s.value);
        assert!(s.caps.is_empty());
        assert_eq!(s.verdict, Verdict::Suspicious);
    }

    #[test]
    fn fuzzy_alone_counts_as_a_fingerprint() {
        let e = Evidence::new().with(Indicator::Fuzzy, 0.95);
        let s = score(&e, &Thresholds::default(), None);
        assert!(s.caps.is_empty(), "TLSH is named in S1 explicitly");
        assert!(s.value > 0.19);
    }

    #[test]
    fn signed_publisher_caps_to_monitor() {
        let mut e = all_maxed();
        e.signed_by_known_publisher = true;
        let s = score(&e, &Thresholds::default(), None);
        assert!(s.value <= 0.30 + f32::EPSILON);
        assert!(s.caps.contains(&Cap::SignedKnownPublisher));
        assert!(!s.interrupts());
    }

    #[test]
    fn clean_corpus_is_capped_to_monitor() {
        let mut e = all_maxed();
        e.in_clean_corpus = true;
        let s = score(&e, &Thresholds::default(), None);
        assert!(s.value <= 0.20 + f32::EPSILON);
        assert!(s.caps.contains(&Cap::CleanCorpus));
    }

    #[test]
    fn veto_beats_everything() {
        let s = score(
            &all_maxed(),
            &Thresholds::default(),
            Some(Veto::AllowListed),
        );
        assert_eq!(s.value, 0.0);
        assert_eq!(s.verdict, Verdict::Clean);
        assert_eq!(s.veto, Some(Veto::AllowListed));
        assert!(s.caps.is_empty());
    }

    #[test]
    fn verdict_boundaries_are_inclusive_at_the_bottom() {
        let t = Thresholds::default();
        assert_eq!(banded_verdict(0.199, &t), Verdict::Clean);
        assert_eq!(banded_verdict(0.20, &t), Verdict::Monitor);
        assert_eq!(banded_verdict(0.50, &t), Verdict::Suspicious);
        assert_eq!(banded_verdict(0.80, &t), Verdict::HighRisk);
        assert_eq!(banded_verdict(0.989, &t), Verdict::HighRisk);
        assert_eq!(banded_verdict(0.99, &t), Verdict::Confirmed);
    }

    #[test]
    fn caution_band_pushes_upward_only() {
        let t = Thresholds::default();
        // Inside the band: stricter verdict, so 0.76 is treated as 0.80.
        assert_eq!(banded_verdict(0.76, &t), Verdict::HighRisk);
        assert_eq!(banded_verdict(0.75, &t), Verdict::HighRisk);
        // Below the band: nothing is raised.
        assert_eq!(banded_verdict(0.74, &t), Verdict::Suspicious);
        // Above the band: nothing is softened.
        assert_eq!(banded_verdict(0.86, &t), Verdict::HighRisk);
        assert_eq!(banded_verdict(0.99, &t), Verdict::Confirmed);
    }

    #[test]
    fn timeout_never_allows() {
        for v in [
            Verdict::Clean,
            Verdict::Monitor,
            Verdict::Suspicious,
            Verdict::HighRisk,
            Verdict::Confirmed,
        ] {
            assert_ne!(v.timeout_action(), "allow");
        }
        assert_eq!(Verdict::Confirmed.timeout_action(), "quarantine");
    }

    #[test]
    fn values_outside_range_are_clamped() {
        let mut e = Evidence::new();
        e.set(Indicator::Fuzzy, 5.0);
        e.set(Indicator::Size, -3.0);
        assert_eq!(e.get(Indicator::Fuzzy), 1.0);
        assert_eq!(e.get(Indicator::Size), 0.0);
    }

    #[test]
    fn a_known_bad_hash_is_a_verdict_not_a_weight() {
        // The hole this test was written to close: an unremarkable name and
        // path over bytes that exactly match a known-bad sample.
        let mut e = Evidence::new();
        e.set(Indicator::HeaderHash, 1.0);
        e.set(Indicator::TailHash, 1.0);
        e.set(Indicator::Size, 1.0);
        let weighted = score(&e, &Thresholds::default(), None);
        assert!(
            (weighted.value - 0.33).abs() < 1e-6,
            "got {}",
            weighted.value
        );
        assert!(!weighted.interrupts(), "this is the silent hole");

        let s = score_with_known_bad(&e, &Thresholds::default(), None, true);
        assert_eq!(s.value, 1.0);
        assert_eq!(s.verdict, Verdict::Confirmed);
        assert_eq!(s.veto, Some(Veto::KnownBad));
        assert!(s.interrupts());
        assert_eq!(s.verdict.timeout_action(), "quarantine");
    }

    #[test]
    fn known_bad_beats_a_heuristic_allow_list() {
        let s = score_with_known_bad(
            &all_maxed(),
            &Thresholds::default(),
            Some(Veto::UserException),
            true,
        );
        assert_eq!(s.verdict, Verdict::Confirmed, "a hash is not an opinion");
    }

    #[test]
    fn known_bad_false_changes_nothing() {
        let s = score_with_known_bad(&Evidence::new(), &Thresholds::default(), None, false);
        assert_eq!(s.value, 0.0);
        assert_eq!(s.verdict, Verdict::Clean);
    }

    #[test]
    fn positive_signal_count_ignores_weak_values() {
        let mut e = Evidence::new();
        e.set(Indicator::Fuzzy, 0.9);
        e.set(Indicator::Size, 0.5);
        e.set(Indicator::PathPattern, 0.6);
        assert_eq!(e.positive_signal_count(), 2);
    }
}
