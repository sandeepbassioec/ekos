//! Layer 98 — Partition routing by PK hash (VV-ENG-010).
//!
//! Tests select_partition() determinism + distribution, and TxnManager with
//! partition_count > 1: flush writes separate files, get/scan still work.

use tempfile::TempDir;
use vectorvolt_txn::{TxnManager, select_partition};
use vectorvolt_wal::{WalConfig, WalWriter};

// ── helpers ───────────────────────────────────────────────────────────────────

fn make_manager(flush_threshold: usize, partition_count: usize) -> (TxnManager, TempDir, TempDir) {
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
    )
    .with_partition_count(partition_count);
    (mgr, wal_dir, flush_dir)
}

// ── select_partition ──────────────────────────────────────────────────────────

#[test]
fn select_partition_single_always_zero() {
    for pk in [0u64, 1, 42, 999, u64::MAX] {
        assert_eq!(select_partition(pk, 1), 0);
    }
}

#[test]
fn select_partition_result_in_range() {
    for n in 1..=8usize {
        for pk in 0u64..200 {
            assert!(select_partition(pk, n) < n);
        }
    }
}

#[test]
fn select_partition_deterministic() {
    for pk in 0u64..50 {
        assert_eq!(select_partition(pk, 4), select_partition(pk, 4));
    }
}

#[test]
fn select_partition_all_partitions_reachable() {
    let n = 4;
    let mut seen = [false; 4];
    for pk in 0u64..1000 {
        seen[select_partition(pk, n)] = true;
    }
    assert!(seen.iter().all(|&s| s));
}

// ── TxnManager with partition_count=1 (default) ───────────────────────────────

#[test]
fn single_partition_flush_one_file() {
    let (mgr, _wd, fd) = make_manager(1000, 1);
    mgr.insert(1, vec![1, 10]).unwrap();
    mgr.insert(2, vec![2, 20]).unwrap();
    mgr.flush().unwrap();
    // One .vvsst file should exist.
    let count = std::fs::read_dir(fd.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "vvsst").unwrap_or(false))
        .count();
    assert_eq!(count, 1);
}

// ── TxnManager with partition_count=2 ────────────────────────────────────────

#[test]
fn two_partitions_flush_writes_multiple_files() {
    let (mgr, _wd, fd) = make_manager(1000, 2);
    // Insert enough rows to guarantee both partitions get at least one row.
    for pk in 0u64..20 {
        mgr.insert(pk, vec![pk, pk]).unwrap();
    }
    mgr.flush().unwrap();
    // Should have 2 .vvsst files (one per non-empty partition group).
    let count = std::fs::read_dir(fd.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "vvsst").unwrap_or(false))
        .count();
    assert_eq!(count, 2);
}

#[test]
fn two_partitions_get_returns_correct_row() {
    let (mgr, _wd, _fd) = make_manager(1000, 2);
    for pk in 0u64..20 {
        mgr.insert(pk, vec![pk, pk * 10]).unwrap();
    }
    mgr.flush().unwrap();
    for pk in 0u64..20 {
        assert_eq!(mgr.get(pk).unwrap(), Some(vec![pk, pk * 10]));
    }
}

#[test]
fn two_partitions_scan_returns_all_rows() {
    let (mgr, _wd, _fd) = make_manager(1000, 2);
    for pk in 0u64..20 {
        mgr.insert(pk, vec![pk, pk]).unwrap();
    }
    mgr.flush().unwrap();
    assert_eq!(mgr.scan().unwrap().len(), 20);
}

#[test]
fn two_partitions_flush_result_row_count_correct() {
    let (mgr, _wd, _fd) = make_manager(1000, 2);
    for pk in 0u64..10 {
        mgr.insert(pk, vec![pk, pk]).unwrap();
    }
    let result = mgr.flush().unwrap().unwrap();
    assert_eq!(result.row_count, 10);
}

#[test]
fn rows_route_deterministically_by_pk() {
    // Two rows that hash to partition 0 and two that hash to partition 1
    // must land in separate files.
    let (mgr, _wd, fd) = make_manager(1000, 2);
    // Find two PKs that route to different partitions.
    let pk_0 = (0u64..1000)
        .find(|&pk| select_partition(pk, 2) == 0)
        .unwrap();
    let pk_1 = (0u64..1000)
        .find(|&pk| select_partition(pk, 2) == 1)
        .unwrap();
    mgr.insert(pk_0, vec![pk_0, 0]).unwrap();
    mgr.insert(pk_1, vec![pk_1, 1]).unwrap();
    mgr.flush().unwrap();
    let count = std::fs::read_dir(fd.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "vvsst").unwrap_or(false))
        .count();
    assert_eq!(count, 2);
}
