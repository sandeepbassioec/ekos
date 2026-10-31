//! [`SyncCommitter`] — synchronous group commit (VV-WAL-011).
//!
//! ## The problem
//!
//! `fsync` measures **526 µs** on the dev laptop against **7.8 µs** for the same
//! insert with no sync — durability costs 67× the actual work. Paying it once per
//! commit is the worst possible shape.
//!
//! ## The fix, and its precise limit
//!
//! When several threads commit concurrently, one `fsync` can make *all* their
//! writes durable — the syscall flushes the file, not one caller's bytes. So one
//! thread becomes the leader and syncs; the rest wait and are released by it. N
//! concurrent commits cost one `fsync` instead of N.
//!
//! **This helps concurrency, not single-thread latency.** A lone thread doing
//! sequential durable inserts has nobody to batch with, so it pays a full `fsync`
//! every time and ~526 µs is irreducible — that is the storage device, not
//! software. Anyone reading "group commit" as "durable writes become fast" will be
//! disappointed; it means "durable writes stop scaling with writer count".
//!
//! Getting a *single* thread below one-fsync-per-commit requires acknowledging
//! before the sync completes, which is a different durability guarantee, not an
//! optimisation. That is deliberately not done here.
//!
//! ## Why not [`crate::GroupCommitBatcher`]
//!
//! That type already exists and does batch commits — but it is `async` and built on
//! a tokio task, while the write path (`TxnManager::insert`) is synchronous. Wiring
//! it in would mean either making the whole write path async or blocking on a
//! future inside a lock, and putting a tokio runtime on a database's commit path is
//! the wrong trade when the goal is nanoseconds. This is its synchronous sibling:
//! a mutex, a condvar, and no runtime.
//!
//! ## Correctness argument
//!
//! `synced_upto` is only ever advanced to a sequence number that was read **before**
//! the `fsync` that made it durable began. So it can lag reality — a concurrent
//! append might get synced without being credited — but it can never lead it. A
//! caller told "your sequence is durable" is always telling the truth; a caller
//! told otherwise merely does one more sync than strictly necessary. Erring in that
//! direction is the only acceptable one for durability.

use std::sync::atomic::{AtomicU64, Ordering};

use parking_lot::{Condvar, Mutex};

use crate::error::WalResult;
use crate::writer::{WalTip, WalWriter};

#[derive(Debug, Default)]
struct SyncState {
    /// Highest sequence known to be on stable storage.
    synced_upto: u64,
    /// A leader is mid-`fsync`; others should wait rather than pile on.
    syncing: bool,
}

/// Counters for verifying batching actually happens.
///
/// A test asserting "group commit works" needs to see commits ≫ syncs; without
/// these it could only observe that the code did not crash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyncStats {
    /// Commits that requested durability.
    pub commits: u64,
    /// `fsync` calls actually issued.
    pub syncs: u64,
    /// Commits made durable by *someone else's* sync.
    pub free_rides: u64,
}

impl SyncStats {
    /// Mean commits per `fsync`. 1.0 means no batching occurred.
    pub fn commits_per_sync(&self) -> f64 {
        if self.syncs == 0 {
            return 0.0;
        }
        self.commits as f64 / self.syncs as f64
    }
}

/// Wraps a [`WalWriter`] with leader-follower group commit.
pub struct SyncCommitter {
    wal: WalWriter,
    state: Mutex<SyncState>,
    ready: Condvar,
    commits: AtomicU64,
    syncs: AtomicU64,
    free_rides: AtomicU64,
}

impl SyncCommitter {
    pub fn new(wal: WalWriter) -> Self {
        SyncCommitter {
            wal,
            state: Mutex::new(SyncState::default()),
            ready: Condvar::new(),
            commits: AtomicU64::new(0),
            syncs: AtomicU64::new(0),
            free_rides: AtomicU64::new(0),
        }
    }

    /// The underlying writer, for reads and non-durable appends.
    pub fn wal(&self) -> &WalWriter {
        &self.wal
    }

    /// Append without any durability guarantee. Bytes stay in the userspace buffer.
    pub fn append(&self, payload: Vec<u8>) -> WalResult<WalTip> {
        self.wal.append(payload)
    }

    /// Append and return only once that frame is on stable storage.
    ///
    /// Under concurrency this is where batching happens: whichever thread arrives
    /// while no sync is in flight becomes the leader and pays the `fsync` for
    /// everyone waiting behind it.
    pub fn append_and_sync(&self, payload: Vec<u8>) -> WalResult<WalTip> {
        let tip = self.wal.append(payload)?;
        self.sync_upto(tip.sequence)?;
        Ok(tip)
    }

    /// Block until `sequence` is durable.
    pub fn sync_upto(&self, sequence: u64) -> WalResult<()> {
        self.commits.fetch_add(1, Ordering::Relaxed);

        loop {
            // Acquired fresh each iteration; the guard is released at the end of
            // the iteration, including on `continue`.
            let mut state = self.state.lock();

            // Somebody already synced past us. Nothing to do — this is the win.
            if state.synced_upto >= sequence {
                self.free_rides.fetch_add(1, Ordering::Relaxed);
                return Ok(());
            }

            if state.syncing {
                // A leader is mid-fsync. Wait; it will either cover us, or we
                // re-evaluate and possibly lead the next round ourselves.
                self.ready.wait(&mut state);
                continue;
            }

            // Become the leader.
            state.syncing = true;
            drop(state); // never hold the state lock across the syscall

            // `flush_covering` reads the tip under the same append lock as its
            // buffer flush, and returns it only once the fsync that follows
            // (outside that lock, so appends keep flowing) has completed. The
            // mark we publish can therefore only lag reality, never lead it: an
            // append landing after the buffer flush is not credited — safe, and
            // it merely costs one extra sync later.
            let result = self.wal.flush_covering().map(|tip| tip.sequence);
            self.syncs.fetch_add(1, Ordering::Relaxed);

            let covered = {
                let mut state = self.state.lock();
                state.syncing = false;
                if let Ok(high_water) = result {
                    state.synced_upto = state.synced_upto.max(high_water);
                }
                // Wake everyone: those we covered return, the rest re-evaluate and
                // one becomes the next leader. Must happen even on error, or
                // waiters would block forever behind a failed sync.
                self.ready.notify_all();
                state.synced_upto >= sequence
            };

            result?;

            // A successful flush may still not cover us, if `high_water` was read
            // before our own append became visible. Loop rather than assume.
            if covered {
                return Ok(());
            }
        }
    }

    /// Flush and sync unconditionally, regardless of what is already durable.
    pub fn flush(&self) -> WalResult<()> {
        let high_water = self.wal.flush_covering()?.sequence;
        self.syncs.fetch_add(1, Ordering::Relaxed);
        let mut state = self.state.lock();
        state.synced_upto = state.synced_upto.max(high_water);
        self.ready.notify_all();
        Ok(())
    }

    pub fn stats(&self) -> SyncStats {
        SyncStats {
            commits: self.commits.load(Ordering::Relaxed),
            syncs: self.syncs.load(Ordering::Relaxed),
            free_rides: self.free_rides.load(Ordering::Relaxed),
        }
    }

    /// Highest sequence currently known durable.
    pub fn synced_upto(&self) -> u64 {
        self.state.lock().synced_upto
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::WalConfig;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn committer(dir: &TempDir, fsync: bool) -> SyncCommitter {
        let cfg = WalConfig::default().with_dir(dir.path()).with_fsync(fsync);
        SyncCommitter::new(WalWriter::open(cfg).unwrap())
    }

    #[test]
    fn single_thread_gets_one_sync_per_commit() {
        // The honest baseline: with no concurrency there is nothing to batch, so
        // group commit cannot help. Asserted so the limitation is documented in
        // executable form rather than only in prose.
        let dir = TempDir::new().unwrap();
        let c = committer(&dir, false);
        for i in 0..10u64 {
            c.append_and_sync(vec![i as u8; 32]).unwrap();
        }
        let s = c.stats();
        assert_eq!(s.commits, 10);
        assert_eq!(s.syncs, 10, "a lone writer cannot batch with anyone");
        assert_eq!(s.free_rides, 0);
        assert!((s.commits_per_sync() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn concurrent_commits_share_syncs() {
        // The actual point of the type.
        const THREADS: usize = 8;
        const PER_THREAD: usize = 40;

        let dir = TempDir::new().unwrap();
        // fsync ON: without a real syscall the sync window is too short for
        // followers to arrive, and the test would prove nothing.
        let c = Arc::new(committer(&dir, true));

        let handles: Vec<_> = (0..THREADS)
            .map(|t| {
                let c = Arc::clone(&c);
                std::thread::spawn(move || {
                    for i in 0..PER_THREAD {
                        c.append_and_sync(vec![(t * 100 + i) as u8; 64]).unwrap();
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().expect("committer thread panicked");
        }

        let s = c.stats();
        let total = (THREADS * PER_THREAD) as u64;
        assert_eq!(s.commits, total);
        println!(
            "  {THREADS} threads x {PER_THREAD}: {} commits, {} syncs, {} free rides \
             ({:.2} commits/sync)",
            s.commits,
            s.syncs,
            s.free_rides,
            s.commits_per_sync()
        );

        assert!(
            s.syncs < s.commits,
            "no batching at all: {} syncs for {} commits",
            s.syncs,
            s.commits
        );
        assert!(
            s.free_rides > 0,
            "no commit was served by another thread's sync"
        );
        // Deliberately a weak bound. Actual batching depends on how threads
        // interleave, which the scheduler decides; a tight assertion here would be
        // flaky. Any batching at all proves the mechanism works.
        assert!(
            s.commits_per_sync() > 1.0,
            "commits/sync = {:.2}, expected > 1",
            s.commits_per_sync()
        );
    }

    #[test]
    fn sync_upto_is_idempotent_and_monotonic() {
        let dir = TempDir::new().unwrap();
        let c = committer(&dir, false);
        let tip = c.append_and_sync(vec![1, 2, 3]).unwrap();

        let syncs_before = c.stats().syncs;
        // Re-requesting an already-durable sequence must not sync again.
        c.sync_upto(tip.sequence).unwrap();
        c.sync_upto(tip.sequence).unwrap();
        assert_eq!(
            c.stats().syncs,
            syncs_before,
            "re-syncing an already-durable sequence issued another fsync"
        );
        assert!(c.synced_upto() >= tip.sequence);
    }

    #[test]
    fn synced_mark_never_leads_reality() {
        // The safety property: we may under-report durability, never over-report.
        let dir = TempDir::new().unwrap();
        let c = committer(&dir, false);

        for i in 0..5u64 {
            let tip = c.append(vec![i as u8; 16]).unwrap();
            // Appended but not synced — the mark must not have moved.
            assert!(
                c.synced_upto() < tip.sequence,
                "sequence {} reported durable without any sync",
                tip.sequence
            );
        }

        c.flush().unwrap();
        assert!(
            c.synced_upto() >= 5,
            "flush did not advance the durable mark: {}",
            c.synced_upto()
        );
    }

    #[test]
    fn append_without_sync_does_not_count_as_a_commit() {
        let dir = TempDir::new().unwrap();
        let c = committer(&dir, false);
        for i in 0..5u64 {
            c.append(vec![i as u8]).unwrap();
        }
        let s = c.stats();
        assert_eq!(s.commits, 0, "plain appends must not be counted as commits");
        assert_eq!(s.syncs, 0);
    }
}
