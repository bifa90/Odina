// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! K26 Browser Download Consent Gate Contract.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsentReceipt {
    pub consumed: bool,
}

impl ConsentReceipt {
    pub fn consume(&mut self) -> bool {
        if self.consumed {
            false
        } else {
            self.consumed = true;
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receipt_is_single_use() {
        let mut r = ConsentReceipt { consumed: false };
        assert!(r.consume());
        assert!(!r.consume());
    }
}
