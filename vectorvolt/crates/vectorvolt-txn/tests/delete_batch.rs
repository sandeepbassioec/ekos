//! Layer 94 — Mutation::DeleteBatch + TxnManager::delete_batch (VV-TXN-020).
//!
//! Tests for the OP_DELETE_BATCH WAL opcode, Mutation::DeleteBatch encode/decode,
//! TxnManager::delete_batch(), and WAL replay of batch delete records.

use tempfile::TempDir;
use vectorvolt_txn::{TxnManager, ops::Mutation, ops::OP_DELETE_BATCH};
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

// ── Mutation::DeleteBatch codec ───────────────────────────────────────────────

#[test]
fn delete_batch_opcode_is_0x04() {
    assert_eq!(OP_DELETE_BATCH, 0x04);
}

#[test]
fn delete_batch_encode_decode_round_trip() {
    let m = Mutation::DeleteBatch {
        pks: vec![1, 2, 3, 100],
    };
    assert_eq!(Mutation::decode(&m.encode()).unwrap(), m);
}

#[test]
fn delete_batch_empty_round_trips() {
    let m = Mutation::DeleteBatch { pks: vec![] };
    assert_eq!(Mutation::decode(&m.encode()).unwrap(), m);
}

#[test]
fn delete_batch_pk_returns_first_pk() {
    let m = Mutation::DeleteBatch {
        pks: vec![42, 99, 7],
    };
    assert_eq!(m.pk(), 42);
}

#[test]
fn delete_batch_empty_pk_returns_zero() {
    let m = Mutation::DeleteBatch { pks: vec![] };
    assert_eq!(m.pk(), 0);
}

// ── TxnManager::delete_batch ──────────────────────────────────────────────────

#[test]
fn delete_batch_removes_all_rows() {
    let (mgr, _wal_dir, _flush_dir) = make_manager(1000);
    mgr.insert(1, vec![1, 10]).unwrap();
    mgr.insert(2, vec![2, 20]).unwrap();
    mgr.insert(3, vec![3, 30]).unwrap();
    mgr.delete_batch(vec![1, 2, 3]).unwrap();
    // All PKs removed from index.
    assert!(mgr.pk_index.read().get(1).is_none());
    assert!(mgr.pk_index.read().get(2).is_none());
    assert!(mgr.pk_index.read().get(3).is_none());
}

#[test]
fn delete_batch_single_wal_frame() {
    let (mgr, wal_dir, _flush_dir) = make_manager(1000);
    mgr.insert(1, vec![1, 10]).unwrap();
    mgr.insert(2, vec![2, 20]).unwrap();
    mgr.delete_batch(vec![1, 2]).unwrap();
    drop(mgr);
    let mutations = read_mutations(wal_dir.path());
    // 2 inserts + 1 delete_batch = 3 frames.
    assert_eq!(mutations.len(), 3);
    assert!(matches!(mutations[2], Mutation::DeleteBatch { .. }));
}

#[test]
fn delete_batch_wal_contains_all_pks() {
    let (mgr, wal_dir, _flush_dir) = make_manager(1000);
    mgr.insert(10, vec![10, 1]).unwrap();
    mgr.insert(20, vec![20, 2]).unwrap();
    mgr.delete_batch(vec![10, 20]).unwrap();
    drop(mgr);
    let mutations = read_mutations(wal_dir.path());
    assert_eq!(mutations[2], Mutation::DeleteBatch { pks: vec![10, 20] });
}

#[test]
fn delete_batch_missing_pk_returns_error() {
    let (mgr, _wal_dir, _flush_dir) = make_manager(1000);
    mgr.insert(1, vec![1, 10]).unwrap();
    // PK 99 does not exist.
    assert!(mgr.delete_batch(vec![1, 99]).is_err());
}

#[test]
fn delete_batch_atomicity_no_partial_delete_on_error() {
    let (mgr, _wal_dir, _flush_dir) = make_manager(1000);
    mgr.insert(1, vec![1, 10]).unwrap();
    mgr.insert(2, vec![2, 20]).unwrap();
    // PK 99 missing — entire batch fails, PKs 1 and 2 stay in index.
    let _ = mgr.delete_batch(vec![1, 99, 2]);
    assert!(mgr.pk_index.read().get(1).is_some());
    assert!(mgr.pk_index.read().get(2).is_some());
}

// ── batch vs individual deletes: WAL frame count ──────────────────────────────

#[test]
fn delete_batch_fewer_wal_frames_than_individual_deletes() {
    let (batch_mgr, batch_wal, _) = make_manager(1000);
    let (solo_mgr, solo_wal, _) = make_manager(1000);

    for i in 1u64..=3 {
        batch_mgr.insert(i, vec![i, i]).unwrap();
        solo_mgr.insert(i, vec![i, i]).unwrap();
    }
    batch_mgr.delete_batch(vec![1, 2, 3]).unwrap();
    drop(batch_mgr);

    for i in 1u64..=3 {
        solo_mgr.delete(i).unwrap();
    }
    drop(solo_mgr);

    let batch_frames = read_mutations(batch_wal.path()).len();
    let solo_frames = read_mutations(solo_wal.path()).len();
    // Both have 3 insert frames; batch has 1 delete frame, solo has 3.
    assert_eq!(batch_frames, 4);
    assert_eq!(solo_frames, 6);
}

// ── WAL replay ────────────────────────────────────────────────────────────────

#[test]
fn delete_batch_survives_wal_replay() {
    let wal_dir = TempDir::new().unwrap();
    let flush_dir = TempDir::new().unwrap();

    {
        let cfg = WalConfig::default()
            .with_dir(wal_dir.path())
            .with_fsync(false);
        let wal = WalWriter::open(cfg).unwrap();
        let mgr = TxnManager::new(wal, vec!["pk".into(), "val".into()], flush_dir.path(), 1000);
        mgr.insert(1, vec![1, 10]).unwrap();
        mgr.insert(2, vec![2, 20]).unwrap();
        mgr.delete_batch(vec![1, 2]).unwrap();
    }

    let mutations = read_mutations(wal_dir.path());
    // 2 inserts + 1 delete_batch.
    assert_eq!(mutations.len(), 3);
    assert!(matches!(
        &mutations[2],
        Mutation::DeleteBatch { pks } if pks == &[1u64, 2u64]
    ));
}

#[test]
fn delete_batch_empty_is_no_op() {
    let (mgr, wal_dir, _flush_dir) = make_manager(1000);
    mgr.delete_batch(vec![]).unwrap();
    drop(mgr);
    // One WAL frame for empty batch.
    let mutations = read_mutations(wal_dir.path());
    assert_eq!(mutations.len(), 1);
    assert!(matches!(mutations[0], Mutation::DeleteBatch { .. }));
}
