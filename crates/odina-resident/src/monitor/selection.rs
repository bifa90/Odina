// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! Browser view of the published table. No new inventory, paths opened, or disk writes.
use super::{Record, State};
use crate::server::UNKNOWN;
use std::{
    collections::{hash_map::RandomState, VecDeque},
    hash::BuildHasher,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Revision(u64, u64, u64); // epoch, inventory, last event
struct Grant {
    id: u64,
    issued: Instant,
    selection: PathBuf,
    file: PathBuf,
    rev: Revision,
}
#[derive(Default)]
pub(super) struct Grants {
    entries: VecDeque<Grant>,
    secret: RandomState,
    counter: u64,
}
impl State {
    fn revision(&self, row: &Record) -> Revision {
        Revision(self.epoch, self.full_scans, row.event.map_or(0, |e| e.seq))
    }
    pub fn still_suspect(&self, key: &Path, rev: Revision) -> bool {
        self.covered
            && self.rows.get(key).is_some_and(|r| {
                r.value
                    .is_some_and(|v| matches!(v.band, "suspicious" | "high_risk"))
                    && self.revision(r) == rev
            })
    }
    /// Exact file: report its classification. Directory/drive: report a known
    /// positive descendant, NEVER infer a clean directory from file-only rows.
    pub fn inspect(&mut self, key: &Path) -> String {
        if !self.covered {
            return UNKNOWN.into();
        }
        let (file, row) = if let Some(r) = self.rows.get(key).filter(|r| r.value.is_some()) {
            match r.value.map(|v| v.band) {
                Some("confirmed") => return r#"{"result":"threat"}"#.into(),
                Some("clean") => return r#"{"result":"clean"}"#.into(),
                Some("suspicious" | "high_risk") => (key, r),
                _ => return UNKNOWN.into(),
            }
        } else {
            let mut suspect: Option<(&Path, &Record)> = None;
            for (k, r) in &self.rows {
                if !k.starts_with(key) {
                    continue;
                }
                match r.value.map(|v| v.band) {
                    Some("confirmed") => return r#"{"result":"threat"}"#.into(),
                    Some("suspicious" | "high_risk")
                        if suspect.map_or(true, |(p, _)| k.as_path() < p) =>
                    {
                        suspect = Some((k, r))
                    }
                    _ => (),
                }
            }
            let Some(s) = suspect else {
                return UNKNOWN.into();
            };
            s
        };
        let rev = self.revision(row);
        let g = &mut self.grants;
        if g.entries.len() == 16 {
            g.entries.pop_front();
        }
        g.counter = g.counter.wrapping_add(1);
        let id = g.secret.hash_one(g.counter);
        g.entries.push_back(Grant {
            id,
            issued: Instant::now(),
            selection: key.into(),
            file: file.into(),
            rev,
        });
        format!(r#"{{"result":"suspicious","ticket":"{id:016x}"}}"#)
    }
    pub fn consume(&mut self, key: &Path, id: u64) -> Option<(PathBuf, Revision)> {
        let i = self
            .grants
            .entries
            .iter()
            .position(|g| g.selection == key && g.id == id)?;
        let g = self.grants.entries.remove(i)?;
        (g.issued.elapsed() < Duration::from_secs(120) && self.still_suspect(&g.file, g.rev))
            .then_some((g.file, g.rev))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::FileVerdict;
    fn grant(s: &mut State, k: &Path) -> u64 {
        s.inspect(k);
        s.grants.entries.back().unwrap().id
    }
    #[test]
    fn grants_are_scoped_single_use_and_do_not_survive_changes_or_gaps() {
        let mut s = State {
            covered: true,
            ..Default::default()
        };
        let dir = Path::new("selected");
        let p = dir.join("file");
        s.seed(
            0,
            &p,
            FileVerdict {
                band: "high_risk",
                bytes: 42,
            },
        );
        let id = grant(&mut s, dir);
        assert!(s.consume(&p, id).is_none());
        assert_eq!(s.consume(dir, id).unwrap().0, p);
        assert!(s.consume(dir, id).is_none());
        let id = grant(&mut s, dir);
        s.grants.entries.back_mut().unwrap().issued -= Duration::from_secs(121);
        assert!(s.consume(dir, id).is_none());
        let id = grant(&mut s, dir);
        s.change(p.clone(), false, 0);
        assert!(s.consume(dir, id).is_none());
        assert_eq!(s.inspect(dir), UNKNOWN);
        s.invalidate_scope();
        assert_eq!(s.inspect(&p), UNKNOWN);
    }
    #[test]
    fn directory_positives_are_component_scoped_and_no_file_rows_prove_a_clean_disk() {
        let mut s = State {
            covered: true,
            ..Default::default()
        };
        for (p, band) in [
            ("ab/d", "confirmed"),
            ("a/f", "clean"),
            ("b/z", "suspicious"),
            ("b/a", "suspicious"),
        ] {
            s.seed(0, Path::new(p), FileVerdict { band, bytes: 1 });
        }
        // A witnessed deletion of a former FILE named b is not evidence about
        // a subsequently created directory b/ containing current positive rows.
        s.change("b".into(), true, 0);
        if let Some(super::super::Task::One(epoch, k, e)) = s.next() {
            s.publish(epoch, &k, e, None);
        } else {
            panic!("deletion not queued");
        }
        assert_eq!(s.inspect(Path::new("a")), UNKNOWN);
        assert_eq!(s.inspect(Path::new("a/f")), r#"{"result":"clean"}"#);
        let id = grant(&mut s, Path::new("b"));
        assert_eq!(s.consume(Path::new("b"), id).unwrap().0, Path::new("b/a"));
        assert_eq!(s.inspect(Path::new("")), r#"{"result":"threat"}"#);
    }
}
