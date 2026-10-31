//! Correctness gaps closed in batch B:
//!
//! - B1 / PP-23: per-pk write ordering — version order equals WAL order.
//! - B2 / PP-24: MVCC history survives a restart.
//! - B23: a delete stays deleted across flush, compaction and restart.
//! - B24: compaction does not block reads or writes while it builds.
//!
//! "Restart" here is what `TableEngine::open` does at the txn layer: a fresh
//! `TxnManager` on the same WAL and data directories, the manifest re-enabled,
//! and the pk index rebuilt from the partitions the manifest names.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use tempfile::TempDir;
use vectorvolt_storage::{CompactionStrategy, PartitionReader, WriteOptions};
use vectorvolt_txn::{Mutation, TxnManager};
use vectorvolt_wal::{WalConfig, WalCursor, WalReader, WalWriter};

fn wal_dir(root: &Path) -> PathBuf {
    root.join("wal")
}

fn data_dir(root: &Path) -> PathBuf {
    root.join("data")
}

fn open_manager(root: &Path, flush_threshold: usize, start_partition: u16) -> TxnManager {
    let wd = wal_dir(root);
    std::fs::create_dir_all(&wd).unwrap();
    let cfg = WalConfig::default().with_dir(&wd).with_fsync(false);
    TxnManager::with_starting_partition(
        WalWriter::open(cfg).unwrap(),
        vec!["pk".into(), "v".into()],
        data_dir(root),
        flush_threshold,
        start_partition,
    )
}

/// A fresh manager over the same directories, recovered the way the engine
/// recovers one (without the engine's replay of unflushed frames — every test
/// flushes before restarting).
fn reopen(root: &Path) -> TxnManager {
    let probe = open_manager(root, 1_000_000, 0);
    probe.enable_manifest().unwrap();
    let metas = probe.partitions();
    drop(probe);

    let mut readers = HashMap::new();
    let mut next_pid = 0u16;
    for meta in &metas {
        let r = PartitionReader::open(&meta.path).unwrap();
        next_pid = next_pid.max(r.partition_id() + 1);
        readers.insert((r.file_id(), r.partition_id()), r);
    }
    let m = open_manager(root, 1_000_000, next_pid);
    m.enable_manifest().unwrap();
    m.rebuild_pk_index_from_partitions(&readers).unwrap();
    m
}

/// Every `(sequence, values)` the WAL holds for `pk`, in WAL order.
fn wal_values_for(root: &Path, pk: u64) -> Vec<(u64, Vec<u64>)> {
    let cfg = WalConfig::default()
        .with_dir(wal_dir(root))
        .with_fsync(false);
    let mut out = Vec::new();
    WalReader::open(cfg)
        .scan(WalCursor::beginning(), |frame| {
            match Mutation::decode(&frame.payload) {
                Ok(Mutation::Insert { pk: p, values }) | Ok(Mutation::Upsert { pk: p, values })
                    if p == pk =>
                {
                    out.push((frame.sequence, values));
                }
                _ => {}
            }
            true
        })
        .unwrap();
    out
}

// ── B1 / PP-23 ───────────────────────────────────────────────────────────────

/// Many threads overwrite one pk. Afterwards the current value is the one in
/// the highest-sequence WAL frame, and the version chain, read at each frame's
/// sequence, replays the WAL exactly.
#[test]
fn concurrent_same_pk_writes_keep_version_order_equal_to_wal_order() {
    let dir = TempDir::new().unwrap();
    let m = Arc::new(open_manager(dir.path(), 1_000_000, 0));
    const PK: u64 = 42;
    m.insert(PK, vec![PK, 0]).unwrap();

    let threads: Vec<_> = (1..=8u64)
        .map(|t| {
            let m = Arc::clone(&m);
            thread::spawn(move || {
                for i in 0..200u64 {
                    m.upsert(PK, vec![PK, t * 10_000 + i]).unwrap();
                    // An unrelated pk on every iteration, so stripes other
                    // than this one are exercised alongside it.
                    m.upsert(t * 1_000_000 + i, vec![t, i]).unwrap();
                }
            })
        })
        .collect();
    for t in threads {
        t.join().unwrap();
    }
    let current = m.get(PK).unwrap();
    let ts_now = m.begin_snapshot();
    // The WAL is only fully on disk once its writer drops.
    drop(m);

    let frames = wal_values_for(dir.path(), PK);
    assert_eq!(frames.len(), 1 + 8 * 200);
    let (last_seq, last_values) = frames.last().unwrap().clone();
    assert_eq!(
        current,
        Some(last_values),
        "current value is not the WAL's last"
    );
    assert!(ts_now >= last_seq);
}

/// The same race, checked against the version chain: every WAL frame for the
/// pk is visible at exactly its own sequence number.
#[test]
fn concurrent_same_pk_versions_match_wal_frames() {
    let dir = TempDir::new().unwrap();
    const PK: u64 = 7;
    let checks: Vec<(u64, Option<Vec<u64>>)>;
    {
        let m = Arc::new(open_manager(dir.path(), 1_000_000, 0));
        m.insert(PK, vec![PK, 0]).unwrap();
        let threads: Vec<_> = (1..=8u64)
            .map(|t| {
                let m = Arc::clone(&m);
                thread::spawn(move || {
                    for i in 0..200u64 {
                        m.upsert(PK, vec![PK, t * 10_000 + i]).unwrap();
                    }
                })
            })
            .collect();
        for t in threads {
            t.join().unwrap();
        }
        // Read the chain at every sequence the WAL could have used.
        let top = m.begin_snapshot();
        checks = (1..=top).map(|ts| (ts, m.get_at(PK, ts))).collect();
    }
    let frames = wal_values_for(dir.path(), PK);
    assert_eq!(frames.len(), 1 + 8 * 200);
    let by_ts: HashMap<u64, Option<Vec<u64>>> = checks.into_iter().collect();
    for (seq, values) in &frames {
        assert_eq!(
            by_ts.get(seq),
            Some(&Some(values.clone())),
            "version at ts {seq} does not match the WAL frame at that sequence"
        );
    }
}

// ── B2 / PP-24 ───────────────────────────────────────────────────────────────

/// Versions written before a restart are readable at their old timestamps
/// after it, including deletes, and new writes get later timestamps.
#[test]
fn mvcc_history_survives_restart() {
    let dir = TempDir::new().unwrap();
    let (ts_v1, ts_v2, ts_before_delete, ts_after_delete);
    {
        let m = open_manager(dir.path(), 1_000_000, 0);
        m.enable_manifest().unwrap();
        m.insert(1, vec![1, 100]).unwrap();
        m.insert(2, vec![2, 200]).unwrap();
        ts_v1 = m.begin_snapshot();
        m.flush().unwrap();
        m.update(1, vec![1, 101]).unwrap();
        ts_v2 = m.begin_snapshot();
        m.flush().unwrap();
        ts_before_delete = m.begin_snapshot();
        m.delete(2).unwrap();
        ts_after_delete = m.begin_snapshot();
        m.flush().unwrap();
    }

    let m = reopen(dir.path());
    assert_eq!(m.get_at(1, ts_v1), Some(vec![1, 100]), "old version lost");
    assert_eq!(m.get_at(1, ts_v2), Some(vec![1, 101]));
    assert_eq!(m.get_at(2, ts_before_delete), Some(vec![2, 200]));
    assert_eq!(m.get_at(2, ts_after_delete), None, "delete lost on restart");
    assert_eq!(
        m.get_at(1, 0),
        None,
        "nothing existed before the first commit"
    );

    let mut at_v1 = m.scan_at(ts_v1);
    at_v1.sort();
    assert_eq!(at_v1, vec![(1, vec![1, 100]), (2, vec![2, 200])]);
    assert_eq!(m.scan_at(m.begin_snapshot()), vec![(1, vec![1, 101])]);

    // The clock resumed past every pre-restart timestamp.
    assert!(m.begin_snapshot() >= ts_after_delete);
    m.update(1, vec![1, 102]).unwrap();
    let ts_v3 = m.begin_snapshot();
    assert!(ts_v3 > ts_after_delete);
    assert_eq!(m.get_at(1, ts_v2), Some(vec![1, 101]));
    assert_eq!(m.get_at(1, ts_v3), Some(vec![1, 102]));
}

// ── B23 ──────────────────────────────────────────────────────────────────────

/// Insert, flush, delete, flush, compact, restart: the row stays gone.
#[test]
fn delete_survives_flush_compaction_and_restart() {
    let dir = TempDir::new().unwrap();
    {
        let m = open_manager(dir.path(), 1_000_000, 0);
        m.enable_manifest().unwrap();
        for pk in 10..40u64 {
            m.insert(pk, vec![pk, 1]).unwrap();
        }
        m.flush().unwrap();
        m.delete(10).unwrap();
        m.flush().unwrap();
        // One partition, 1/30 garbage: the default planner leaves it alone, so
        // the deleted row's bytes are still on disk after this.
        m.run_compaction(CompactionStrategy::default(), WriteOptions::default())
            .unwrap();
        assert_eq!(m.get(10).unwrap(), None);
    }
    let m = reopen(dir.path());
    assert_eq!(m.get(10).unwrap(), None, "deleted row came back on restart");
    assert_eq!(m.get(11).unwrap(), Some(vec![11, 1]));

    // A re-insert after the delete is not hidden by the old tombstone.
    m.insert(10, vec![10, 2]).unwrap();
    m.flush().unwrap();
    drop(m);
    let m = reopen(dir.path());
    assert_eq!(m.get(10).unwrap(), Some(vec![10, 2]));
}

/// A delete or overwrite that lands while compaction is merging must not be
/// undone when the merged output is installed.
#[test]
fn writes_racing_compaction_are_not_undone_by_the_install() {
    let dir = TempDir::new().unwrap();
    {
        let m = Arc::new(open_manager(dir.path(), 1_000_000, 0));
        four_partitions_insert(&m);
        let racer = Arc::clone(&m);
        m.set_compaction_hook(Some(Arc::new(move || {
            racer.delete(0).unwrap();
            racer.upsert(1, vec![1, 999]).unwrap();
        })));
        let outcomes = m
            .run_compaction(CompactionStrategy::default(), WriteOptions::default())
            .unwrap();
        m.set_compaction_hook(None);
        assert_eq!(outcomes.len(), 1, "expected one merge");
        assert_eq!(m.get(0).unwrap(), None, "install resurrected a delete");
        assert_eq!(m.get(1).unwrap(), Some(vec![1, 999]), "install rolled back");
        assert_eq!(m.get(2).unwrap(), Some(vec![2, 1]));
        m.flush().unwrap();
    }
    let m = reopen(dir.path());
    assert_eq!(m.get(0).unwrap(), None);
    assert_eq!(m.get(1).unwrap(), Some(vec![1, 999]));
    assert_eq!(m.get(301).unwrap(), Some(vec![301, 1]));
}

/// Four partitions to merge, so the default planner runs one job.
fn four_partitions_insert(m: &TxnManager) {
    m.enable_manifest().unwrap();
    for batch in 0..4u64 {
        for i in 0..10u64 {
            let pk = batch * 100 + i;
            m.insert(pk, vec![pk, 1]).unwrap();
        }
        m.flush().unwrap();
    }
}

// ── B24 ──────────────────────────────────────────────────────────────────────

/// While compaction is paused between building its output and installing it,
/// point reads and writes complete promptly.
#[test]
fn reads_and_writes_proceed_while_compaction_is_in_flight() {
    let dir = TempDir::new().unwrap();
    let m = Arc::new(open_manager(dir.path(), 1_000_000, 0));
    four_partitions_insert(&m);
    // Warm the reader cache so the in-flight read exercises it.
    assert_eq!(m.get(5).unwrap(), Some(vec![5, 1]));

    let (entered_tx, entered_rx) = mpsc::channel::<()>();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let entered_tx = Mutex::new(entered_tx);
    let release_rx = Mutex::new(release_rx);
    m.set_compaction_hook(Some(Arc::new(move || {
        entered_tx.lock().unwrap().send(()).unwrap();
        release_rx.lock().unwrap().recv().unwrap();
    })));

    let compactor = {
        let m = Arc::clone(&m);
        thread::spawn(move || {
            m.run_compaction(CompactionStrategy::default(), WriteOptions::default())
                .unwrap()
        })
    };
    entered_rx
        .recv_timeout(Duration::from_secs(30))
        .expect("compaction never reached its hook");

    let (done_tx, done_rx) = mpsc::channel();
    {
        let m = Arc::clone(&m);
        thread::spawn(move || {
            let read = m.get(205).unwrap();
            let cold = m.get(305).unwrap();
            m.insert(9_999, vec![9_999, 1]).unwrap();
            done_tx.send((read, cold)).unwrap();
        });
    }
    let (read, cold) = done_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("a read or write blocked behind compaction");
    assert_eq!(read, Some(vec![205, 1]));
    assert_eq!(cold, Some(vec![305, 1]));

    release_tx.send(()).unwrap();
    let outcomes = compactor.join().unwrap();
    assert_eq!(outcomes.len(), 1);
    assert_eq!(m.get(205).unwrap(), Some(vec![205, 1]));
    assert_eq!(m.get(9_999).unwrap(), Some(vec![9_999, 1]));
}
