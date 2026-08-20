//! Layer 96 — TxnManager::get() + scan_all_pks() read path (VV-TXN-022).
//!
//! Tests point-read from the pending write buffer, the in-flight map, and
//! flushed .vvsst partition files.  Also covers scan_all_pks() snapshot.

use tempfile::TempDir;
use vectorvolt_txn::TxnManager;
use vectorvolt_wal::{WalConfig, WalWriter};

// ── helpers ───────────────────────────────────────────────────────────────────

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

// ── TxnManager::get — pending (write buffer) ─────────────────────────────────

#[test]
fn get_returns_none_for_missing_pk() {
    let (mgr, _wd, _fd) = make_manager(1000);
    assert!(mgr.get(42).unwrap().is_none());
}

#[test]
fn get_returns_row_after_insert() {
    let (mgr, _wd, _fd) = make_manager(1000);
    mgr.insert(1, vec![1, 100]).unwrap();
    assert_eq!(mgr.get(1).unwrap(), Some(vec![1, 100]));
}

#[test]
fn get_returns_none_after_delete() {
    let (mgr, _wd, _fd) = make_manager(1000);
    mgr.insert(1, vec![1, 100]).unwrap();
    mgr.delete(1).unwrap();
    assert!(mgr.get(1).unwrap().is_none());
}

#[test]
fn get_returns_updated_row_after_upsert() {
    let (mgr, _wd, _fd) = make_manager(1000);
    mgr.insert(1, vec![1, 10]).unwrap();
    mgr.upsert(1, vec![1, 99]).unwrap();
    assert_eq!(mgr.get(1).unwrap(), Some(vec![1, 99]));
}

#[test]
fn get_multiple_distinct_pks() {
    let (mgr, _wd, _fd) = make_manager(1000);
    mgr.insert(10, vec![10, 1]).unwrap();
    mgr.insert(20, vec![20, 2]).unwrap();
    mgr.insert(30, vec![30, 3]).unwrap();
    assert_eq!(mgr.get(10).unwrap(), Some(vec![10, 1]));
    assert_eq!(mgr.get(20).unwrap(), Some(vec![20, 2]));
    assert_eq!(mgr.get(30).unwrap(), Some(vec![30, 3]));
}

// ── TxnManager::get — flushed partition ───────────────────────────────────────

#[test]
fn get_returns_row_after_flush() {
    // flush_threshold=1 so the first insert triggers a flush to disk.
    let (mgr, _wd, _fd) = make_manager(1);
    mgr.insert(1, vec![1, 42]).unwrap();
    mgr.flush().unwrap();
    assert_eq!(mgr.get(1).unwrap(), Some(vec![1, 42]));
}

#[test]
fn get_returns_correct_row_from_multi_row_flush() {
    let (mgr, _wd, _fd) = make_manager(5);
    for i in 1u64..=5 {
        mgr.insert(i, vec![i, i * 10]).unwrap();
    }
    mgr.flush().unwrap();
    for i in 1u64..=5 {
        assert_eq!(mgr.get(i).unwrap(), Some(vec![i, i * 10]));
    }
}

#[test]
fn get_none_after_delete_from_flushed_partition() {
    let (mgr, _wd, _fd) = make_manager(1);
    mgr.insert(7, vec![7, 70]).unwrap();
    mgr.flush().unwrap();
    mgr.delete(7).unwrap();
    assert!(mgr.get(7).unwrap().is_none());
}

// ── TxnManager::scan_all_pks ──────────────────────────────────────────────────

#[test]
fn scan_all_pks_empty() {
    let (mgr, _wd, _fd) = make_manager(1000);
    assert!(mgr.scan_all_pks().is_empty());
}

#[test]
fn scan_all_pks_after_inserts() {
    let (mgr, _wd, _fd) = make_manager(1000);
    mgr.insert(1, vec![1, 10]).unwrap();
    mgr.insert(2, vec![2, 20]).unwrap();
    mgr.insert(3, vec![3, 30]).unwrap();
    let mut pks = mgr.scan_all_pks();
    pks.sort();
    assert_eq!(pks, vec![1, 2, 3]);
}

#[test]
fn get_returns_row_inserted_via_insert_batch() {
    let (mgr, _wd, _fd) = make_manager(1000);
    mgr.insert_batch(vec![(5, vec![5, 50]), (6, vec![6, 60])])
        .unwrap();
    assert_eq!(mgr.get(5).unwrap(), Some(vec![5, 50]));
    assert_eq!(mgr.get(6).unwrap(), Some(vec![6, 60]));
}

#[test]
fn get_returns_row_inserted_via_upsert_batch() {
    let (mgr, _wd, _fd) = make_manager(1000);
    mgr.insert(10, vec![10, 1]).unwrap();
    mgr.upsert_batch(vec![(10, vec![10, 99]), (11, vec![11, 2])])
        .unwrap();
    assert_eq!(mgr.get(10).unwrap(), Some(vec![10, 99]));
    assert_eq!(mgr.get(11).unwrap(), Some(vec![11, 2]));
}

#[test]
fn scan_all_pks_excludes_deleted_pk() {
    let (mgr, _wd, _fd) = make_manager(1000);
    mgr.insert(1, vec![1, 10]).unwrap();
    mgr.insert(2, vec![2, 20]).unwrap();
    mgr.delete(1).unwrap();
    let mut pks = mgr.scan_all_pks();
    pks.sort();
    assert_eq!(pks, vec![2]);
}
