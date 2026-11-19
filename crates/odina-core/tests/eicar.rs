// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! The one test file that proves the pipeline end to end, using the standard
//! antivirus test file.
//!
//! ## What EICAR is, and why it is here
//!
//! EICAR is a **68-byte harmless string** published by the European Institute
//! for Computer Antivirus Research so that people can check whether a scanner
//! reacts *without* handling real malware. It contains no code, does nothing,
//! and is not a virus. Every commercial antivirus detects it by agreement.
//!
//! We hash it and assert the digest is the well-known value. That proves, in
//! one assertion, that: the reader works, the head/tail windows work, the hex
//! encoding is right, and the full digest matches an authoritative source. If
//! this test ever passes a *different* number, the hashing code is broken.
//!
//! ## Why the string is assembled instead of written literally
//!
//! So that this source file does not itself look like the test file to a
//! scanner, a mail gateway, or a repository secret scanner in between. Same
//! bytes, no false alarms on people just reading the repo.

use odina_core::hashing::{hash_bytes, hash_reader};
use odina_core::{score, Evidence, Indicator, Thresholds, Verdict, Veto};

/// The EICAR test string, built at runtime from harmless pieces.
fn eicar() -> Vec<u8> {
    let parts = [
        "X5O!P%@AP[4\\PZX54(P^)7CC)7}$",
        "EICAR-STANDARD-ANTIVIRUS-",
        "TEST-FILE!$H+H*",
    ];
    parts.concat().into_bytes()
}

/// SHA-256 of the EICAR test string — the published, stable value.
const EICAR_SHA256: &str = "275a021bbfb6489e54d471899f7db9d1663fc695ec2fe2a2c4538aabf651fd0f";

#[test]
fn eicar_string_is_the_expected_length() {
    assert_eq!(eicar().len(), 68, "EICAR is exactly 68 bytes by definition");
}

#[test]
fn eicar_full_digest_matches_the_published_vector() {
    let h = hash_bytes(&eicar());
    assert_eq!(h.full, EICAR_SHA256);
    assert_eq!(h.size, 68);
}

#[test]
fn eicar_is_short_so_head_and_tail_are_the_full_file() {
    let h = hash_bytes(&eicar());
    // 68 bytes < 4096, so both windows see the whole file.
    assert_eq!(h.head, EICAR_SHA256);
    assert_eq!(h.tail, EICAR_SHA256);
}

#[test]
fn eicar_through_the_streaming_reader_is_identical() {
    let data = eicar();
    let from_mem = hash_bytes(&data);
    let from_reader = hash_reader(&data[..], None).expect("reader");
    assert_eq!(from_mem.full, from_reader.full);
    assert_eq!(from_mem.head, from_reader.head);
    assert_eq!(from_mem.tail, from_reader.tail);
    assert_eq!(from_mem.size, from_reader.size);
}

#[test]
fn g1_requires_the_exact_full_digest_even_if_both_windows_match() {
    use odina_core::g1_reference::matches;

    let mut bytes = eicar();
    assert!(matches(&hash_reader(&bytes[..], Some(68)).unwrap()));
    let capped = hash_reader(&bytes[..], Some(67)).unwrap();
    assert!(capped.full_digest_skipped());
    assert_eq!(capped.head, EICAR_SHA256);
    assert_eq!(capped.tail, EICAR_SHA256);
    assert!(!matches(&capped));
    bytes[0] = b'Y'; // same size, different full digest
    assert!(!matches(&hash_bytes(&bytes)));
}

#[test]
fn a_known_sample_without_a_fingerprint_match_is_not_reported_as_a_threat() {
    // This is the honest case: recognising EICAR is a *database* lookup, not a
    // guess from the name. With no fingerprint in the evidence the engine must
    // stay quiet — that is cap S1 doing its job, and it is what keeps a
    // file called `invoice.exe` from becoming an alert.
    let e = Evidence::new()
        .with(Indicator::NamePattern, 0.9)
        .with(Indicator::PathPattern, 0.7)
        .with(Indicator::Unsigned, 1.0);
    let s = score(&e, &Thresholds::default(), None);
    // 8% x 0.9 + 8% x 0.7 + 5% x 1.0 = 17.8% — under the 20% monitor line.
    assert!(s.value < 0.20, "got {}", s.value);
    assert_eq!(s.verdict, Verdict::Clean, "say nothing at all");
    assert!(!s.interrupts(), "no fingerprint means no alert");
}

#[test]
fn a_sample_with_a_fingerprint_and_a_second_signal_does_interrupt() {
    let mut e = Evidence::new();
    e.set(Indicator::HeaderHash, 1.0);
    e.set(Indicator::TailHash, 1.0);
    e.set(Indicator::PeHash, 1.0);
    e.set(Indicator::Fuzzy, 1.0);
    e.set(Indicator::Size, 1.0);
    e.set(Indicator::NamePattern, 1.0);
    e.set(Indicator::PathPattern, 1.0);
    let s = score(&e, &Thresholds::default(), None);
    // 57% fingerprints + 10 + 8 + 8 = 83%.
    assert!((s.value - 0.83).abs() < 1e-6, "got {}", s.value);
    assert!(s.value >= 0.80, "got {}", s.value);
    assert_eq!(s.verdict, Verdict::HighRisk);
    assert!(s.interrupts());
    assert_eq!(s.verdict.timeout_action(), "block");
}

#[test]
fn eicar_on_the_allow_list_is_silent() {
    let e = Evidence::new().with(Indicator::Fuzzy, 1.0);
    let s = score(&e, &Thresholds::default(), Some(Veto::AllowListed));
    assert_eq!(s.value, 0.0);
    assert_eq!(s.verdict, Verdict::Clean);
    assert!(!s.interrupts());
}
