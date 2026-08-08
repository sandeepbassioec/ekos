/// Integration tests for `TableEngine::secondary_scan` and `fk_exists`.
///
/// These test the query path that wires the in-memory SecondaryIndex and
/// CuckooFilter (maintained by the write path) to the engine's read API.
use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};

fn open(dir: &TempDir) -> Arc<TableEngine> {
    Arc::new(
        TableEngine::open_with(
            vec!["pk".into(), "age".into(), "score".into()],
            dir.path(),
            EngineOptions::new(10_000).with_durability(Durability::WriteOnly),
        )
        .unwrap(),
    )
}

// ── secondary_scan ────────────────────────────────────────────────────────────

#[test]
fn secondary_scan_returns_rows_matching_value() {
    let dir = TempDir::new().unwrap();
    let engine = open(&dir);

    // Insert rows with varying `age` (col 1).
    engine.insert(1, vec![1, 30, 100]).unwrap();
    engine.insert(2, vec![2, 40, 200]).unwrap();
    engine.insert(3, vec![3, 30, 300]).unwrap(); // age == 30, same as pk=1

    let rows = engine.secondary_scan(1, 30).unwrap();
    let mut pks: Vec<u64> = rows.iter().map(|r| r[0]).collect();
    pks.sort_unstable();
    assert_eq!(pks, vec![1, 3], "both rows with age=30 must be returned");
}

#[test]
fn secondary_scan_returns_empty_for_unknown_value() {
    let dir = TempDir::new().unwrap();
    let engine = open(&dir);

    engine.insert(1, vec![1, 30, 100]).unwrap();

    let rows = engine.secondary_scan(1, 999).unwrap();
    assert!(rows.is_empty(), "no rows have age=999");
}

#[test]
fn secondary_scan_works_on_any_non_pk_column() {
    let dir = TempDir::new().unwrap();
    let engine = open(&dir);

    engine.insert(10, vec![10, 50, 77]).unwrap();
    engine.insert(20, vec![20, 60, 77]).unwrap(); // same score as pk=10
    engine.insert(30, vec![30, 70, 88]).unwrap();

    // col 2 = score
    let rows = engine.secondary_scan(2, 77).unwrap();
    let mut pks: Vec<u64> = rows.iter().map(|r| r[0]).collect();
    pks.sort_unstable();
    assert_eq!(pks, vec![10, 20]);
}

#[test]
fn secondary_scan_returns_full_rows_not_just_pks() {
    let dir = TempDir::new().unwrap();
    let engine = open(&dir);

    engine.insert(5, vec![5, 42, 99]).unwrap();

    let rows = engine.secondary_scan(1, 42).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0], vec![5, 42, 99]);
}

#[test]
fn secondary_scan_after_delete_omits_deleted_row() {
    let dir = TempDir::new().unwrap();
    let engine = open(&dir);

    engine.insert(1, vec![1, 30, 0]).unwrap();
    engine.insert(2, vec![2, 30, 0]).unwrap();
    engine.delete(1).unwrap();

    let rows = engine.secondary_scan(1, 30).unwrap();
    let pks: Vec<u64> = rows.iter().map(|r| r[0]).collect();
    assert!(!pks.contains(&1), "deleted pk=1 must not appear");
    assert!(pks.contains(&2), "live pk=2 must still appear");
}

#[test]
fn secondary_scan_across_flush_boundary() {
    let dir = TempDir::new().unwrap();
    let engine = open(&dir);

    engine.insert(100, vec![100, 55, 0]).unwrap();
    engine.flush().unwrap();
    engine.insert(200, vec![200, 55, 0]).unwrap(); // same age, different partition

    let rows = engine.secondary_scan(1, 55).unwrap();
    let mut pks: Vec<u64> = rows.iter().map(|r| r[0]).collect();
    pks.sort_unstable();
    // Both rows exist — one in SST, one in write buffer.
    assert!(pks.contains(&100) && pks.contains(&200), "pks = {pks:?}");
}

#[test]
fn secondary_scan_on_pk_column_zero_is_meaningless_but_does_not_panic() {
    let dir = TempDir::new().unwrap();
    let engine = open(&dir);

    engine.insert(7, vec![7, 10, 20]).unwrap();
    // col_idx=0 is the PK; the secondary index is NOT updated for col 0.
    // The result is empty (or may contain a false match due to the hash). This
    // test just verifies we do not panic.
    let _ = engine.secondary_scan(0, 7);
}

// ── fk_exists ─────────────────────────────────────────────────────────────────

#[test]
fn fk_exists_true_for_inserted_pk() {
    let dir = TempDir::new().unwrap();
    let engine = open(&dir);

    engine.insert(42, vec![42, 1, 2]).unwrap();

    assert!(engine.fk_exists(42), "pk=42 was inserted, must be found");
}

#[test]
fn fk_exists_false_for_never_inserted_pk() {
    let dir = TempDir::new().unwrap();
    let engine = open(&dir);

    engine.insert(1, vec![1, 1, 1]).unwrap();

    // This could theoretically be a false positive (~0.78%), but with a
    // specific far-away value the probability of collision is negligible.
    assert!(!engine.fk_exists(99999), "pk=99999 was never inserted");
}

#[test]
fn fk_exists_true_after_flush() {
    let dir = TempDir::new().unwrap();
    let engine = open(&dir);

    engine.insert(55, vec![55, 1, 2]).unwrap();
    engine.flush().unwrap();

    // Filter is maintained in memory independently of partition files.
    assert!(engine.fk_exists(55));
}

#[test]
fn fk_exists_true_for_deleted_pk() {
    let dir = TempDir::new().unwrap();
    let engine = open(&dir);

    engine.insert(77, vec![77, 1, 2]).unwrap();
    engine.delete(77).unwrap();

    // The filter is not cleaned on delete (by design — it's write-only).
    // Callers that need a definitive answer must also check the pk_index.
    assert!(
        engine.fk_exists(77),
        "filter still contains 77 after delete"
    );
}

#[test]
fn fk_exists_for_many_pks() {
    let dir = TempDir::new().unwrap();
    let engine = open(&dir);

    for pk in 1u64..=100 {
        engine.insert(pk, vec![pk, pk, pk]).unwrap();
    }

    let false_negatives = (1u64..=100).filter(|&pk| !engine.fk_exists(pk)).count();
    assert_eq!(false_negatives, 0, "no false negatives allowed");
}
