/// Layer 25 — conditional write integration tests.
///
/// Tests cover `insert_if_absent`, `compare_and_swap`, and `upsert`,
/// including concurrent correctness: two threads racing on the same PK
/// must produce exactly one winner.
use std::sync::{Arc, Barrier};
use std::thread;
use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine, UpsertResult};

fn open() -> (TableEngine, TempDir) {
    let dir = TempDir::new().unwrap();
    let eng = TableEngine::open_with(
        vec!["pk".into(), "val".into(), "extra".into()],
        dir.path(),
        EngineOptions::new(1024).with_durability(Durability::WriteOnly),
    )
    .unwrap();
    (eng, dir)
}

// ── insert_if_absent ──────────────────────────────────────────────────────────

#[test]
fn insert_if_absent_inserts_new_row() {
    let (eng, _dir) = open();
    let inserted = eng.insert_if_absent(1, vec![1, 42, 0]).unwrap();
    assert!(inserted, "should return true for a new PK");
    let row = eng.get(1).unwrap().unwrap();
    assert_eq!(row.values[1], 42);
}

#[test]
fn insert_if_absent_skips_existing_pk() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 10, 0]).unwrap();
    let inserted = eng.insert_if_absent(1, vec![1, 99, 0]).unwrap();
    assert!(!inserted, "should return false when PK exists");
    // original values must be unchanged
    let row = eng.get(1).unwrap().unwrap();
    assert_eq!(row.values[1], 10);
}

#[test]
fn insert_if_absent_different_pks_both_succeed() {
    let (eng, _dir) = open();
    assert!(eng.insert_if_absent(1, vec![1, 10, 0]).unwrap());
    assert!(eng.insert_if_absent(2, vec![2, 20, 0]).unwrap());
    assert_eq!(eng.get(1).unwrap().unwrap().values[1], 10);
    assert_eq!(eng.get(2).unwrap().unwrap().values[1], 20);
}

#[test]
fn insert_if_absent_concurrent_same_pk_exactly_one_wins() {
    let dir = TempDir::new().unwrap();
    let eng = Arc::new(
        TableEngine::open_with(
            vec!["pk".into(), "val".into(), "extra".into()],
            dir.path(),
            EngineOptions::new(1024).with_durability(Durability::WriteOnly),
        )
        .unwrap(),
    );

    let n = 8;
    let barrier = Arc::new(Barrier::new(n));
    let mut handles = Vec::new();

    for i in 0..n {
        let eng = Arc::clone(&eng);
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait(); // all threads start simultaneously
            eng.insert_if_absent(42, vec![42, i as u64, 0]).unwrap()
        }));
    }

    let results: Vec<bool> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    let winners = results.iter().filter(|&&r| r).count();
    assert_eq!(winners, 1, "exactly one thread must win insert_if_absent");
    // The stored row must be one of the valid inserts.
    let row = eng.get(42).unwrap().unwrap();
    assert_eq!(row.pk, 42);
}

// ── compare_and_swap ──────────────────────────────────────────────────────────

#[test]
fn cas_succeeds_when_values_match() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 10, 5]).unwrap();
    let swapped = eng
        .compare_and_swap(1, &[1, 10, 5], vec![1, 99, 5])
        .unwrap();
    assert!(swapped);
    assert_eq!(eng.get(1).unwrap().unwrap().values[1], 99);
}

#[test]
fn cas_fails_when_values_mismatch() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 10, 5]).unwrap();
    let swapped = eng
        .compare_and_swap(1, &[1, 99, 5], vec![1, 77, 5])
        .unwrap();
    assert!(!swapped, "should return false when expected doesn't match");
    // values unchanged
    assert_eq!(eng.get(1).unwrap().unwrap().values[1], 10);
}

#[test]
fn cas_fails_for_missing_pk() {
    let (eng, _dir) = open();
    let swapped = eng
        .compare_and_swap(99, &[99, 0, 0], vec![99, 1, 0])
        .unwrap();
    assert!(!swapped);
    assert!(eng.get(99).unwrap().is_none());
}

#[test]
fn cas_partial_expected_only_checks_given_prefix() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 10, 5]).unwrap();
    // expected prefix [1, 10] matches first two values
    let swapped = eng.compare_and_swap(1, &[1, 10], vec![1, 20, 5]).unwrap();
    assert!(swapped);
    assert_eq!(eng.get(1).unwrap().unwrap().values[1], 20);
}

#[test]
fn cas_concurrent_only_one_succeeds() {
    let dir = TempDir::new().unwrap();
    let eng = Arc::new(
        TableEngine::open_with(
            vec!["pk".into(), "val".into(), "extra".into()],
            dir.path(),
            EngineOptions::new(1024).with_durability(Durability::WriteOnly),
        )
        .unwrap(),
    );

    // Seed initial row.
    eng.insert(1, vec![1, 0, 0]).unwrap();

    let n = 6;
    let barrier = Arc::new(Barrier::new(n));
    let mut handles = Vec::new();

    for _ in 0..n {
        let eng = Arc::clone(&eng);
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait();
            eng.compare_and_swap(1, &[1, 0, 0], vec![1, 1, 0]).unwrap()
        }));
    }

    let results: Vec<bool> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    let winners = results.iter().filter(|&&r| r).count();
    assert_eq!(
        winners, 1,
        "exactly one CAS must win on the same expected value"
    );
}

// ── upsert ────────────────────────────────────────────────────────────────────

#[test]
fn upsert_inserts_when_absent() {
    let (eng, _dir) = open();
    let result = eng.upsert(5, vec![5, 42, 0]).unwrap();
    assert_eq!(result, UpsertResult::Inserted);
    assert_eq!(eng.get(5).unwrap().unwrap().values[1], 42);
}

#[test]
fn upsert_overwrites_existing_and_returns_old_row() {
    let (eng, _dir) = open();
    eng.insert(5, vec![5, 10, 3]).unwrap();
    let result = eng.upsert(5, vec![5, 99, 7]).unwrap();
    match result {
        UpsertResult::Updated(old) => {
            assert_eq!(old.pk, 5);
            assert_eq!(old.values[1], 10);
        }
        UpsertResult::Inserted => panic!("expected Updated, got Inserted"),
    }
    assert_eq!(eng.get(5).unwrap().unwrap().values[1], 99);
}

#[test]
fn upsert_idempotent_on_same_values() {
    let (eng, _dir) = open();
    eng.upsert(1, vec![1, 10, 0]).unwrap();
    let r2 = eng.upsert(1, vec![1, 10, 0]).unwrap();
    assert!(matches!(r2, UpsertResult::Updated(_)));
    assert_eq!(eng.get(1).unwrap().unwrap().values[1], 10);
}

#[test]
fn upsert_multiple_rows_correct() {
    let (eng, _dir) = open();
    for pk in 1u64..=5 {
        assert_eq!(
            eng.upsert(pk, vec![pk, pk * 10, 0]).unwrap(),
            UpsertResult::Inserted
        );
    }
    for pk in 1u64..=5 {
        let r = eng.upsert(pk, vec![pk, pk * 100, 0]).unwrap();
        assert!(matches!(r, UpsertResult::Updated(_)));
    }
    for pk in 1u64..=5 {
        assert_eq!(eng.get(pk).unwrap().unwrap().values[1], pk * 100);
    }
}
