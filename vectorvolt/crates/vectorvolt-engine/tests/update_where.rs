/// Layer 36 — bulk predicate-filtered field update integration tests.
///
/// Schema: pk | category | score | flags
use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};

fn open() -> (TableEngine, TempDir) {
    let dir = TempDir::new().unwrap();
    let eng = TableEngine::open_with(
        vec![
            "pk".into(),
            "category".into(),
            "score".into(),
            "flags".into(),
        ],
        dir.path(),
        EngineOptions::new(1024).with_durability(Durability::WriteOnly),
    )
    .unwrap();
    (eng, dir)
}

fn score(eng: &TableEngine, pk: u64) -> u64 {
    eng.get(pk).unwrap().unwrap().values[2]
}

fn flags(eng: &TableEngine, pk: u64) -> u64 {
    eng.get(pk).unwrap().unwrap().values[3]
}

// ── update_field_where ────────────────────────────────────────────────────────

#[test]
fn update_field_where_updates_matching_rows() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 1, 10, 0]).unwrap();
    eng.insert(2, vec![2, 2, 20, 0]).unwrap();
    eng.insert(3, vec![3, 1, 30, 0]).unwrap();
    eng.flush().unwrap();

    // Set score=99 for all rows where category==1.
    eng.update_field_where(&[(1, 1, 1)], 2, 99).unwrap();

    assert_eq!(score(&eng, 1), 99);
    assert_eq!(score(&eng, 2), 20); // unchanged
    assert_eq!(score(&eng, 3), 99);
}

#[test]
fn update_field_where_returns_count_of_updated_rows() {
    let (eng, _dir) = open();
    for i in 1u64..=5 {
        eng.insert(i, vec![i, i % 2, 0, 0]).unwrap(); // odd pks: cat=1; even pks: cat=0
    }
    eng.flush().unwrap();

    // category==1: pks 1, 3, 5 → 3 rows
    let n = eng.update_field_where(&[(1, 1, 1)], 2, 7).unwrap();
    assert_eq!(n, 3);
}

#[test]
fn update_field_where_no_match_returns_zero() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 5, 100, 0]).unwrap();
    eng.flush().unwrap();

    let n = eng.update_field_where(&[(1, 99, 99)], 2, 1).unwrap();
    assert_eq!(n, 0);
    assert_eq!(score(&eng, 1), 100); // unchanged
}

#[test]
fn update_field_where_empty_predicates_updates_all() {
    let (eng, _dir) = open();
    for i in 1u64..=4 {
        eng.insert(i, vec![i, i, 0, 0]).unwrap();
    }
    eng.flush().unwrap();

    let n = eng.update_field_where(&[], 2, 55).unwrap();
    assert_eq!(n, 4);
    for i in 1u64..=4 {
        assert_eq!(score(&eng, i), 55);
    }
}

#[test]
fn update_field_where_leaves_non_matching_rows_intact() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 1, 10, 7]).unwrap();
    eng.insert(2, vec![2, 2, 20, 7]).unwrap();
    eng.flush().unwrap();

    eng.update_field_where(&[(1, 1, 1)], 2, 99).unwrap();

    // pk=2 score and flags untouched
    assert_eq!(score(&eng, 2), 20);
    assert_eq!(flags(&eng, 2), 7);
    // pk=1 flags untouched (only score was the target col)
    assert_eq!(flags(&eng, 1), 7);
}

#[test]
fn update_field_where_bad_target_col_returns_error() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 1, 0, 0]).unwrap();
    eng.flush().unwrap();

    assert!(eng.update_field_where(&[(1, 1, 1)], 99, 1).is_err());
}

#[test]
fn update_field_where_bad_predicate_col_returns_error() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 1, 0, 0]).unwrap();
    eng.flush().unwrap();

    assert!(eng.update_field_where(&[(99, 1, 1)], 2, 1).is_err());
}

#[test]
fn update_field_where_multi_predicate() {
    let (eng, _dir) = open();
    //                      pk  cat  score  flags
    eng.insert(1, vec![1, 1, 50, 0]).unwrap(); // cat=1, score=50 ✓
    eng.insert(2, vec![2, 1, 200, 0]).unwrap(); // cat=1, score=200 ✗
    eng.insert(3, vec![3, 2, 50, 0]).unwrap(); // cat=2, score=50 ✗
    eng.flush().unwrap();

    // cat==1 AND score in [10, 100]
    let n = eng
        .update_field_where(&[(1, 1, 1), (2, 10, 100)], 3, 1)
        .unwrap();
    assert_eq!(n, 1);
    assert_eq!(flags(&eng, 1), 1); // updated
    assert_eq!(flags(&eng, 2), 0); // score too high
    assert_eq!(flags(&eng, 3), 0); // wrong category
}

// ── increment_field_where ─────────────────────────────────────────────────────

#[test]
fn increment_field_where_increments_matching_rows() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 1, 10, 0]).unwrap();
    eng.insert(2, vec![2, 2, 10, 0]).unwrap();
    eng.insert(3, vec![3, 1, 10, 0]).unwrap();
    eng.flush().unwrap();

    let n = eng.increment_field_where(&[(1, 1, 1)], 2, 5).unwrap();
    assert_eq!(n, 2);
    assert_eq!(score(&eng, 1), 15);
    assert_eq!(score(&eng, 2), 10); // unchanged
    assert_eq!(score(&eng, 3), 15);
}

#[test]
fn increment_field_where_applied_twice_accumulates() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 1, 0, 0]).unwrap();
    eng.flush().unwrap();

    eng.increment_field_where(&[(1, 1, 1)], 2, 3).unwrap();
    eng.increment_field_where(&[(1, 1, 1)], 2, 3).unwrap();

    assert_eq!(score(&eng, 1), 6);
}

#[test]
fn increment_field_where_count_correct() {
    let (eng, _dir) = open();
    for i in 1u64..=6 {
        eng.insert(i, vec![i, 1, 0, 0]).unwrap();
    }
    eng.flush().unwrap();

    let n = eng.increment_field_where(&[], 2, 1).unwrap();
    assert_eq!(n, 6);
}

// ── decrement_field_where ─────────────────────────────────────────────────────

#[test]
fn decrement_field_where_decrements_matching_rows() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 1, 100, 0]).unwrap();
    eng.insert(2, vec![2, 2, 100, 0]).unwrap();
    eng.flush().unwrap();

    let n = eng.decrement_field_where(&[(1, 1, 1)], 2, 40).unwrap();
    assert_eq!(n, 1);
    assert_eq!(score(&eng, 1), 60);
    assert_eq!(score(&eng, 2), 100); // unchanged
}

#[test]
fn decrement_field_where_saturates_at_zero() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 1, 5, 0]).unwrap();
    eng.flush().unwrap();

    eng.decrement_field_where(&[(1, 1, 1)], 2, 100).unwrap();
    assert_eq!(score(&eng, 1), 0); // saturated
}

#[test]
fn decrement_field_where_count_correct() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 1, 50, 0]).unwrap();
    eng.insert(2, vec![2, 1, 50, 0]).unwrap();
    eng.insert(3, vec![3, 9, 50, 0]).unwrap(); // excluded
    eng.flush().unwrap();

    let n = eng.decrement_field_where(&[(1, 1, 1)], 2, 10).unwrap();
    assert_eq!(n, 2);
}

// ── Secondary index stays consistent ─────────────────────────────────────────

#[test]
fn secondary_index_consistent_after_update_field_where() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 1, 10, 0]).unwrap();
    eng.insert(2, vec![2, 1, 10, 0]).unwrap();
    eng.insert(3, vec![3, 2, 10, 0]).unwrap();
    eng.flush().unwrap();
    eng.build_secondary_index(2).unwrap(); // index on score

    // Bulk-update score from 10 to 99 for category==1
    eng.update_field_where(&[(1, 1, 1)], 2, 99).unwrap();

    // Old value bucket (10) should only contain pk=3 now.
    let at_10 = eng.secondary_lookup(2, 10).unwrap();
    assert!(!at_10.contains(&1));
    assert!(!at_10.contains(&2));
    assert!(at_10.contains(&3));

    // New value bucket (99) should contain pk=1 and pk=2.
    let at_99 = eng.secondary_lookup(2, 99).unwrap();
    assert!(at_99.contains(&1));
    assert!(at_99.contains(&2));
}
