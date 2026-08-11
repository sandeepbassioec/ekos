/// Layer 18 — MVCC transaction integration tests.
///
/// Each test opens a fresh `TableEngine` and exercises the begin/insert/get/
/// delete/commit/rollback API to verify correctness of:
/// - read-your-own-writes within a transaction
/// - uncommitted writes invisible to concurrent readers
/// - rollback discards pending mutations
/// - commit makes all mutations durably visible
/// - isolation between two concurrent transactions
use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};

fn open() -> (TableEngine, TempDir) {
    let dir = TempDir::new().unwrap();
    let eng = TableEngine::open_with(
        vec!["pk".into(), "val".into()],
        dir.path(),
        EngineOptions::new(1_048_576).with_durability(Durability::WriteOnly),
    )
    .unwrap();
    (eng, dir)
}

// ── begin ─────────────────────────────────────────────────────────────────────

#[test]
fn begin_returns_unique_ids() {
    let (eng, _dir) = open();
    let t1 = eng.begin();
    let t2 = eng.begin();
    let t3 = eng.begin();
    assert_ne!(t1, t2);
    assert_ne!(t2, t3);
}

// ── get_txn with no prior insert falls through to engine ──────────────────────

#[test]
fn get_txn_sees_committed_engine_rows() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 42]).unwrap();
    let txn = eng.begin();
    let row = eng.get_txn(txn, 1).unwrap().unwrap();
    assert_eq!(row.values[1], 42);
    eng.rollback_txn(txn).unwrap();
}

// ── read-your-own-writes ──────────────────────────────────────────────────────

#[test]
fn get_txn_sees_own_pending_insert() {
    let (eng, _dir) = open();
    let txn = eng.begin();
    eng.insert_txn(txn, 7, vec![7, 99]).unwrap();
    let row = eng.get_txn(txn, 7).unwrap().unwrap();
    assert_eq!(row.pk, 7);
    assert_eq!(row.values[1], 99);
    eng.rollback_txn(txn).unwrap();
}

#[test]
fn get_txn_respects_pending_delete() {
    let (eng, _dir) = open();
    eng.insert(5, vec![5, 10]).unwrap();
    let txn = eng.begin();
    eng.delete_txn(txn, 5).unwrap();
    assert!(eng.get_txn(txn, 5).unwrap().is_none());
    eng.rollback_txn(txn).unwrap();
}

// ── isolation: uncommitted writes invisible outside the transaction ───────────

#[test]
fn uncommitted_insert_not_visible_via_engine_get() {
    let (eng, _dir) = open();
    let txn = eng.begin();
    eng.insert_txn(txn, 3, vec![3, 77]).unwrap();
    // regular get bypasses the txn buffer
    assert!(
        eng.get(3).unwrap().is_none(),
        "uncommitted row must not be visible"
    );
    eng.rollback_txn(txn).unwrap();
}

// ── commit ────────────────────────────────────────────────────────────────────

#[test]
fn commit_makes_insert_visible() {
    let (eng, _dir) = open();
    let txn = eng.begin();
    eng.insert_txn(txn, 10, vec![10, 55]).unwrap();
    eng.commit_txn(txn).unwrap();
    let row = eng.get(10).unwrap().unwrap();
    assert_eq!(row.values[1], 55);
}

#[test]
fn commit_applies_delete() {
    let (eng, _dir) = open();
    eng.insert(20, vec![20, 1]).unwrap();
    let txn = eng.begin();
    eng.delete_txn(txn, 20).unwrap();
    eng.commit_txn(txn).unwrap();
    assert!(
        eng.get(20).unwrap().is_none(),
        "row must be gone after commit"
    );
}

#[test]
fn commit_multi_mutation_batch() {
    let (eng, _dir) = open();
    let txn = eng.begin();
    for pk in 1u64..=5 {
        eng.insert_txn(txn, pk, vec![pk, pk * 10]).unwrap();
    }
    eng.commit_txn(txn).unwrap();
    for pk in 1u64..=5 {
        let row = eng.get(pk).unwrap().unwrap();
        assert_eq!(row.values[1], pk * 10);
    }
}

// ── rollback ──────────────────────────────────────────────────────────────────

#[test]
fn rollback_discards_inserts() {
    let (eng, _dir) = open();
    let txn = eng.begin();
    eng.insert_txn(txn, 99, vec![99, 1]).unwrap();
    eng.rollback_txn(txn).unwrap();
    assert!(
        eng.get(99).unwrap().is_none(),
        "rolled-back insert must vanish"
    );
}

#[test]
fn rollback_txn_not_found_returns_error() {
    let (eng, _dir) = open();
    let result = eng.rollback_txn(9999);
    assert!(result.is_err(), "rolling back unknown txn must fail");
}

#[test]
fn commit_txn_not_found_returns_error() {
    let (eng, _dir) = open();
    let result = eng.commit_txn(9999);
    assert!(result.is_err(), "committing unknown txn must fail");
}

// ── concurrent isolation ──────────────────────────────────────────────────────

#[test]
fn two_concurrent_txns_are_isolated() {
    let eng = Arc::new({
        let dir = tempfile::TempDir::new().unwrap();
        // SAFETY: we keep dir alive via Box::leak for this test
        let dir = Box::leak(Box::new(dir));
        TableEngine::open_with(
            vec!["pk".into(), "val".into()],
            dir.path(),
            EngineOptions::new(1_048_576).with_durability(Durability::WriteOnly),
        )
        .unwrap()
    });

    let t1 = eng.begin();
    let t2 = eng.begin();

    eng.insert_txn(t1, 1, vec![1, 100]).unwrap();
    eng.insert_txn(t2, 1, vec![1, 200]).unwrap();

    // Each txn sees its own write, not the other's
    assert_eq!(eng.get_txn(t1, 1).unwrap().unwrap().values[1], 100);
    assert_eq!(eng.get_txn(t2, 1).unwrap().unwrap().values[1], 200);

    // Commit t1 first; t2 still sees its own pending value before commit
    eng.commit_txn(t1).unwrap();
    assert_eq!(eng.get_txn(t2, 1).unwrap().unwrap().values[1], 200);

    // After t2 commits, the committed engine value is t2's
    eng.commit_txn(t2).unwrap();
    assert_eq!(eng.get(1).unwrap().unwrap().values[1], 200);
}
