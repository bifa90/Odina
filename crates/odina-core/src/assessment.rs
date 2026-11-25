// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! One decision from path features and supplied content evidence. No I/O.
use crate::{
    g1_reference, guess_path_flags, name_shape, path_score, score_with_known_bad, signature_db,
    Evidence, FileHashes, Indicator, Score, Thresholds,
};
use std::path::{Path, PathBuf};

pub struct Assessment {
    pub path: PathBuf,
    pub hashes: FileHashes,
    pub known_bad: bool,
    pub score: Score,
    pub name_score: f32,
    pub path_score: f32,
}

/// In this catalog only V5 consumes content. If another indicator needs
/// bytes, request its evidence in the file scanner before any early return.
pub fn assess(path: &Path, hashes: FileHashes) -> Assessment {
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let name_score = name_shape(&file_name).score();
    let path_flags = guess_path_flags(path);
    let path_score = path_score(path_flags);

    let mut evidence = Evidence::new();
    evidence.set(Indicator::NamePattern, name_score);
    evidence.set(Indicator::PathPattern, path_score);
    // Indicators 5-8 and 10 need a database or a parser that does not exist
    // yet. They stay at zero — an unmeasured indicator is not evidence.

    // Two exact-digest sources, one verdict: the pinned K44 vector and the seed
    // catalogue (D104). The seed only ever warns; "protected" needs a signed
    // full catalogue, so a seed hit here can raise a score but never bless a
    // machine. Neither source is coverage on its own.
    let known_bad = g1_reference::matches(&hashes) || signature_db::seed_matches(&hashes);

    let score = score_with_known_bad(&evidence, &Thresholds::default(), None, known_bad);

    Assessment {
        path: path.to_path_buf(),
        hashes,
        known_bad,
        score,
        name_score,
        path_score,
    }
}
