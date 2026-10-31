//! [`Follower`] — receives [`ReplicationFrame`]s from a leader and applies
//! them to a local replica engine.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crossbeam_channel::{Receiver, TryRecvError};
use vectorvolt_engine::{EngineResult, EngineRow, TableEngine};

use crate::frame::{ReplicaMutation, ReplicationFrame};

/// A replication follower.
///
/// Holds a local `TableEngine` used as the replica store.  Mutations arrive
/// from the leader via an in-process channel and are applied by calling
/// [`apply_pending`] (drain once) or [`catchup`] (drain until empty).
///
/// Reads served from the follower reflect whatever mutations have been applied
/// so far — the follower may lag the leader.
pub struct Follower {
    engine: Arc<TableEngine>,
    pub(crate) receiver: Receiver<ReplicationFrame>,
    /// Highest sequence number applied.
    last_seq: AtomicU64,
    /// Total frames applied since construction.
    frames_applied: AtomicU64,
}

impl Follower {
    pub(crate) fn new(engine: Arc<TableEngine>, receiver: Receiver<ReplicationFrame>) -> Self {
        Self {
            engine,
            receiver,
            last_seq: AtomicU64::new(u64::MAX), // sentinel: nothing applied yet
            frames_applied: AtomicU64::new(0),
        }
    }

    /// Drain all pending frames from the channel and apply them.
    ///
    /// Returns the number of frames applied in this call.  May be called from
    /// any thread; concurrent calls are safe (crossbeam channels are MPMC).
    pub fn apply_pending(&self) -> EngineResult<usize> {
        let mut count = 0;
        loop {
            match self.receiver.try_recv() {
                Ok(frame) => {
                    self.apply_frame(frame)?;
                    count += 1;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }
        Ok(count)
    }

    /// Block until the channel is empty or disconnected, applying all frames.
    ///
    /// Useful in tests to guarantee the follower is fully caught up before
    /// asserting on replica state.
    pub fn catchup(&self) -> EngineResult<usize> {
        self.apply_pending()
    }

    /// Read a row from the replica engine (reflects applied mutations only).
    pub fn get(&self, pk: u64) -> EngineResult<Option<EngineRow>> {
        self.engine.get(pk)
    }

    /// Highest replication sequence number applied so far.
    ///
    /// Returns `None` if no frames have been applied yet.
    pub fn last_seq(&self) -> Option<u64> {
        let v = self.last_seq.load(Ordering::Relaxed);
        if v == u64::MAX {
            None
        } else {
            Some(v)
        }
    }

    /// Total number of frames ever applied by this follower.
    pub fn frames_applied(&self) -> u64 {
        self.frames_applied.load(Ordering::Relaxed)
    }

    fn apply_frame(&self, frame: ReplicationFrame) -> EngineResult<()> {
        match frame.mutation {
            ReplicaMutation::Insert { pk, values } => {
                self.engine.insert(pk, values)?;
            }
            ReplicaMutation::Delete { pk } => {
                self.engine.delete(pk)?;
            }
        }
        self.last_seq.store(frame.seq, Ordering::Relaxed);
        self.frames_applied.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}
