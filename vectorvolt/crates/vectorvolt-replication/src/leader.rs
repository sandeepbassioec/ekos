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
///
/// ## Ordering
///
/// The engine write, the sequence-number assignment and the send to every
/// follower all happen while holding the `senders` lock.  Leader writes are
/// therefore serialized, and the order in which frames reach a follower's
/// channel (and their `seq`) is exactly the order in which the writes were
/// applied to the leader engine.  Assigning the seq after releasing the
/// engine write would let two concurrent writers to the same key apply
/// A-then-B on the leader but ship B-then-A, leaving the follower diverged.
pub struct Leader {
    pub(crate) engine: Arc<TableEngine>,
    seq: AtomicU64,
    /// Follower channels. This lock also serializes leader writes (see the
    /// type-level docs), so it must be held across write + seq + send.
    senders: Mutex<Vec<Sender<ReplicationFrame>>>,
    /// Test hook run once, right after the next engine write and before its
    /// frame is sequenced, to widen the write/broadcast window in tests.
    #[cfg(test)]
    after_engine_write: Mutex<Option<Box<dyn FnOnce() + Send>>>,
}

impl Leader {
    /// Create a leader backed by `engine`.
    pub fn new(engine: Arc<TableEngine>) -> Arc<Self> {
        Arc::new(Self {
            engine,
            seq: AtomicU64::new(0),
            senders: Mutex::new(Vec::new()),
            #[cfg(test)]
            after_engine_write: Mutex::new(None),
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
        let mut senders = self.senders.lock();
        self.engine.insert(pk, values.clone())?;
        self.run_test_hook();
        self.broadcast_locked(&mut senders, ReplicaMutation::Insert { pk, values });
        Ok(())
    }

    /// Delete a row from the leader engine and broadcast to all followers.
    pub fn delete(&self, pk: u64) -> EngineResult<()> {
        let mut senders = self.senders.lock();
        self.engine.delete(pk)?;
        self.run_test_hook();
        self.broadcast_locked(&mut senders, ReplicaMutation::Delete { pk });
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

    #[inline]
    fn run_test_hook(&self) {
        // Take the hook first so its mutex is not held while it runs
        // (edition 2021 keeps `if let` scrutinee temporaries alive).
        #[cfg(test)]
        {
            let hook = self.after_engine_write.lock().take();
            if let Some(hook) = hook {
                hook();
            }
        }
    }

    /// Assign the next seq and send the frame. The caller must hold the
    /// `senders` lock across its engine write and this call.
    fn broadcast_locked(
        &self,
        senders: &mut Vec<Sender<ReplicationFrame>>,
        mutation: ReplicaMutation,
    ) {
        let seq = self.seq.fetch_add(1, Ordering::Relaxed);
        let frame = ReplicationFrame { seq, mutation };
        // Remove disconnected followers (send error means the receiver dropped).
        senders.retain(|tx| tx.send(frame.clone()).is_ok());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vectorvolt_engine::{Durability, EngineOptions};

    fn engine(dir: &tempfile::TempDir) -> Arc<TableEngine> {
        Arc::new(
            TableEngine::open_with(
                vec!["pk".into(), "val".into()],
                dir.path(),
                EngineOptions::new(1_048_576).with_durability(Durability::WriteOnly),
            )
            .unwrap(),
        )
    }

    /// Writer A is paused between its engine write and its broadcast while
    /// writer B writes the same key. B must not be able to slip its frame in
    /// ahead of A's (B19): the follower must end with B's value, as the
    /// leader does.
    #[test]
    fn write_paused_before_broadcast_does_not_reorder_frames() {
        let ld = tempfile::TempDir::new().unwrap();
        let rd = tempfile::TempDir::new().unwrap();
        let leader = Leader::new(engine(&ld));
        let follower = leader.connect_replica(engine(&rd));

        let (entered_tx, entered_rx) = crossbeam_channel::bounded::<()>(1);
        *leader.after_engine_write.lock() = Some(Box::new(move || {
            entered_tx.send(()).unwrap();
            // Give writer B ample time to run; with the fix it simply blocks.
            std::thread::sleep(std::time::Duration::from_millis(200));
        }));

        std::thread::scope(|s| {
            let a = s.spawn(|| leader.insert(1, vec![1, 100]).unwrap());
            entered_rx.recv().unwrap();
            let b = s.spawn(|| leader.insert(1, vec![1, 200]).unwrap());
            a.join().unwrap();
            b.join().unwrap();
        });
        follower.catchup().unwrap();

        let l = leader.get(1).unwrap().unwrap().values;
        let f = follower.get(1).unwrap().unwrap().values;
        assert_eq!(l, f, "follower diverged from leader");
        assert_eq!(f[1], 200);
    }

    /// Frames must reach the follower channel in seq order even when many
    /// threads write concurrently (B19).
    #[test]
    fn concurrent_writes_ship_frames_in_seq_order() {
        let ld = tempfile::TempDir::new().unwrap();
        let rd = tempfile::TempDir::new().unwrap();
        let leader = Leader::new(engine(&ld));
        let follower = leader.connect_replica(engine(&rd));

        const THREADS: u64 = 8;
        const PER_THREAD: u64 = 200;
        std::thread::scope(|s| {
            for t in 0..THREADS {
                let leader = &leader;
                s.spawn(move || {
                    for i in 0..PER_THREAD {
                        leader.insert(i % 4, vec![i % 4, t * 1000 + i]).unwrap();
                    }
                });
            }
        });

        let seqs: Vec<u64> = follower.receiver.try_iter().map(|f| f.seq).collect();
        let expected: Vec<u64> = (0..THREADS * PER_THREAD).collect();
        assert_eq!(
            seqs, expected,
            "frames reached the channel out of seq order"
        );
    }
}
