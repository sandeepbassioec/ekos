use crate::epoch::EpochFence;
use crate::node::NodeId;
use dashmap::DashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, PartialEq)]
pub enum QuorumStatus {
    Pending,
    Achieved,
    Failed,
}

pub struct QuorumTracker {
    required: usize,
    acks: HashSet<NodeId>,
    nacks: HashSet<NodeId>,
    total_nodes: usize,
}

impl QuorumTracker {
    pub fn new(required: usize) -> Self {
        QuorumTracker {
            required,
            acks: HashSet::new(),
            nacks: HashSet::new(),
            total_nodes: 0,
        }
    }

    pub fn with_total(required: usize, total: usize) -> Self {
        QuorumTracker {
            required,
            acks: HashSet::new(),
            nacks: HashSet::new(),
            total_nodes: total,
        }
    }

    pub fn ack(&mut self, from: NodeId) -> QuorumStatus {
        self.acks.insert(from);
        self.status()
    }

    pub fn nack(&mut self, from: NodeId) -> QuorumStatus {
        self.nacks.insert(from);
        self.status()
    }

    pub fn status(&self) -> QuorumStatus {
        if self.acks.len() >= self.required {
            return QuorumStatus::Achieved;
        }
        // Failed if nacks make quorum impossible.
        if self.total_nodes > 0 {
            let remaining = self.total_nodes - self.acks.len() - self.nacks.len();
            let possible = self.acks.len() + remaining;
            if possible < self.required {
                return QuorumStatus::Failed;
            }
        }
        QuorumStatus::Pending
    }

    pub fn ack_count(&self) -> usize {
        self.acks.len()
    }
}

pub struct QuorumWrite {
    pub write_id: u64,
    pub mutation: Vec<u8>,
    pub tracker: QuorumTracker,
    pub epoch: u64,
}

pub struct QuorumCoordinator {
    writes: DashMap<u64, QuorumWrite>,
    next_id: AtomicU64,
    required: usize,
    fence: Arc<EpochFence>,
}

impl QuorumCoordinator {
    pub fn new(required: usize, fence: Arc<EpochFence>) -> Self {
        QuorumCoordinator {
            writes: DashMap::new(),
            next_id: AtomicU64::new(1),
            required,
            fence,
        }
    }

    pub fn begin_write(&self, mutation: Vec<u8>, epoch: u64) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.writes.insert(
            id,
            QuorumWrite {
                write_id: id,
                mutation,
                tracker: QuorumTracker::new(self.required),
                epoch,
            },
        );
        id
    }

    pub fn record_ack(&self, write_id: u64, from: NodeId) -> QuorumStatus {
        if let Some(mut w) = self.writes.get_mut(&write_id) {
            return w.tracker.ack(from);
        }
        QuorumStatus::Pending
    }

    pub fn record_nack(&self, write_id: u64, from: NodeId) -> QuorumStatus {
        if let Some(mut w) = self.writes.get_mut(&write_id) {
            return w.tracker.nack(from);
        }
        QuorumStatus::Pending
    }

    pub fn is_committed(&self, write_id: u64) -> bool {
        self.writes
            .get(&write_id)
            .map(|w| w.tracker.status() == QuorumStatus::Achieved)
            .unwrap_or(false)
    }

    pub fn abort_stale(&self) -> usize {
        let current = self.fence.current();
        let stale_ids: Vec<u64> = self
            .writes
            .iter()
            .filter(|e| e.epoch < current)
            .map(|e| *e.key())
            .collect();
        let count = stale_ids.len();
        for id in stale_ids {
            self.writes.remove(&id);
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::epoch::EpochFence;

    fn fence() -> Arc<EpochFence> {
        Arc::new(EpochFence::new())
    }

    #[test]
    fn new_tracker_pending() {
        let t = QuorumTracker::new(2);
        assert_eq!(t.status(), QuorumStatus::Pending);
    }

    #[test]
    fn ack_once_pending_quorum_2() {
        let mut t = QuorumTracker::new(2);
        let s = t.ack(NodeId::new());
        assert_eq!(s, QuorumStatus::Pending);
    }

    #[test]
    fn ack_twice_achieved() {
        let mut t = QuorumTracker::new(2);
        t.ack(NodeId::new());
        let s = t.ack(NodeId::new());
        assert_eq!(s, QuorumStatus::Achieved);
    }

    #[test]
    fn nack_majority_failed() {
        let mut t = QuorumTracker::with_total(2, 3);
        t.nack(NodeId::new());
        t.nack(NodeId::new());
        t.nack(NodeId::new());
        assert_eq!(t.status(), QuorumStatus::Failed);
    }

    #[test]
    fn quorum_1_achieved_on_first_ack() {
        let mut t = QuorumTracker::new(1);
        let s = t.ack(NodeId::new());
        assert_eq!(s, QuorumStatus::Achieved);
    }

    #[test]
    fn ack_count_correct() {
        let mut t = QuorumTracker::new(3);
        t.ack(NodeId::new());
        t.ack(NodeId::new());
        assert_eq!(t.ack_count(), 2);
    }

    #[test]
    fn begin_write_returns_id() {
        let f = fence();
        let coord = QuorumCoordinator::new(2, f.clone());
        let id = coord.begin_write(b"data".to_vec(), f.current());
        assert!(id > 0);
    }

    #[test]
    fn record_ack_advances_status() {
        let f = fence();
        let coord = QuorumCoordinator::new(2, f.clone());
        let epoch = f.current();
        let id = coord.begin_write(b"x".to_vec(), epoch);
        coord.record_ack(id, NodeId::new());
        let s = coord.record_ack(id, NodeId::new());
        assert_eq!(s, QuorumStatus::Achieved);
    }

    #[test]
    fn is_committed_true_after_quorum() {
        let f = fence();
        let coord = QuorumCoordinator::new(2, f.clone());
        let epoch = f.current();
        let id = coord.begin_write(b"x".to_vec(), epoch);
        coord.record_ack(id, NodeId::new());
        coord.record_ack(id, NodeId::new());
        assert!(coord.is_committed(id));
    }

    #[test]
    fn is_committed_false_before_quorum() {
        let f = fence();
        let coord = QuorumCoordinator::new(2, f.clone());
        let id = coord.begin_write(b"x".to_vec(), f.current());
        coord.record_ack(id, NodeId::new());
        assert!(!coord.is_committed(id));
    }

    #[test]
    fn stale_epoch_write_aborted() {
        let f = fence();
        let coord = QuorumCoordinator::new(2, f.clone());
        // Write at epoch 0.
        let id = coord.begin_write(b"x".to_vec(), 0);
        // Advance epoch.
        f.advance(1);
        let aborted = coord.abort_stale();
        assert_eq!(aborted, 1);
        assert!(!coord.is_committed(id));
    }

    #[test]
    fn two_concurrent_writes_independent() {
        let f = fence();
        let coord = QuorumCoordinator::new(2, f.clone());
        let epoch = f.current();
        let id1 = coord.begin_write(b"a".to_vec(), epoch);
        let id2 = coord.begin_write(b"b".to_vec(), epoch);
        coord.record_ack(id1, NodeId::new());
        coord.record_ack(id1, NodeId::new());
        assert!(coord.is_committed(id1));
        assert!(!coord.is_committed(id2));
    }

    #[test]
    fn same_node_acks_twice_idempotent() {
        let f = fence();
        let coord = QuorumCoordinator::new(2, f.clone());
        let id = coord.begin_write(b"x".to_vec(), f.current());
        let node = NodeId::new();
        coord.record_ack(id, node.clone());
        coord.record_ack(id, node.clone());
        assert!(!coord.is_committed(id)); // only 1 unique ack
    }
}
