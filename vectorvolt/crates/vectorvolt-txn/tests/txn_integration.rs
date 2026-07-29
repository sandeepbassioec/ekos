//! Integration tests for `vectorvolt-txn`.
//!
//! Each test spins up a fresh WAL + flush directory, runs operations through
//! TxnManager, and verifies the in-memory index + flushed partition files.

use tempfile::TempDir;
use vectorvolt_storage::PartitionReader;
use vectorvolt_txn::{Mutation, TxnManager};
use vectorvolt_wal::{WalConfig, WalReader, WalWriter};

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
        vec!["pk".into(), "age".into()],
        flush_dir.path(),
        flush_threshold,
    );
    (mgr, wal_dir, flush_dir)
}

// ── tests ─────────────────────────────────────────────────────────────────────

/// INSERT logs to WAL and updates the in-memory index.
#[test]
fn insert_updates_index() {
    let (mgr, _wal_dir, _flush_dir) = make_manager(100);

    mgr.insert(1, vec![1, 25]).unwrap();
    mgr.insert(2, vec![2, 30]).unwrap();
    mgr.insert(3, vec![3, 42]).unwrap();

    assert_eq!(mgr.row_count_buffered(), 3);
    assert!(mgr.pk_index.read().get(1).is_some());
    assert!(mgr.pk_index.read().get(2).is_some());
    assert!(mgr.pk_index.read().get(3).is_some());
    assert!(mgr.fk_filter.read().contains(1));
    assert!(mgr.fk_filter.read().contains(3));
}

/// Schema mismatch returns an error.
#[test]
fn insert_schema_mismatch_errors() {
    let (mgr, _wal_dir, _flush_dir) = make_manager(100);
    let err = mgr.insert(1, vec![1, 25, 999]).unwrap_err(); // 3 values, schema has 2
    assert!(matches!(
        err,
        vectorvolt_txn::TxnError::SchemaMismatch {
            expected: 2,
            got: 3
        }
    ));
}

/// DELETE removes pk from index and WAL-logs the op.
#[test]
fn delete_removes_from_index() {
    let (mgr, _wal_dir, _flush_dir) = make_manager(100);

    mgr.insert(5, vec![5, 99]).unwrap();
    assert!(mgr.pk_index.read().get(5).is_some());

    mgr.delete(5).unwrap();
    assert!(mgr.pk_index.read().get(5).is_none());
}

/// DELETE on a missing pk returns PkNotFound.
#[test]
fn delete_missing_pk_errors() {
    let (mgr, _wal_dir, _flush_dir) = make_manager(100);
    assert!(matches!(
        mgr.delete(99).unwrap_err(),
        vectorvolt_txn::TxnError::PkNotFound(99)
    ));
}

/// Explicit flush writes a .vvsst file and updates RowDescriptors.
#[test]
fn flush_writes_vvsst_file() {
    let (mgr, _wal_dir, _flush_dir) = make_manager(100);

    mgr.insert(10, vec![10, 20]).unwrap();
    mgr.insert(11, vec![11, 21]).unwrap();
    mgr.insert(12, vec![12, 22]).unwrap();

    let res = mgr.flush().unwrap().expect("buffer was not empty");

    assert_eq!(res.row_count, 3);
    assert_eq!(res.file_id, 0);
    assert_eq!(res.partition_id, 0);
    assert!(res.path.exists());

    // Verify the file is readable via PartitionReader.
    let reader = PartitionReader::open(&res.path).unwrap();
    assert_eq!(reader.row_count(), 3);
    assert_eq!(reader.col_count(), 2);

    // After flush, the CuckooMap should point to the real (0, 0) partition.
    let desc = mgr.pk_index.read().get(10).unwrap();
    assert_eq!(desc.file_id(), 0);
    assert_eq!(desc.partition_id(), 0);
    assert_eq!(desc.row_offset(), 0);

    // total_rows_flushed should be 3.
    assert_eq!(mgr.total_rows_flushed(), 3);
}

/// Flush on empty buffer returns None.
#[test]
fn flush_empty_returns_none() {
    let (mgr, _wal_dir, _flush_dir) = make_manager(100);
    assert!(mgr.flush().unwrap().is_none());
}

/// Auto-flush triggers when buffer hits threshold.
#[test]
fn auto_flush_on_threshold() {
    let (mgr, _wal_dir, flush_dir) = make_manager(3); // flush every 3 rows

    // Insert 3 rows — should auto-flush on the 3rd.
    mgr.insert(1, vec![1, 10]).unwrap();
    mgr.insert(2, vec![2, 20]).unwrap();
    mgr.insert(3, vec![3, 30]).unwrap(); // triggers auto-flush

    assert_eq!(mgr.row_count_buffered(), 0);
    assert_eq!(mgr.total_rows_flushed(), 3);

    // A .vvsst file must exist.
    let files: Vec<_> = std::fs::read_dir(flush_dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("vvsst"))
        .collect();
    assert_eq!(files.len(), 1);
}

/// WAL contains exactly the right number of frames after inserts.
#[test]
fn wal_records_all_mutations() {
    let wal_dir = TempDir::new().unwrap();
    let flush_dir = TempDir::new().unwrap();
    let cfg = WalConfig::default()
        .with_dir(wal_dir.path())
        .with_fsync(false);
    let wal = WalWriter::open(cfg.clone()).unwrap();
    let mgr = TxnManager::new(wal, vec!["pk".into(), "v".into()], flush_dir.path(), 100);

    mgr.insert(1, vec![1, 100]).unwrap();
    mgr.insert(2, vec![2, 200]).unwrap();
    mgr.delete(1).unwrap();

    // Flush WAL buffers to disk so the reader can see them.
    drop(mgr);

    let reader = WalReader::open(cfg);
    let frames = reader.read_all().unwrap();
    // 3 mutations → 3 Data frames.
    assert_eq!(frames.len(), 3);
}

/// Mutation encode/decode round-trip (via WAL replay).
#[test]
fn wal_payload_round_trips() {
    let wal_dir = TempDir::new().unwrap();
    let flush_dir = TempDir::new().unwrap();
    let cfg = WalConfig::default()
        .with_dir(wal_dir.path())
        .with_fsync(false);
    let wal = WalWriter::open(cfg.clone()).unwrap();
    let mgr = TxnManager::new(wal, vec!["pk".into(), "v".into()], flush_dir.path(), 100);

    mgr.insert(7, vec![7, 777]).unwrap();
    mgr.delete(7).unwrap();
    drop(mgr);

    let reader = WalReader::open(cfg);
    let frames = reader.read_all().unwrap();
    assert_eq!(frames.len(), 2);

    let m0 = Mutation::decode(&frames[0].payload).unwrap();
    assert_eq!(
        m0,
        Mutation::Insert {
            pk: 7,
            values: vec![7, 777]
        }
    );

    let m1 = Mutation::decode(&frames[1].payload).unwrap();
    assert_eq!(m1, Mutation::Delete { pk: 7 });
}

/// Two sequential flushes create two distinct partition files.
#[test]
fn two_flushes_produce_two_partitions() {
    let (mgr, _wal_dir, flush_dir) = make_manager(100);

    mgr.insert(1, vec![1, 10]).unwrap();
    mgr.insert(2, vec![2, 20]).unwrap();
    let r0 = mgr.flush().unwrap().unwrap();
    assert_eq!(r0.partition_id, 0);

    mgr.insert(3, vec![3, 30]).unwrap();
    let r1 = mgr.flush().unwrap().unwrap();
    assert_eq!(r1.partition_id, 1);

    let files: Vec<_> = std::fs::read_dir(flush_dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("vvsst"))
        .collect();
    assert_eq!(files.len(), 2);

    assert_eq!(mgr.total_rows_flushed(), 3);
}
