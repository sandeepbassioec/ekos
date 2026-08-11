//! [`Leader`] — accepts writes, applies them to the local engine, and ships
//! [`ReplicationFrame`]s to all connected followers.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crossbeam_channel::{unbounded, Sender};
use parking_lot::Mutex;
use vectorvolt_engine::{EngineResult, TableEngine};

use crate::follower::Follower;
use crate::frame::{ReplicaMutation, ReplicationFrame};

/// The replication leader.
///
/// Wraps a `TableEngine` and fans every write out to all registered followers
/// via an in-process channel.  Followers can lag arbitrarily; the leader never
/// blocks on them.
pub struct Leader {
    pub(crate) engine: Arc<TableEngine>,
    seq: AtomicU64,
    senders: Mutex<Vec<Sender<ReplicationFrame>>>,
}

impl Leader {
    /// Create a leader backed by `engine`.
    pub fn new(engine: Arc<TableEngine>) -> Arc<Self> {
        Arc::new(Self {
            engine,
            seq: AtomicU64::new(0),
            senders: Mutex::new(Vec::new()),
        })
    }

    /// Attach a new follower backed by `replica_engine`.
    ///
    /// Returns the follower which receives every frame the leader emits from
    /// this point forward.  Frames emitted before `connect_replica` are not
    /// retroactively delivered (no log replay in this layer).
    pub fn connect_replica(self: &Arc<Self>, replica_engine: Arc<TableEngine>) -> Follower {
        let (tx, rx) = unbounded();
        self.senders.lock().push(tx);
        Follower::new(replica_engine, rx)
    }

    /// Insert a row into the leader engine and broadcast to all followers.
    pub fn insert(&self, pk: u64, values: Vec<u64>) -> EngineResult<()> {
        self.engine.insert(pk, values.clone())?;
        self.broadcast(ReplicaMutation::Insert { pk, values });
        Ok(())
    }

    /// Delete a row from the leader engine and broadcast to all followers.
    pub fn delete(&self, pk: u64) -> EngineResult<()> {
        self.engine.delete(pk)?;
        self.broadcast(ReplicaMutation::Delete { pk });
        Ok(())
    }

    /// Read a row directly from the leader engine.
    pub fn get(&self, pk: u64) -> EngineResult<Option<vectorvolt_engine::EngineRow>> {
        self.engine.get(pk)
    }

    /// Number of followers currently connected.
    pub fn replica_count(&self) -> usize {
        self.senders.lock().len()
    }

    fn broadcast(&self, mutation: ReplicaMutation) {
        let seq = self.seq.fetch_add(1, Ordering::Relaxed);
        let frame = ReplicationFrame { seq, mutation };
        let mut senders = self.senders.lock();
        // Remove disconnected followers (send error means the receiver dropped).
        senders.retain(|tx| tx.send(frame.clone()).is_ok());
    }
}
