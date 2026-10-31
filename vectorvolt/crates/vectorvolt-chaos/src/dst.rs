//! [`DstHarness`] — Deterministic Simulation Testing harness.
//!
//! A single-threaded, event-driven simulator that replays network/disk faults
//! reproducibly from a seed.  All non-determinism is replaced by seeded LCG
//! choices so two runs with the same seed produce identical event logs.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::fault::FaultInjector;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// The kind of event processed by the DST harness.
#[derive(Debug, Clone)]
pub enum DstEventKind {
    NetworkSend {
        from: u64,
        to: u64,
        payload: Vec<u8>,
    },
    NetworkRecv {
        from: u64,
        to: u64,
        payload: Vec<u8>,
    },
    DiskWrite {
        node: u64,
        offset: u64,
        data: Vec<u8>,
    },
    DiskRead {
        node: u64,
        offset: u64,
        len: usize,
    },
    Crash {
        node: u64,
    },
    Recover {
        node: u64,
    },
    Tick,
}

/// A single scheduled event in the DST harness.
#[derive(Debug, Clone)]
pub struct DstEvent {
    pub at_tick: u64,
    pub kind: DstEventKind,
}

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

/// Deterministic Simulation Testing harness.
pub struct DstHarness {
    tick: u64,
    /// Priority queue of `(at_tick, event_log_index)` — min-heap.
    pending: BinaryHeap<Reverse<(u64, usize)>>,
    /// All events that have been scheduled (by index).
    scheduled: Vec<DstEvent>,
    event_log: Vec<DstEvent>,
    pub injector: FaultInjector,
}

impl DstHarness {
    /// Create a new harness with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            tick: 0,
            pending: BinaryHeap::new(),
            scheduled: Vec::new(),
            event_log: Vec::new(),
            injector: FaultInjector::new(seed),
        }
    }

    /// Schedule an event to fire at `at_tick`.
    pub fn schedule(&mut self, at_tick: u64, kind: DstEventKind) {
        let idx = self.scheduled.len();
        self.scheduled.push(DstEvent { at_tick, kind });
        self.pending.push(Reverse((at_tick, idx)));
    }

    /// Process the next scheduled event (lowest `at_tick`).
    ///
    /// Advances `self.tick` to the event's tick, appends to `event_log`, and
    /// returns a reference to the newly logged event.  Returns `None` when the
    /// pending queue is empty.
    pub fn advance(&mut self) -> Option<&DstEvent> {
        let Reverse((at_tick, idx)) = self.pending.pop()?;
        self.tick = at_tick;
        let event = self.scheduled[idx].clone();
        self.event_log.push(event);
        self.injector.tick();
        Some(self.event_log.last().unwrap())
    }

    /// Process all events up to and including `max_tick`.
    pub fn run_until(&mut self, max_tick: u64) -> Vec<&DstEvent> {
        let mut out = Vec::new();
        loop {
            match self.pending.peek() {
                Some(Reverse((t, _))) if *t <= max_tick => {}
                _ => break,
            }
            if let Some(ev) = self.advance() {
                // Safety: lifetime is tied to self.event_log which only grows.
                let ptr: *const DstEvent = ev;
                out.push(unsafe { &*ptr });
            }
        }
        out
    }

    /// Iterate over all logged events in the order they were processed.
    pub fn replay(&self) -> impl Iterator<Item = &DstEvent> {
        self.event_log.iter()
    }

    /// Current simulation tick.
    pub fn tick(&self) -> u64 {
        self.tick
    }

    /// Total number of events in the log.
    pub fn event_count(&self) -> usize {
        self.event_log.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_harness_tick_zero() {
        let h = DstHarness::new(42);
        assert_eq!(h.tick(), 0);
    }

    #[test]
    fn schedule_event_at_tick_5() {
        let mut h = DstHarness::new(0);
        h.schedule(5, DstEventKind::Tick);
        assert_eq!(h.event_count(), 0); // not yet processed
    }

    #[test]
    fn advance_returns_event_at_tick_5() {
        let mut h = DstHarness::new(0);
        h.schedule(5, DstEventKind::Tick);
        let ev = h.advance().expect("should have event");
        assert_eq!(ev.at_tick, 5);
    }

    #[test]
    fn advance_none_when_empty() {
        let mut h = DstHarness::new(0);
        assert!(h.advance().is_none());
    }

    #[test]
    fn run_until_processes_all_events() {
        let mut h = DstHarness::new(0);
        h.schedule(1, DstEventKind::Tick);
        h.schedule(3, DstEventKind::Tick);
        h.schedule(5, DstEventKind::Tick);
        let events = h.run_until(5);
        assert_eq!(events.len(), 3);
    }

    #[test]
    fn replay_returns_all_events_in_order() {
        let mut h = DstHarness::new(0);
        h.schedule(2, DstEventKind::Tick);
        h.schedule(1, DstEventKind::Tick);
        h.run_until(10);
        let ticks: Vec<u64> = h.replay().map(|e| e.at_tick).collect();
        assert_eq!(ticks, vec![1, 2]);
    }

    #[test]
    fn two_events_same_tick_both_returned() {
        let mut h = DstHarness::new(0);
        h.schedule(3, DstEventKind::Tick);
        h.schedule(3, DstEventKind::Tick);
        let evs = h.run_until(10);
        assert_eq!(evs.len(), 2);
    }

    #[test]
    fn crash_event_at_tick_3() {
        let mut h = DstHarness::new(0);
        h.schedule(3, DstEventKind::Crash { node: 1 });
        let ev = h.advance().unwrap();
        assert_eq!(ev.at_tick, 3);
        assert!(matches!(ev.kind, DstEventKind::Crash { node: 1 }));
    }

    #[test]
    fn recover_after_crash() {
        let mut h = DstHarness::new(0);
        h.schedule(1, DstEventKind::Crash { node: 2 });
        h.schedule(5, DstEventKind::Recover { node: 2 });
        h.run_until(10);
        let kinds: Vec<bool> = h
            .replay()
            .map(|e| matches!(e.kind, DstEventKind::Recover { .. }))
            .collect();
        assert!(kinds.contains(&true));
    }

    #[test]
    fn network_send_recv_pair() {
        let mut h = DstHarness::new(0);
        h.schedule(
            1,
            DstEventKind::NetworkSend {
                from: 0,
                to: 1,
                payload: vec![1, 2, 3],
            },
        );
        h.schedule(
            2,
            DstEventKind::NetworkRecv {
                from: 0,
                to: 1,
                payload: vec![1, 2, 3],
            },
        );
        h.run_until(10);
        assert_eq!(h.event_count(), 2);
    }

    #[test]
    fn disk_write_then_read() {
        let mut h = DstHarness::new(0);
        h.schedule(
            1,
            DstEventKind::DiskWrite {
                node: 0,
                offset: 0,
                data: vec![0xAB],
            },
        );
        h.schedule(
            2,
            DstEventKind::DiskRead {
                node: 0,
                offset: 0,
                len: 1,
            },
        );
        h.run_until(10);
        assert_eq!(h.event_count(), 2);
    }

    #[test]
    fn seed_reproducibility() {
        let mut h1 = DstHarness::new(1234);
        h1.schedule(1, DstEventKind::Tick);
        h1.schedule(2, DstEventKind::Tick);
        h1.run_until(10);

        let mut h2 = DstHarness::new(1234);
        h2.schedule(1, DstEventKind::Tick);
        h2.schedule(2, DstEventKind::Tick);
        h2.run_until(10);

        let t1: Vec<u64> = h1.replay().map(|e| e.at_tick).collect();
        let t2: Vec<u64> = h2.replay().map(|e| e.at_tick).collect();
        assert_eq!(t1, t2);
    }

    #[test]
    fn event_count_correct() {
        let mut h = DstHarness::new(0);
        for i in 0..5u64 {
            h.schedule(i, DstEventKind::Tick);
        }
        h.run_until(100);
        assert_eq!(h.event_count(), 5);
    }
}
