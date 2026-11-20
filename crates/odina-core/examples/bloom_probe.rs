// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! B3 sizing experiment, not part of the scanner. Keys are WHOLE digests:
//! a negative saves an exact DB lookup, not reading/hashing the input file.
//! Run: cargo run --release --locked -p odina-core --example bloom_probe

use sha2::{Digest, Sha256};

type Key = [u8; 32];
const KEYS: u64 = 500_000;
const NEGATIVES: u64 = 1_000_000;
const BITS: usize = KEYS as usize * 15;
const PROBES: u64 = 10;

fn key(i: u64) -> Key {
    Sha256::digest(i.to_le_bytes()).into()
}

// Two independent parts of an already uniformly distributed SHA-256 key.
fn positions(key: &Key) -> impl Iterator<Item = usize> {
    let a = u64::from_le_bytes(key[..8].try_into().unwrap()) % BITS as u64;
    let b = (u64::from_le_bytes(key[8..16].try_into().unwrap()) | 1) % BITS as u64;
    (0..PROBES).map(move |i| ((a + i * b) % BITS as u64) as usize)
}

fn maybe(bits: &[u8], key: &Key) -> bool {
    positions(key).all(|p| bits[p / 8] & (1 << (p % 8)) != 0)
}

fn main() {
    let mut exact: Vec<Key> = (0..KEYS).map(key).collect();
    exact.sort_unstable();
    let mut bits = vec![0u8; BITS.div_ceil(8)];
    for k in &exact {
        for p in positions(k) {
            bits[p / 8] |= 1 << (p % 8);
        }
    }
    assert!(
        exact.iter().all(|k| maybe(&bits, k)),
        "lost an inserted key"
    );
    let mut false_positives = 0u64;
    for i in KEYS..KEYS + NEGATIVES {
        let k = key(i);
        if maybe(&bits, &k) {
            // A positive is only a candidate, never a verdict.
            assert!(exact.binary_search(&k).is_err());
            false_positives += 1;
        }
    }
    println!("keys={KEYS} filter_bytes={} probes={PROBES} missing=0 negatives={NEGATIVES} false_positives={false_positives}", bits.len());
    // Sample FPR on these disjoint synthetic inputs only; not scanner FPR.
    assert!(false_positives * 1000 < NEGATIVES);
}
