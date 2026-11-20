// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! The watcher invalidates synchronously; only the worker reads file content.
//! The authoritative table is volatile, distinct from any persistent hash hints.
use crate::{watch::now_ns, wire::Channel, FileVerdict};
use std::{
    collections::{HashMap, VecDeque},
    path::{Path, PathBuf},
    sync::{Condvar, Mutex},
};

const QUEUE: usize = 4096; // distinct pending paths; repeats replace, never extend the queue
const ROWS: usize = 65_536;
const TOMBSTONES: usize = 1024;
#[derive(Clone, Copy)]
pub(crate) struct Event {
    pub remove: bool,
    seq: u64,
    received: u64,
    invalidated: u64,
}
struct Record {
    value: Option<FileVerdict>, // None = witnessed deletion, never a clean verdict
    event: Option<Event>,
    published: u64,
}
#[derive(Default)]
pub(crate) struct State {
    pub stopping: bool,
    pub manual: Option<Option<Channel>>,
    pub manual_busy: bool,
    pub full_busy: bool,
    pub completed: u64, // completed MANUAL requests, as in IPC v1
    pub last: Option<u8>,
    terminal: bool, // unrecoverable native error: explicit restart after correcting the cause
    armed: bool,    // native subscription (not sufficient for coverage)
    need_full: bool,
    baseline: bool, // complete inventory in this epoch
    fenced: bool,   // kernel stream drained through the end-of-inventory wake
    covered: bool,  // also replayed changes from the inventory window
    epoch: u64,
    seq: u64,
    full_scans: u64,
    delta_scans: u64,
    delta_errors: u64,
    last_path: PathBuf,
    same_path_run: u64,
    loss_incidents: u64,
    queue_overflows: u64,
    active_delta: bool,
    pending: HashMap<PathBuf, Event>,
    order: VecDeque<PathBuf>,
    rows: HashMap<PathBuf, Record>,
    retired: VecDeque<(PathBuf, u64)>,
}
pub(crate) enum Task {
    Full(u64, Option<Option<Channel>>),
    One(u64, PathBuf, Event),
}
impl State {
    fn invalidate_scope(&mut self) {
        self.loss_incidents += 1;
        self.epoch += 1;
        self.fenced = false;
        self.need_full = true;
        self.baseline = false;
        self.covered = false;
        self.pending.clear();
        self.order.clear();
        self.rows.clear();
        self.retired.clear();
    }
    fn change(&mut self, path: PathBuf, remove: bool, received: u64) {
        if self.stopping {
            return;
        }
        if self.pending.len() >= QUEUE && !self.pending.contains_key(&path) {
            self.queue_overflows += 1;
            self.invalidate_scope(); // discard only behind an epoch fence + mandatory inventory
        }
        self.seq += 1;
        if self.last_path == path {
            self.same_path_run += 1;
        } else {
            self.last_path.clone_from(&path);
            self.same_path_run = 1;
        }
        self.rows.remove(&path); // old verdict is no longer queryable, even if the scanner is busy
        let event = Event {
            remove,
            seq: self.seq,
            received,
            invalidated: now_ns(),
        };
        if self.pending.insert(path.clone(), event).is_none() {
            self.order.push_back(path);
        }
    }
    pub fn next(&mut self) -> Option<Task> {
        if self.manual.is_some() || (self.armed && self.need_full) {
            self.need_full = false;
            self.fenced = false;
            self.full_busy = true;
            self.full_scans += 1;
            self.covered = false;
            self.baseline = false;
            self.rows.clear();
            self.retired.clear();
            return Some(Task::Full(self.epoch, self.manual.take()));
        }
        let path = self.order.pop_front()?;
        let event = self
            .pending
            .remove(&path)
            .expect("one slot per queued path");
        self.active_delta = true;
        self.delta_scans += u64::from(!event.remove);
        Some(Task::One(self.epoch, path, event))
    }
    fn insert(&mut self, path: &Path, r: Record) {
        if self.rows.len() >= ROWS && !self.rows.contains_key(path) {
            // Refuse partial coverage. Keep bounds; don't trigger an endless re-inventory.
            self.baseline = false;
            self.covered = false;
        } else {
            self.rows.insert(path.to_path_buf(), r);
        }
    }
    pub fn seed(&mut self, epoch: u64, path: &Path, v: FileVerdict) -> bool {
        if self.epoch == epoch && !self.pending.contains_key(path) {
            if self.rows.len() >= ROWS {
                return false;
            }
            self.insert(
                path,
                Record {
                    value: Some(v),
                    event: None,
                    published: 0,
                },
            );
        }
        true
    }
    pub fn end_full(&mut self, epoch: u64, enumerated: bool) {
        self.full_busy = false;
        if self.armed && self.epoch == epoch {
            self.baseline = enumerated;
        }
        self.settle();
    }
    fn settle(&mut self) {
        if self.armed
            && self.baseline
            && self.fenced
            && !self.need_full
            && self.pending.is_empty()
            && !self.active_delta
        {
            self.covered = true;
        }
    }
    pub fn publish(&mut self, epoch: u64, path: &Path, e: Event, value: Option<FileVerdict>) {
        self.active_delta = false;
        if self.armed && self.epoch == epoch && !self.pending.contains_key(path) {
            if e.remove || value.is_some() {
                if e.remove {
                    if self.retired.len() == TOMBSTONES {
                        let (old, seq) = self.retired.pop_front().unwrap();
                        if self
                            .rows
                            .get(&old)
                            .and_then(|r| r.event)
                            .is_some_and(|r| r.seq == seq)
                        {
                            self.rows.remove(&old);
                        }
                    }
                    self.retired.push_back((path.to_path_buf(), e.seq));
                }
                self.insert(
                    path,
                    Record {
                        value,
                        event: Some(e),
                        published: 0,
                    },
                );
                // T3 at commit, under the same mutex as lookup. Never scan start, notification,
                // or the time an external observer happens to poll. All clocks are system monotonic.
                if let Some(r) = self.rows.get_mut(path) {
                    r.published = now_ns();
                }
            } else {
                self.delta_errors += 1;
            } // unknown; never resurrect the invalidated value
        }
        self.settle();
    }
    pub fn json(&self) -> String {
        let backend = crate::watch::BACKEND;
        let phase = if self.full_busy || self.manual_busy {
            "scanning"
        } else if self.active_delta || !self.pending.is_empty() {
            "processing"
        } else if self.covered {
            "idle"
        } else {
            "unknown"
        };
        let stream = if self.terminal {
            "offline"
        } else if !self.armed {
            "subscribing"
        } else if self.need_full || self.full_busy {
            "reconciling"
        } else if !self.baseline {
            "incomplete_inventory"
        } else if !self.covered {
            "replaying"
        } else {
            "current"
        };
        let last = self.last.map_or_else(|| "null".into(), |c| c.to_string());
        format!(
            r#"{{"schema":"odina-resident/v2","backend":"{backend}","phase":"{phase}","watching":{},"completed":{},"last_exit":{last},"subscribed":{},"coverage":"{}","monitor":"{stream}","epoch":{},"full_scans":{},"delta_scans":{},"pending":{},"events":{},"loss_incidents":{},"queue_overflows":{},"delta_errors":{},"consecutive_same_path_events":{}}}"#,
            self.covered,
            self.completed,
            self.armed,
            if self.covered { "current" } else { "unknown" },
            self.epoch,
            self.full_scans,
            self.delta_scans,
            self.pending.len(),
            self.seq,
            self.loss_incidents,
            self.queue_overflows,
            self.delta_errors,
            self.same_path_run
        )
    }
    pub fn lookup(&self, path: &Path) -> String {
        let unknown = |reason| {
            format!(
                r#"{{"state":"unknown","reason":"{reason}","epoch":{}}}"#,
                self.epoch
            )
        };
        if !self.covered {
            return unknown("coverage");
        }
        let Some(r) = self.rows.get(path) else {
            return unknown("pending_or_unavailable");
        };
        let state = if r.value.is_some() {
            "complete"
        } else {
            "deleted"
        };
        let band = r
            .value
            .map_or_else(|| "null".into(), |v| format!("\"{}\"", v.band));
        let origin = if r.event.is_some() {
            "event"
        } else {
            "inventory"
        };
        format!(
            r#"{{"state":"{state}","origin":"{origin}","epoch":{},"generation":{},"verdict":{band},"bytes":{},"received_ns":{},"invalidated_ns":{},"published_ns":{}}}"#,
            self.epoch,
            r.event.map_or(0, |e| e.seq),
            r.value.map_or(0, |v| v.bytes),
            r.event.map_or(0, |e| e.received),
            r.event.map_or(0, |e| e.invalidated),
            r.published
        )
    }
}
#[derive(Default)]
pub(crate) struct Shared {
    pub state: Mutex<State>,
    pub wake: Condvar,
}
impl Shared {
    // A wake is not a fence by itself. The native reader captures this ticket, drains any
    // readable batches and acknowledges only when its kernel queue is observed empty.
    pub fn ticket(&self) -> (u64, u64) {
        let s = self.state.lock().unwrap();
        (s.epoch, s.full_scans)
    }
    pub fn drained(&self, ticket: (u64, u64)) {
        let mut s = self.state.lock().unwrap();
        if (s.epoch, s.full_scans) == ticket && s.baseline {
            s.fenced = true;
            s.settle();
        }
    }
    pub fn arm(&self) {
        let mut s = self.state.lock().unwrap();
        s.armed = true;
        s.need_full = true;
        self.wake.notify_one();
    }
    pub fn loss(&self) {
        let mut s = self.state.lock().unwrap();
        s.armed = false;
        s.invalidate_scope();
        self.wake.notify_one();
    }
    pub fn offline(&self) {
        let mut s = self.state.lock().unwrap();
        s.terminal = true;
        s.armed = false;
        s.invalidate_scope();
        self.wake.notify_one();
    }
    pub fn event(&self, path: &Path, remove: bool, received: u64) {
        let mut s = self.state.lock().unwrap();
        s.change(path.to_path_buf(), remove, received);
        self.wake.notify_one();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const V: FileVerdict = FileVerdict {
        band: "confirmed",
        bytes: 68,
    };
    fn ready(m: &Shared) {
        m.arm();
        let mut s = m.state.lock().unwrap();
        let Some(Task::Full(epoch, None)) = s.next() else {
            panic!("need baseline")
        };
        assert!(s.seed(epoch, Path::new("a"), V));
        assert!(s.seed(epoch, Path::new("b"), V));
        s.end_full(epoch, true);
        assert!(!s.covered);
        drop(s);
        m.drained(m.ticket());
    }
    #[test]
    fn invalidation_is_immediate_and_an_inflight_verdict_cannot_resurrect_a_deletion() {
        let m = Shared::default();
        ready(&m);
        let p = Path::new("a");
        let b = Path::new("b");
        m.event(p, false, now_ns());
        let mut s = m.state.lock().unwrap();
        assert!(s.covered);
        assert!(s.lookup(p).contains("unknown"));
        assert!(s.lookup(b).contains("confirmed"));
        let Some(Task::One(epoch, _, old)) = s.next() else {
            panic!()
        };
        s.change(p.to_owned(), true, now_ns());
        s.publish(epoch, p, old, Some(V));
        assert!(s.lookup(p).contains("unknown"));
        let Some(Task::One(epoch, _, del)) = s.next() else {
            panic!()
        };
        s.publish(epoch, p, del, None);
        assert!(s.lookup(p).contains("deleted"));
        assert!(!s.lookup(p).contains("confirmed"));
        assert_eq!((s.full_scans, s.delta_scans), (1, 1));
    }
    #[test]
    fn overflow_fences_stale_work_and_requires_inventory_kernel_drain_and_replay() {
        let m = Shared::default();
        ready(&m);
        let p = Path::new("a");
        m.event(p, false, now_ns());
        let Some(Task::One(old_epoch, _, old)) = m.state.lock().unwrap().next() else {
            panic!()
        };
        // Actual bounded application queue, no production overflow injection command.
        for i in 0..=QUEUE {
            m.event(Path::new(&i.to_string()), false, now_ns());
        }
        let mut s = m.state.lock().unwrap();
        assert_eq!(s.queue_overflows, 1);
        assert!(!s.covered);
        s.publish(old_epoch, p, old, Some(V));
        assert!(s.lookup(p).contains("coverage"));
        let Some(Task::Full(epoch, None)) = s.next() else {
            panic!()
        };
        assert_ne!(epoch, old_epoch);
        s.change(p.to_owned(), true, now_ns());
        s.seed(epoch, p, V);
        assert!(!s.rows.contains_key(p));
        s.end_full(epoch, true);
        drop(s);
        m.drained(m.ticket());
        let mut s = m.state.lock().unwrap();
        assert!(!s.covered);
        while let Some(Task::One(ep, k, e)) = s.next() {
            s.publish(ep, &k, e, if e.remove { None } else { Some(V) });
        }
        assert!(s.covered && s.lookup(p).contains("deleted"));
        assert_eq!(s.full_scans, 2);
        drop(s);
        m.loss();
        assert!(m.state.lock().unwrap().lookup(p).contains("coverage"));
    }
}
