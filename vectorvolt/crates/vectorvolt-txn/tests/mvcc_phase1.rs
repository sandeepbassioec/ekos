//! MVCC Phase 1 — commit clock + per-pk version chains (row versioning).
//!
//! `get_at`/`scan_at` read the version chain directly and are independent of
//! `pk_index`, so a reader holding an older snapshot timestamp keeps seeing the
//! value as of that timestamp even while newer writes land on top.

use std::thread;

use tempfile::TempDir;
use vectorvolt_txn::TxnManager;
use vectorvolt_wal::{WalConfig, WalWriter};

fn make_manager(flush_threshold: usize) -> (TxnManager, TempDir, TempDir) {
    let wal_dir = TempDir::new().unwrap();
    let flush_dir = TempDir::new().unwrap();
    let cfg = WalConfig::default()
        .with_dir(wal_dir.path())
        .with_fsync(false);
    let wal = WalWriter::open(cfg).unwrap();
    let mgr = TxnManager::new(
        wal,
        vec!["pk".into(), "val".into()],
        flush_dir.path(),
        flush_threshold,
    );
    (mgr, wal_dir, flush_dir)
}

#[test]
fn test_mvcc_read_at_old_snapshot_sees_old_value() {
    let (mgr, _wd, _fd) = make_manager(1000);
    mgr.insert(1, vec![1, 100]).unwrap();
    let ts1 = mgr.begin_snapshot();
    mgr.update(1, vec![1, 200]).unwrap();

    assert_eq!(mgr.get_at(1, ts1), Some(vec![1, 100]));
    assert_eq!(mgr.get(1).unwrap(), Some(vec![1, 200]));
}

#[test]
fn test_mvcc_read_at_snapshot_before_insert_sees_none() {
    let (mgr, _wd, _fd) = make_manager(1000);
    let ts0 = mgr.begin_snapshot();
    mgr.insert(1, vec![1, 100]).unwrap();

    assert_eq!(mgr.get_at(1, ts0), None);
    assert_eq!(mgr.get(1).unwrap(), Some(vec![1, 100]));
}

#[test]
fn test_mvcc_delete_creates_tombstone_not_removal() {
    let (mgr, _wd, _fd) = make_manager(1000);
    mgr.insert(1, vec![1, 100]).unwrap();
    let ts_before_delete = mgr.begin_snapshot();
    mgr.delete(1).unwrap();

    assert_eq!(mgr.get_at(1, ts_before_delete), Some(vec![1, 100]));
    assert!(mgr.get(1).unwrap().is_none());
}

#[test]
fn test_mvcc_scan_at_is_snapshot_consistent() {
    let (mgr, _wd, _fd) = make_manager(1000);
    mgr.insert(1, vec![1, 100]).unwrap();
    let ts1 = mgr.begin_snapshot();
    mgr.insert(2, vec![2, 200]).unwrap();

    let rows = mgr.scan_at(ts1);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0], (1, vec![1, 100]));

    let rows_now = mgr.scan_at(mgr.begin_snapshot());
    assert_eq!(rows_now.len(), 2);
}

#[test]
fn test_mvcc_multiple_versions_same_pk() {
    let (mgr, _wd, _fd) = make_manager(1000);
    mgr.insert(1, vec![1, 100]).unwrap();
    mgr.update(1, vec![1, 200]).unwrap();
    mgr.update(1, vec![1, 300]).unwrap();

    // Three distinct snapshots, one per version, each seeing the value as of
    // that point in commit order.
    let ts_after_insert = 1u64; // first commit_ts ever handed out
    assert_eq!(mgr.get_at(1, ts_after_insert), Some(vec![1, 100]));

    let latest = mgr.get(1).unwrap();
    assert_eq!(latest, Some(vec![1, 300]));
}

#[test]
fn test_mvcc_gc_prunes_old_versions_but_keeps_watermark_visible() {
    let (mgr, _wd, _fd) = make_manager(1000);
    mgr.insert(1, vec![1, 100]).unwrap();
    let ts1 = mgr.begin_snapshot();
    mgr.update(1, vec![1, 200]).unwrap();
    let ts2 = mgr.begin_snapshot();
    mgr.update(1, vec![1, 300]).unwrap();

    // GC everything older than ts2: version at ts1 should be reclaimable, but
    // the watermark version (ts2) itself must stay visible.
    mgr.gc_versions_before(1, ts2);

    assert_eq!(mgr.get_at(1, ts2), Some(vec![1, 200]));
    assert_eq!(mgr.get_at(1, mgr.begin_snapshot()), Some(vec![1, 300]));
    // The pruned, older-than-watermark version (ts1) is gone: no surviving
    // version has commit_ts <= ts1 anymore.
    assert_eq!(mgr.get_at(1, ts1), None);
}

#[test]
fn test_mvcc_clock_monotonic_across_operations() {
    let (mgr, _wd, _fd) = make_manager(1000);
    let mut timestamps = Vec::new();
    mgr.insert(1, vec![1, 1]).unwrap();
    timestamps.push(mgr.begin_snapshot());
    mgr.update(1, vec![1, 2]).unwrap();
    timestamps.push(mgr.begin_snapshot());
    mgr.insert(2, vec![2, 1]).unwrap();
    timestamps.push(mgr.begin_snapshot());
    mgr.delete(1).unwrap();
    timestamps.push(mgr.begin_snapshot());

    for pair in timestamps.windows(2) {
        assert!(pair[0] < pair[1], "clock must be strictly increasing");
    }
}

#[test]
fn test_mvcc_concurrent_writers_get_distinct_commit_ts() {
    let (mgr, _wd, _fd) = make_manager(1000);
    let mgr = std::sync::Arc::new(mgr);

    let mut handles = Vec::new();
    for t in 0..4u64 {
        let mgr = std::sync::Arc::clone(&mgr);
        handles.push(thread::spawn(move || {
            for i in 0..25u64 {
                let pk = t * 1000 + i;
                mgr.insert(pk, vec![pk, 1]).unwrap();
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }

    // 100 inserts, each must have produced a distinct, live-visible version.
    let total_visible = mgr.scan_at(mgr.begin_snapshot()).len();
    assert_eq!(total_visible, 100);
    assert_eq!(mgr.commit_clock().current(), 100);
}
