/// Layer 38 — distinct values and value counts integration tests.
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

// ── distinct_values ───────────────────────────────────────────────────────────

#[test]
fn distinct_values_basic() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 3, 0, 0]).unwrap();
    eng.insert(2, vec![2, 1, 0, 0]).unwrap();
    eng.insert(3, vec![3, 3, 0, 0]).unwrap();
    eng.insert(4, vec![4, 2, 0, 0]).unwrap();
    eng.flush().unwrap();

    let vals = eng.distinct_values(1, 0, u64::MAX).unwrap();
    assert_eq!(vals, vec![1, 2, 3]);
}

#[test]
fn distinct_values_range_filters_out_of_bounds() {
    let (eng, _dir) = open();
    for i in 1u64..=5 {
        eng.insert(i, vec![i, i * 10, 0, 0]).unwrap(); // categories: 10,20,30,40,50
    }
    eng.flush().unwrap();

    // Only values 20..=40 (pks 2,3,4)
    let vals = eng.distinct_values(1, 20, 40).unwrap();
    assert_eq!(vals, vec![20, 30, 40]);
}

#[test]
fn distinct_values_all_same() {
    let (eng, _dir) = open();
    for i in 1u64..=4 {
        eng.insert(i, vec![i, 7, 0, 0]).unwrap();
    }
    eng.flush().unwrap();

    let vals = eng.distinct_values(1, 0, u64::MAX).unwrap();
    assert_eq!(vals, vec![7]);
}

#[test]
fn distinct_values_lo_greater_than_hi_returns_empty() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 5, 0, 0]).unwrap();
    eng.flush().unwrap();

    let vals = eng.distinct_values(1, 10, 5).unwrap();
    assert!(vals.is_empty());
}

#[test]
fn distinct_values_empty_table_returns_empty() {
    let (eng, _dir) = open();
    let vals = eng.distinct_values(1, 0, u64::MAX).unwrap();
    assert!(vals.is_empty());
}

#[test]
fn distinct_values_bad_col_returns_error() {
    let (eng, _dir) = open();
    assert!(eng.distinct_values(99, 0, u64::MAX).is_err());
}

#[test]
fn distinct_values_result_is_sorted() {
    let (eng, _dir) = open();
    // Insert in reverse order of category value.
    for cat in [50u64, 10, 30, 20, 40].iter() {
        eng.insert(*cat, vec![*cat, *cat, 0, 0]).unwrap();
    }
    eng.flush().unwrap();

    let vals = eng.distinct_values(1, 0, u64::MAX).unwrap();
    assert_eq!(vals, vec![10, 20, 30, 40, 50]);
}

#[test]
fn distinct_values_updated_row_counted_once() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 5, 0, 0]).unwrap();
    eng.flush().unwrap();

    // Update category — old partition has 5, new write has 9.
    eng.update_field(1, 1, 9).unwrap();

    // pk=1 should contribute exactly one value (9, the live one).
    let vals = eng.distinct_values(1, 0, u64::MAX).unwrap();
    assert_eq!(vals, vec![9], "stale partition value must not appear");
}

// ── value_counts ──────────────────────────────────────────────────────────────

#[test]
fn value_counts_basic() {
    let (eng, _dir) = open();
    // Three rows with cat=1, two with cat=2.
    for pk in [1u64, 2, 3] {
        eng.insert(pk, vec![pk, 1, 0, 0]).unwrap();
    }
    for pk in [4u64, 5] {
        eng.insert(pk, vec![pk, 2, 0, 0]).unwrap();
    }
    eng.flush().unwrap();

    let counts = eng.value_counts(1, 0, u64::MAX).unwrap();
    assert_eq!(counts, vec![(1, 3), (2, 2)]);
}

#[test]
fn value_counts_range_excludes_outliers() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 5, 0, 0]).unwrap();
    eng.insert(2, vec![2, 10, 0, 0]).unwrap();
    eng.insert(3, vec![3, 15, 0, 0]).unwrap();
    eng.insert(4, vec![4, 20, 0, 0]).unwrap();
    eng.flush().unwrap();

    // Only values in [10, 15]
    let counts = eng.value_counts(1, 10, 15).unwrap();
    assert_eq!(counts, vec![(10, 1), (15, 1)]);
}

#[test]
fn value_counts_lo_greater_than_hi_returns_empty() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 5, 0, 0]).unwrap();
    eng.flush().unwrap();

    assert!(eng.value_counts(1, 10, 5).unwrap().is_empty());
}

#[test]
fn value_counts_empty_table_returns_empty() {
    let (eng, _dir) = open();
    assert!(eng.value_counts(1, 0, u64::MAX).unwrap().is_empty());
}

#[test]
fn value_counts_bad_col_returns_error() {
    let (eng, _dir) = open();
    assert!(eng.value_counts(99, 0, u64::MAX).is_err());
}

#[test]
fn value_counts_result_sorted_by_value() {
    let (eng, _dir) = open();
    for (pk, cat) in [(1u64, 30u64), (2, 10), (3, 20)] {
        eng.insert(pk, vec![pk, cat, 0, 0]).unwrap();
    }
    eng.flush().unwrap();

    let counts = eng.value_counts(1, 0, u64::MAX).unwrap();
    assert_eq!(counts, vec![(10, 1), (20, 1), (30, 1)]);
}

#[test]
fn value_counts_updated_row_counted_once() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 5, 0, 0]).unwrap();
    eng.insert(2, vec![2, 5, 0, 0]).unwrap();
    eng.flush().unwrap();

    // Update pk=1 category from 5 to 9 — old partition still has cat=5.
    eng.update_field(1, 1, 9).unwrap();

    let counts = eng.value_counts(1, 0, u64::MAX).unwrap();
    // cat=5 → only pk=2 (pk=1 is now cat=9); cat=9 → pk=1
    assert_eq!(counts, vec![(5, 1), (9, 1)]);
}
