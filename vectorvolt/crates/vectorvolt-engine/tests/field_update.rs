/// Layer 32 — field-level atomic update integration tests.
///
/// Schema: pk | counter | score | flags
use std::sync::{Arc, Barrier};
use std::thread;
use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};

fn open() -> (TableEngine, TempDir) {
    let dir = TempDir::new().unwrap();
    let eng = TableEngine::open_with(
        vec![
            "pk".into(),
            "counter".into(),
            "score".into(),
            "flags".into(),
        ],
        dir.path(),
        EngineOptions::new(1024).with_durability(Durability::WriteOnly),
    )
    .unwrap();
    (eng, dir)
}

// ── update_field ──────────────────────────────────────────────────────────────

#[test]
fn update_field_returns_old_value() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 10, 100, 0]).unwrap();
    let old = eng.update_field(1, 2, 999).unwrap();
    assert_eq!(old, Some(100));
}

#[test]
fn update_field_stores_new_value() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 10, 100, 0]).unwrap();
    eng.update_field(1, 2, 999).unwrap();
    assert_eq!(eng.get(1).unwrap().unwrap().values[2], 999);
}

#[test]
fn update_field_leaves_other_columns_intact() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 42, 100, 7]).unwrap();
    eng.update_field(1, 2, 200).unwrap();
    let row = eng.get(1).unwrap().unwrap();
    assert_eq!(row.values[1], 42, "counter unchanged");
    assert_eq!(row.values[2], 200, "score updated");
    assert_eq!(row.values[3], 7, "flags unchanged");
}

#[test]
fn update_field_missing_pk_returns_none() {
    let (eng, _dir) = open();
    let result = eng.update_field(99, 1, 50).unwrap();
    assert_eq!(result, None);
}

#[test]
fn update_field_missing_pk_no_row_created() {
    let (eng, _dir) = open();
    eng.update_field(99, 1, 50).unwrap();
    assert!(eng.get(99).unwrap().is_none());
}

#[test]
fn update_field_bad_col_returns_error() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 0, 0, 0]).unwrap();
    assert!(eng.update_field(1, 99, 1).is_err());
}

#[test]
fn update_field_first_column() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 5, 50, 0]).unwrap();
    let old = eng.update_field(1, 1, 99).unwrap();
    assert_eq!(old, Some(5));
    assert_eq!(eng.get(1).unwrap().unwrap().values[1], 99);
}

#[test]
fn update_field_same_value_is_noop_semantically() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 42, 0, 0]).unwrap();
    let old = eng.update_field(1, 1, 42).unwrap();
    assert_eq!(old, Some(42));
    assert_eq!(eng.get(1).unwrap().unwrap().values[1], 42);
}

// ── increment_field ───────────────────────────────────────────────────────────

#[test]
fn increment_field_returns_new_value() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 10, 0, 0]).unwrap();
    let new_val = eng.increment_field(1, 1, 5).unwrap();
    assert_eq!(new_val, Some(15));
}

#[test]
fn increment_field_stores_new_value() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 10, 0, 0]).unwrap();
    eng.increment_field(1, 1, 5).unwrap();
    assert_eq!(eng.get(1).unwrap().unwrap().values[1], 15);
}

#[test]
fn increment_field_multiple_times() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 0, 0, 0]).unwrap();
    eng.increment_field(1, 1, 1).unwrap();
    eng.increment_field(1, 1, 1).unwrap();
    eng.increment_field(1, 1, 1).unwrap();
    assert_eq!(eng.get(1).unwrap().unwrap().values[1], 3);
}

#[test]
fn increment_field_wraps_on_overflow() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, u64::MAX, 0, 0]).unwrap();
    let new_val = eng.increment_field(1, 1, 1).unwrap();
    assert_eq!(new_val, Some(0)); // wrapping
}

#[test]
fn increment_field_missing_pk_returns_none() {
    let (eng, _dir) = open();
    assert_eq!(eng.increment_field(99, 1, 5).unwrap(), None);
}

#[test]
fn increment_field_bad_col_returns_error() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 0, 0, 0]).unwrap();
    assert!(eng.increment_field(1, 99, 1).is_err());
}

// ── decrement_field ───────────────────────────────────────────────────────────

#[test]
fn decrement_field_returns_new_value() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 100, 0, 0]).unwrap();
    let new_val = eng.decrement_field(1, 1, 30).unwrap();
    assert_eq!(new_val, Some(70));
}

#[test]
fn decrement_field_stores_new_value() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 100, 0, 0]).unwrap();
    eng.decrement_field(1, 1, 30).unwrap();
    assert_eq!(eng.get(1).unwrap().unwrap().values[1], 70);
}

#[test]
fn decrement_field_saturates_at_zero() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 5, 0, 0]).unwrap();
    let new_val = eng.decrement_field(1, 1, 100).unwrap();
    assert_eq!(new_val, Some(0)); // saturating, not wrapping
}

#[test]
fn decrement_field_exact_zero() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 50, 0, 0]).unwrap();
    let new_val = eng.decrement_field(1, 1, 50).unwrap();
    assert_eq!(new_val, Some(0));
}

#[test]
fn decrement_field_missing_pk_returns_none() {
    let (eng, _dir) = open();
    assert_eq!(eng.decrement_field(99, 1, 5).unwrap(), None);
}

#[test]
fn decrement_field_bad_col_returns_error() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 0, 0, 0]).unwrap();
    assert!(eng.decrement_field(1, 99, 1).is_err());
}

// ── mixed operations ──────────────────────────────────────────────────────────

#[test]
fn mixed_increment_decrement_update() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 50, 200, 1]).unwrap();
    eng.increment_field(1, 1, 10).unwrap(); // counter: 60
    eng.decrement_field(1, 2, 50).unwrap(); // score: 150
    eng.update_field(1, 3, 0).unwrap(); // flags: 0
    let row = eng.get(1).unwrap().unwrap();
    assert_eq!(row.values[1], 60);
    assert_eq!(row.values[2], 150);
    assert_eq!(row.values[3], 0);
}

// ── concurrent correctness ────────────────────────────────────────────────────

#[test]
fn concurrent_increments_all_counted() {
    let dir = TempDir::new().unwrap();
    let eng = Arc::new(
        TableEngine::open_with(
            vec![
                "pk".into(),
                "counter".into(),
                "score".into(),
                "flags".into(),
            ],
            dir.path(),
            EngineOptions::new(1024).with_durability(Durability::WriteOnly),
        )
        .unwrap(),
    );
    eng.insert(1, vec![1, 0, 0, 0]).unwrap();

    let n = 8;
    let barrier = Arc::new(Barrier::new(n));
    let mut handles = Vec::new();
    for _ in 0..n {
        let eng = Arc::clone(&eng);
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait();
            eng.increment_field(1, 1, 1).unwrap();
        }));
    }
    for h in handles {
        h.join().unwrap();
    }

    // All 8 increments must be visible — cas_lock serialises them.
    let counter = eng.get(1).unwrap().unwrap().values[1];
    assert_eq!(counter, 8, "all concurrent increments must be counted");
}
