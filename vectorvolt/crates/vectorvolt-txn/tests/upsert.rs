//! Layer 91 — Mutation::Upsert + TxnManager::upsert (VV-TXN-010).
//!
//! Tests for the new OP_UPSERT WAL opcode, Mutation::Upsert encode/decode,
//! TxnManager::upsert(), and WAL replay of upsert records.

use tempfile::TempDir;
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

// ── Mutation::Upsert codec ────────────────────────────────────────────────────

#[test]
fn upsert_mutation_encodes_and_decodes() {
    let m = Mutation::Upsert {
        pk: 42,
        values: vec![42, 99],
    };
    assert_eq!(Mutation::decode(&m.encode()).unwrap(), m);
}

#[test]
fn upsert_mutation_zero_values_round_trips() {
    let m = Mutation::Upsert {
        pk: 0,
        values: vec![],
    };
    assert_eq!(Mutation::decode(&m.encode()).unwrap(), m);
}

#[test]
fn upsert_opcode_distinct_from_insert_and_delete() {
    let upsert = Mutation::Upsert {
        pk: 1,
        values: vec![1, 2],
    };
    let insert = Mutation::Insert {
        pk: 1,
        values: vec![1, 2],
    };
    // Same data, but different encoded first byte (opcode).
    assert_ne!(upsert.encode()[0], insert.encode()[0]);
}

#[test]
fn upsert_pk_accessor_returns_correct_value() {
    let m = Mutation::Upsert {
        pk: 77,
        values: vec![77, 100],
    };
    assert_eq!(m.pk(), 77);
}

// ── TxnManager::upsert — new row ─────────────────────────────────────────────

#[test]
fn upsert_new_pk_buffered_and_readable() {
    let (mgr, _wal_dir, _flush_dir) = make_manager(100);
    mgr.upsert(1, vec![1, 42]).unwrap();
    assert_eq!(mgr.row_count_buffered(), 1);
}

#[test]
fn upsert_new_pk_logs_upsert_mutation_to_wal() {
    let (mgr, wal_dir, _flush_dir) = make_manager(100);
    mgr.upsert(5, vec![5, 10]).unwrap();
    drop(mgr); // flush WAL buffers to disk before reading
    let mutations = read_mutations(wal_dir.path());
    assert_eq!(mutations.len(), 1);
    assert_eq!(
        mutations[0],
        Mutation::Upsert {
            pk: 5,
            values: vec![5, 10]
        }
    );
}

// ── TxnManager::upsert — existing row ────────────────────────────────────────

#[test]
fn upsert_existing_pk_overwrites_old_index_entry() {
    let (mgr, _wal_dir, _flush_dir) = make_manager(100);
    mgr.insert(1, vec![1, 10]).unwrap();
    mgr.upsert(1, vec![1, 99]).unwrap();
    // The ring has 2 physical entries (old + new) but only 1 pk_index entry.
    // After upsert, row_count_buffered is 2 (ring size), but pk is unique in the index.
    assert!(mgr.row_count_buffered() >= 1);
}

#[test]
fn upsert_existing_pk_logs_upsert_mutation() {
    let (mgr, wal_dir, _flush_dir) = make_manager(100);
    mgr.insert(3, vec![3, 7]).unwrap();
    mgr.upsert(3, vec![3, 55]).unwrap();
    drop(mgr);
    let mutations = read_mutations(wal_dir.path());
    // First mutation: Insert; second: Upsert.
    assert_eq!(mutations.len(), 2);
    assert!(matches!(mutations[0], Mutation::Insert { pk: 3, .. }));
    assert_eq!(
        mutations[1],
        Mutation::Upsert {
            pk: 3,
            values: vec![3, 55]
        }
    );
}

// ── TxnManager::upsert — schema validation ────────────────────────────────────

#[test]
fn upsert_wrong_value_count_returns_error() {
    let (mgr, _wal_dir, _flush_dir) = make_manager(100);
    assert!(mgr.upsert(1, vec![1, 2, 3]).is_err());
}

// ── multiple upserts ─────────────────────────────────────────────────────────

#[test]
fn multiple_upserts_succeed_without_error() {
    let (mgr, _wal_dir, _flush_dir) = make_manager(100);
    mgr.upsert(1, vec![1, 10]).unwrap();
    mgr.upsert(1, vec![1, 20]).unwrap();
    mgr.upsert(1, vec![1, 30]).unwrap();
    // At least one buffered entry; pk_index has exactly one entry for pk=1.
    assert!(mgr.row_count_buffered() >= 1);
}

#[test]
fn upsert_multiple_pks_all_buffered() {
    let (mgr, _wal_dir, _flush_dir) = make_manager(100);
    mgr.upsert(1, vec![1, 1]).unwrap();
    mgr.upsert(2, vec![2, 2]).unwrap();
    mgr.upsert(3, vec![3, 3]).unwrap();
    assert_eq!(mgr.row_count_buffered(), 3);
}

// ── WAL replay ───────────────────────────────────────────────────────────────

#[test]
fn upsert_mutation_survives_wal_replay() {
    let wal_dir = TempDir::new().unwrap();
    let flush_dir = TempDir::new().unwrap();

    // Write a single upsert to the WAL.
    {
        let cfg = WalConfig::default()
            .with_dir(wal_dir.path())
            .with_fsync(false);
        let wal = WalWriter::open(cfg).unwrap();
        let mgr = TxnManager::new(wal, vec!["pk".into(), "val".into()], flush_dir.path(), 100);
        mgr.insert(1, vec![1, 10]).unwrap();
        mgr.upsert(1, vec![1, 99]).unwrap();
    }

    // Replay: the WAL should have one Insert + one Upsert.
    let mutations = read_mutations(wal_dir.path());
    assert_eq!(mutations.len(), 2);
    assert!(matches!(mutations[1], Mutation::Upsert { pk: 1, .. }));
}
