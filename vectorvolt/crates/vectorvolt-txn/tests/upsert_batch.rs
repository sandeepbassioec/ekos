//! Layer 95 — Mutation::UpsertBatch + TxnManager::upsert_batch (VV-TXN-021).
//!
//! Tests for the OP_UPSERT_BATCH WAL opcode, Mutation::UpsertBatch encode/decode,
//! TxnManager::upsert_batch(), and WAL replay of batch upsert records.

use tempfile::TempDir;
use vectorvolt_txn::{TxnManager, ops::Mutation, ops::OP_UPSERT_BATCH};
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

// ── Mutation::UpsertBatch codec ───────────────────────────────────────────────

#[test]
fn upsert_batch_opcode_is_0x05() {
    assert_eq!(OP_UPSERT_BATCH, 0x05);
}

#[test]
fn upsert_batch_encode_decode_round_trip() {
    let m = Mutation::UpsertBatch {
        rows: vec![(1, vec![1, 10]), (2, vec![2, 20]), (3, vec![3, 30])],
    };
    assert_eq!(Mutation::decode(&m.encode()).unwrap(), m);
}

#[test]
fn upsert_batch_empty_round_trips() {
    let m = Mutation::UpsertBatch { rows: vec![] };
    assert_eq!(Mutation::decode(&m.encode()).unwrap(), m);
}

#[test]
fn upsert_batch_pk_returns_first_pk() {
    let m = Mutation::UpsertBatch {
        rows: vec![(42, vec![42, 1]), (99, vec![99, 2])],
    };
    assert_eq!(m.pk(), 42);
}

#[test]
fn upsert_batch_empty_pk_returns_zero() {
    let m = Mutation::UpsertBatch { rows: vec![] };
    assert_eq!(m.pk(), 0);
}

// ── TxnManager::upsert_batch ──────────────────────────────────────────────────

#[test]
fn upsert_batch_inserts_new_rows() {
    let (mgr, _wal_dir, _flush_dir) = make_manager(1000);
    mgr.upsert_batch(vec![(1, vec![1, 10]), (2, vec![2, 20])])
        .unwrap();
    assert!(mgr.pk_index.read().get(1).is_some());
    assert!(mgr.pk_index.read().get(2).is_some());
}

#[test]
fn upsert_batch_overwrites_existing_rows() {
    let (mgr, _wal_dir, _flush_dir) = make_manager(1000);
    mgr.insert(1, vec![1, 10]).unwrap();
    mgr.upsert_batch(vec![(1, vec![1, 99])]).unwrap();
    // PK 1 still present after overwrite.
    assert!(mgr.pk_index.read().get(1).is_some());
}

#[test]
fn upsert_batch_single_wal_frame() {
    let (mgr, wal_dir, _flush_dir) = make_manager(1000);
    mgr.upsert_batch(vec![(1, vec![1, 10]), (2, vec![2, 20])])
        .unwrap();
    drop(mgr);
    let mutations = read_mutations(wal_dir.path());
    assert_eq!(mutations.len(), 1);
    assert!(matches!(mutations[0], Mutation::UpsertBatch { .. }));
}

#[test]
fn upsert_batch_wal_contains_all_rows() {
    let (mgr, wal_dir, _flush_dir) = make_manager(1000);
    let rows = vec![(10, vec![10, 1]), (20, vec![20, 2])];
    mgr.upsert_batch(rows.clone()).unwrap();
    drop(mgr);
    let mutations = read_mutations(wal_dir.path());
    assert_eq!(mutations[0], Mutation::UpsertBatch { rows });
}

#[test]
fn upsert_batch_schema_mismatch_returns_error() {
    let (mgr, _wal_dir, _flush_dir) = make_manager(1000);
    // Schema has 2 columns; pass 3 values.
    assert!(mgr.upsert_batch(vec![(1, vec![1, 2, 3])]).is_err());
}

#[test]
fn upsert_batch_no_partial_write_on_schema_error() {
    let (mgr, _wal_dir, _flush_dir) = make_manager(1000);
    // First row ok, second row wrong; both must be rejected.
    let _ = mgr.upsert_batch(vec![(1, vec![1, 10]), (2, vec![2, 20, 99])]);
    assert!(mgr.pk_index.read().get(1).is_none());
    assert!(mgr.pk_index.read().get(2).is_none());
}

// ── batch vs individual upserts: WAL frame count ──────────────────────────────

#[test]
fn upsert_batch_fewer_wal_frames_than_individual_upserts() {
    let (batch_mgr, batch_wal, _) = make_manager(1000);
    let (solo_mgr, solo_wal, _) = make_manager(1000);

    batch_mgr
        .upsert_batch(vec![(1, vec![1, 1]), (2, vec![2, 2]), (3, vec![3, 3])])
        .unwrap();
    drop(batch_mgr);

    for i in 1u64..=3 {
        solo_mgr.upsert(i, vec![i, i]).unwrap();
    }
    drop(solo_mgr);

    let batch_frames = read_mutations(batch_wal.path()).len();
    let solo_frames = read_mutations(solo_wal.path()).len();
    assert_eq!(batch_frames, 1);
    assert_eq!(solo_frames, 3);
}

// ── WAL replay ────────────────────────────────────────────────────────────────

#[test]
fn upsert_batch_survives_wal_replay() {
    let wal_dir = TempDir::new().unwrap();
    let flush_dir = TempDir::new().unwrap();

    {
        let cfg = WalConfig::default()
            .with_dir(wal_dir.path())
            .with_fsync(false);
        let wal = WalWriter::open(cfg).unwrap();
        let mgr = TxnManager::new(wal, vec!["pk".into(), "val".into()], flush_dir.path(), 1000);
        mgr.upsert_batch(vec![(1, vec![1, 10]), (2, vec![2, 20])])
            .unwrap();
    }

    let mutations = read_mutations(wal_dir.path());
    assert_eq!(mutations.len(), 1);
    assert!(matches!(
        &mutations[0],
        Mutation::UpsertBatch { rows } if rows.len() == 2
    ));
}
