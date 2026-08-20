//! Layer 93 — Mutation::InsertBatch + TxnManager::insert_batch (VV-TXN-019).
//!
//! Tests for the OP_INSERT_BATCH WAL opcode, Mutation::InsertBatch encode/decode,
//! TxnManager::insert_batch(), and WAL replay of batch records.

use tempfile::TempDir;
use vectorvolt_txn::{TxnManager, ops::Mutation, ops::OP_INSERT_BATCH};
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
        vec!["pk".into(), "val".into()],
        flush_dir.path(),
        flush_threshold,
    );
    (mgr, wal_dir, flush_dir)
}

fn read_mutations(wal_dir: &std::path::Path) -> Vec<Mutation> {
    let cfg = WalConfig::default().with_dir(wal_dir).with_fsync(false);
    let reader = WalReader::open(cfg);
    reader
        .read_all()
        .unwrap()
        .into_iter()
        .filter_map(|f| Mutation::decode(&f.payload).ok())
        .collect()
}

// ── Mutation::InsertBatch codec ───────────────────────────────────────────────

#[test]
fn insert_batch_opcode_is_0x03() {
    assert_eq!(OP_INSERT_BATCH, 0x03);
}

#[test]
fn insert_batch_encode_decode_round_trip() {
    let m = Mutation::InsertBatch {
        rows: vec![(1, vec![1, 10]), (2, vec![2, 20]), (3, vec![3, 30])],
    };
    assert_eq!(Mutation::decode(&m.encode()).unwrap(), m);
}

#[test]
fn insert_batch_empty_rows_round_trips() {
    let m = Mutation::InsertBatch { rows: vec![] };
    assert_eq!(Mutation::decode(&m.encode()).unwrap(), m);
}

#[test]
fn insert_batch_pk_returns_first_pk() {
    let m = Mutation::InsertBatch {
        rows: vec![(42, vec![42, 1]), (99, vec![99, 2])],
    };
    assert_eq!(m.pk(), 42);
}

#[test]
fn insert_batch_empty_pk_returns_zero() {
    let m = Mutation::InsertBatch { rows: vec![] };
    assert_eq!(m.pk(), 0);
}

// ── TxnManager::insert_batch — basic ─────────────────────────────────────────

#[test]
fn insert_batch_all_rows_buffered() {
    let (mgr, _wal_dir, _flush_dir) = make_manager(1000);
    mgr.insert_batch(vec![(1, vec![1, 10]), (2, vec![2, 20]), (3, vec![3, 30])])
        .unwrap();
    assert_eq!(mgr.row_count_buffered(), 3);
}

#[test]
fn insert_batch_single_wal_frame_written() {
    let (mgr, wal_dir, _flush_dir) = make_manager(1000);
    mgr.insert_batch(vec![(1, vec![1, 10]), (2, vec![2, 20])])
        .unwrap();
    drop(mgr);
    let mutations = read_mutations(wal_dir.path());
    assert_eq!(mutations.len(), 1);
    assert!(matches!(mutations[0], Mutation::InsertBatch { .. }));
}

#[test]
fn insert_batch_wal_payload_contains_all_rows() {
    let (mgr, wal_dir, _flush_dir) = make_manager(1000);
    let batch = vec![(10, vec![10, 100]), (20, vec![20, 200])];
    mgr.insert_batch(batch.clone()).unwrap();
    drop(mgr);
    let mutations = read_mutations(wal_dir.path());
    assert_eq!(mutations[0], Mutation::InsertBatch { rows: batch });
}

// ── schema validation ─────────────────────────────────────────────────────────

#[test]
fn insert_batch_wrong_value_count_rejects_whole_batch() {
    let (mgr, _wal_dir, _flush_dir) = make_manager(1000);
    // Row with 3 values instead of 2.
    let result = mgr.insert_batch(vec![(1, vec![1, 10, 99])]);
    assert!(result.is_err());
    assert_eq!(mgr.row_count_buffered(), 0);
}

#[test]
fn insert_batch_mixed_valid_invalid_rejected() {
    let (mgr, _wal_dir, _flush_dir) = make_manager(1000);
    // First row valid, second row has wrong count — entire batch should fail.
    let result = mgr.insert_batch(vec![(1, vec![1, 10]), (2, vec![2, 20, 99])]);
    assert!(result.is_err());
}

// ── WAL replay ────────────────────────────────────────────────────────────────

#[test]
fn insert_batch_survives_wal_replay() {
    let wal_dir = TempDir::new().unwrap();
    let flush_dir = TempDir::new().unwrap();

    {
        let cfg = WalConfig::default()
            .with_dir(wal_dir.path())
            .with_fsync(false);
        let wal = WalWriter::open(cfg).unwrap();
        let mgr = TxnManager::new(wal, vec!["pk".into(), "val".into()], flush_dir.path(), 1000);
        mgr.insert_batch(vec![(1, vec![1, 10]), (2, vec![2, 20])])
            .unwrap();
    }

    let mutations = read_mutations(wal_dir.path());
    assert_eq!(mutations.len(), 1);
    assert!(matches!(
        &mutations[0],
        Mutation::InsertBatch { rows } if rows.len() == 2
    ));
}

// ── batch vs individual inserts produce same WAL row count ────────────────────

#[test]
fn insert_batch_fewer_wal_frames_than_individual_inserts() {
    let (batch_mgr, batch_wal, _) = make_manager(1000);
    let (solo_mgr, solo_wal, _) = make_manager(1000);

    batch_mgr
        .insert_batch(vec![(1, vec![1, 1]), (2, vec![2, 2]), (3, vec![3, 3])])
        .unwrap();
    drop(batch_mgr);

    solo_mgr.insert(1, vec![1, 1]).unwrap();
    solo_mgr.insert(2, vec![2, 2]).unwrap();
    solo_mgr.insert(3, vec![3, 3]).unwrap();
    drop(solo_mgr);

    let batch_frames = read_mutations(batch_wal.path()).len();
    let solo_frames = read_mutations(solo_wal.path()).len();
    // Batch uses 1 WAL frame; solo uses 3.
    assert_eq!(batch_frames, 1);
    assert_eq!(solo_frames, 3);
    assert!(batch_frames < solo_frames);
}

#[test]
fn insert_batch_empty_is_no_op() {
    let (mgr, wal_dir, _flush_dir) = make_manager(1000);
    mgr.insert_batch(vec![]).unwrap();
    let count = mgr.row_count_buffered();
    drop(mgr);
    // Even an empty batch writes one WAL frame with row_count=0.
    let mutations = read_mutations(wal_dir.path());
    assert_eq!(mutations.len(), 1);
    assert_eq!(count, 0);
}
