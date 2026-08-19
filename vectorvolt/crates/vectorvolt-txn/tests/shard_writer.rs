//! Layer 86 — per-shard writer threads (VV-TXN-009).
//!
//! Tests the [`ShardWriter`] independently, then exercises it through
//! [`TxnManager`] to verify the full insert / drain / flush correctness path.

use parking_lot::{Mutex, RwLock};
use std::collections::HashMap;
use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_index::CuckooMap;
use vectorvolt_txn::TxnManager;
use vectorvolt_txn::shard_writer::ShardWriter;
use vectorvolt_wal::{WalConfig, WalWriter};

// ── ShardWriter unit tests ────────────────────────────────────────────────────

fn make_writer(threshold: usize, width: usize) -> (ShardWriter, Arc<RwLock<CuckooMap>>) {
    let pk_index = Arc::new(RwLock::new(CuckooMap::with_capacity(64)));
    let in_flight = Arc::new(Mutex::new(HashMap::new()));
    let sw = ShardWriter::new(threshold, width, Arc::clone(&pk_index), in_flight);
    (sw, pk_index)
}

#[test]
fn insert_returns_false_below_threshold() {
    let (sw, _idx) = make_writer(16, 2);
    let full = sw.insert(1, vec![1, 100]).unwrap();
    assert!(
        !full,
        "should not be full after one insert into threshold=16 writer"
    );
}

#[test]
fn insert_returns_true_at_threshold() {
    let (sw, _idx) = make_writer(4, 2);
    let mut full = false;
    for i in 0..4u64 {
        full = sw.insert(i, vec![i, i * 10]).unwrap();
    }
    assert!(full, "should signal full after threshold inserts");
}

#[test]
fn len_tracks_inserts() {
    let (sw, _idx) = make_writer(32, 2);
    assert_eq!(sw.len(), 0);
    sw.insert(1, vec![1, 10]).unwrap();
    sw.insert(2, vec![2, 20]).unwrap();
    assert_eq!(sw.len(), 2);
}

#[test]
fn drain_returns_all_rows() {
    let (sw, _idx) = make_writer(32, 2);
    sw.insert(10, vec![10, 100]).unwrap();
    sw.insert(20, vec![20, 200]).unwrap();
    let rows = sw.drain().unwrap();
    assert_eq!(rows.len(), 2, "drain must return all buffered rows");
}

#[test]
fn drain_empties_the_buffer() {
    let (sw, _idx) = make_writer(32, 2);
    sw.insert(1, vec![1, 10]).unwrap();
    let _ = sw.drain().unwrap();
    assert_eq!(sw.len(), 0, "buffer must be empty after drain");
}

#[test]
fn drain_on_empty_returns_empty_vec() {
    let (sw, _idx) = make_writer(32, 2);
    let rows = sw.drain().unwrap();
    assert!(
        rows.is_empty(),
        "drain of empty writer must return empty vec"
    );
}

#[test]
fn row_for_finds_buffered_row() {
    let (sw, _idx) = make_writer(32, 2);
    sw.insert(42, vec![42, 999]).unwrap();
    let found = sw.row_for(42, 0);
    assert!(found.is_some(), "row_for must find the just-inserted row");
    assert_eq!(found.unwrap(), vec![42, 999]);
}

#[test]
fn row_for_returns_none_for_wrong_pk() {
    let (sw, _idx) = make_writer(32, 2);
    sw.insert(1, vec![1, 10]).unwrap();
    // slot 0 holds pk=1; asking for pk=99 at offset 0 should return None
    let found = sw.row_for(99, 0);
    assert!(
        found.is_none(),
        "row_for must return None when pk does not match slot"
    );
}

#[test]
fn pk_index_populated_after_insert() {
    let (sw, idx) = make_writer(32, 2);
    sw.insert(7, vec![7, 77]).unwrap();
    let desc = idx.read().get(7);
    assert!(desc.is_some(), "pk_index must contain the key after insert");
}

#[test]
fn in_flight_populated_after_drain() {
    let pk_index = Arc::new(RwLock::new(CuckooMap::with_capacity(64)));
    let in_flight: Arc<Mutex<HashMap<u64, Vec<u64>>>> = Arc::new(Mutex::new(HashMap::new()));
    let sw = ShardWriter::new(32, 2, Arc::clone(&pk_index), Arc::clone(&in_flight));
    sw.insert(5, vec![5, 50]).unwrap();
    let _ = sw.drain().unwrap();
    let guard = in_flight.lock();
    assert!(
        guard.contains_key(&5),
        "in_flight must contain drained row before drain() returns"
    );
}

// ── TxnManager integration tests ─────────────────────────────────────────────

fn make_mgr(threshold: usize) -> (TxnManager, TempDir, TempDir) {
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
        threshold,
    );
    (mgr, wal_dir, flush_dir)
}

#[test]
fn txn_insert_and_flush_round_trip() {
    let (mgr, _wd, _fd) = make_mgr(64);
    mgr.insert(1, vec![1, 100]).unwrap();
    mgr.insert(2, vec![2, 200]).unwrap();
    let result = mgr.flush().unwrap();
    assert!(result.is_some(), "flush must return Some after inserts");
    assert_eq!(result.unwrap().row_count, 2);
}

#[test]
fn txn_flush_on_empty_returns_none() {
    let (mgr, _wd, _fd) = make_mgr(64);
    let result = mgr.flush().unwrap();
    assert!(result.is_none(), "flush on empty writer must return None");
}

#[test]
fn txn_row_count_buffered_tracks_inserts() {
    let (mgr, _wd, _fd) = make_mgr(64);
    assert_eq!(mgr.row_count_buffered(), 0);
    mgr.insert(1, vec![1, 10]).unwrap();
    assert_eq!(mgr.row_count_buffered(), 1);
}

#[test]
fn txn_buffered_or_inflight_finds_row_before_flush() {
    let (mgr, _wd, _fd) = make_mgr(64);
    mgr.insert(42, vec![42, 420]).unwrap();
    let found = mgr.buffered_or_inflight_row(42, 0);
    assert!(
        found.is_some(),
        "buffered_or_inflight_row must find the row before flush"
    );
}
