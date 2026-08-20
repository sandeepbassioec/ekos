//! Layer 97 — TxnManager::scan() full row iterator (VV-TXN-023).
//!
//! Tests full-table scan across pending write buffer, flushed partitions,
//! post-delete consistency, batch mutations, and upsert overwrite visibility.

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

fn sorted_rows(mut rows: Vec<(u64, Vec<u64>)>) -> Vec<(u64, Vec<u64>)> {
    rows.sort_by_key(|(pk, _)| *pk);
    rows
}

// ── basic correctness ─────────────────────────────────────────────────────────

#[test]
fn scan_empty_table_returns_empty() {
    let (mgr, _wd, _fd) = make_manager(1000);
    assert_eq!(mgr.scan().unwrap(), vec![]);
}

#[test]
fn scan_single_row() {
    let (mgr, _wd, _fd) = make_manager(1000);
    mgr.insert(1, vec![1, 42]).unwrap();
    assert_eq!(mgr.scan().unwrap(), vec![(1, vec![1, 42])]);
}

#[test]
fn scan_multiple_rows_all_values_correct() {
    let (mgr, _wd, _fd) = make_manager(1000);
    for i in 1u64..=5 {
        mgr.insert(i, vec![i, i * 10]).unwrap();
    }
    let rows = sorted_rows(mgr.scan().unwrap());
    assert_eq!(
        rows,
        vec![
            (1, vec![1, 10]),
            (2, vec![2, 20]),
            (3, vec![3, 30]),
            (4, vec![4, 40]),
            (5, vec![5, 50]),
        ]
    );
}

#[test]
fn scan_excludes_deleted_rows() {
    let (mgr, _wd, _fd) = make_manager(1000);
    mgr.insert(1, vec![1, 10]).unwrap();
    mgr.insert(2, vec![2, 20]).unwrap();
    mgr.insert(3, vec![3, 30]).unwrap();
    mgr.delete(2).unwrap();
    let rows = sorted_rows(mgr.scan().unwrap());
    assert_eq!(rows, vec![(1, vec![1, 10]), (3, vec![3, 30])]);
}

#[test]
fn scan_row_count_matches_insert_minus_delete() {
    let (mgr, _wd, _fd) = make_manager(1000);
    for i in 1u64..=10 {
        mgr.insert(i, vec![i, i]).unwrap();
    }
    for i in 1u64..=5 {
        mgr.delete(i).unwrap();
    }
    assert_eq!(mgr.scan().unwrap().len(), 5);
}

// ── flushed partitions ────────────────────────────────────────────────────────

#[test]
fn scan_after_flush_returns_all_rows() {
    // flush_threshold=3 forces a flush after 3 inserts.
    let (mgr, _wd, _fd) = make_manager(3);
    for i in 1u64..=3 {
        mgr.insert(i, vec![i, i * 100]).unwrap();
    }
    mgr.flush().unwrap();
    let rows = sorted_rows(mgr.scan().unwrap());
    assert_eq!(
        rows,
        vec![(1, vec![1, 100]), (2, vec![2, 200]), (3, vec![3, 300])]
    );
}

#[test]
fn scan_spans_pending_and_flushed_rows() {
    let (mgr, _wd, _fd) = make_manager(1000);
    mgr.insert(1, vec![1, 10]).unwrap();
    mgr.insert(2, vec![2, 20]).unwrap();
    mgr.flush().unwrap();
    // Two more rows remain pending.
    mgr.insert(3, vec![3, 30]).unwrap();
    mgr.insert(4, vec![4, 40]).unwrap();
    let rows = sorted_rows(mgr.scan().unwrap());
    assert_eq!(
        rows,
        vec![
            (1, vec![1, 10]),
            (2, vec![2, 20]),
            (3, vec![3, 30]),
            (4, vec![4, 40]),
        ]
    );
}

// ── upsert visibility ─────────────────────────────────────────────────────────

#[test]
fn scan_reflects_upsert_overwrite() {
    let (mgr, _wd, _fd) = make_manager(1000);
    mgr.insert(1, vec![1, 10]).unwrap();
    mgr.upsert(1, vec![1, 99]).unwrap();
    assert_eq!(mgr.scan().unwrap(), vec![(1, vec![1, 99])]);
}

// ── batch mutations ───────────────────────────────────────────────────────────

#[test]
fn scan_after_insert_batch() {
    let (mgr, _wd, _fd) = make_manager(1000);
    mgr.insert_batch(vec![(1, vec![1, 10]), (2, vec![2, 20]), (3, vec![3, 30])])
        .unwrap();
    assert_eq!(mgr.scan().unwrap().len(), 3);
}

#[test]
fn scan_after_delete_batch_excludes_deleted() {
    let (mgr, _wd, _fd) = make_manager(1000);
    for i in 1u64..=4 {
        mgr.insert(i, vec![i, i]).unwrap();
    }
    mgr.delete_batch(vec![1, 3]).unwrap();
    let rows = sorted_rows(mgr.scan().unwrap());
    assert_eq!(rows, vec![(2, vec![2, 2]), (4, vec![4, 4])]);
}

#[test]
fn scan_after_upsert_batch_shows_new_values() {
    let (mgr, _wd, _fd) = make_manager(1000);
    mgr.insert(10, vec![10, 1]).unwrap();
    mgr.insert(20, vec![20, 2]).unwrap();
    mgr.upsert_batch(vec![(10, vec![10, 99]), (20, vec![20, 88])])
        .unwrap();
    let rows = sorted_rows(mgr.scan().unwrap());
    assert_eq!(rows, vec![(10, vec![10, 99]), (20, vec![20, 88])]);
}

// ── large scan ───────────────────────────────────────────────────────────────

#[test]
fn scan_large_table_row_count_correct() {
    let (mgr, _wd, _fd) = make_manager(1000);
    for i in 1u64..=100 {
        mgr.insert(i, vec![i, i * 2]).unwrap();
    }
    assert_eq!(mgr.scan().unwrap().len(), 100);
}

// ── idempotency ───────────────────────────────────────────────────────────────

#[test]
fn scan_twice_returns_same_result() {
    let (mgr, _wd, _fd) = make_manager(1000);
    mgr.insert(1, vec![1, 10]).unwrap();
    mgr.insert(2, vec![2, 20]).unwrap();
    let first = sorted_rows(mgr.scan().unwrap());
    let second = sorted_rows(mgr.scan().unwrap());
    assert_eq!(first, second);
}
