/// Layer 30 — range delete integration tests.
///
/// Schema: pk | score | category | region
use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};

fn open() -> (TableEngine, TempDir) {
    let dir = TempDir::new().unwrap();
    let eng = TableEngine::open_with(
        vec![
            "pk".into(),
            "score".into(),
            "category".into(),
            "region".into(),
        ],
        dir.path(),
        EngineOptions::new(1024).with_durability(Durability::WriteOnly),
    )
    .unwrap();
    (eng, dir)
}

/// pk=1..=n, score=pk*10, category=pk%3, region=pk%4
fn seed(eng: &TableEngine, n: u64) {
    for pk in 1u64..=n {
        eng.insert(pk, vec![pk, pk * 10, pk % 3, pk % 4]).unwrap();
    }
}

// ── basic correctness ─────────────────────────────────────────────────────────

#[test]
fn delete_range_removes_matched_rows() {
    let (eng, _dir) = open();
    seed(&eng, 10);
    // score in [10, 50] → pks 1..=5
    let deleted = eng.delete_range(1, 10, 50).unwrap();
    assert_eq!(deleted, 5);
    for pk in 1u64..=5 {
        assert!(eng.get(pk).unwrap().is_none(), "pk={pk} should be deleted");
    }
}

#[test]
fn delete_range_leaves_unmatched_rows_intact() {
    let (eng, _dir) = open();
    seed(&eng, 10);
    // Delete score [10, 50] → pks 1..=5; pks 6..=10 must survive
    eng.delete_range(1, 10, 50).unwrap();
    for pk in 6u64..=10 {
        assert!(eng.get(pk).unwrap().is_some(), "pk={pk} should survive");
    }
}

#[test]
fn delete_range_returns_correct_count() {
    let (eng, _dir) = open();
    seed(&eng, 10);
    let deleted = eng.delete_range(1, 30, 70).unwrap(); // pks 3..=7 → 5 rows
    assert_eq!(deleted, 5);
}

#[test]
fn delete_range_exact_single_row() {
    let (eng, _dir) = open();
    seed(&eng, 5);
    let deleted = eng.delete_range(1, 30, 30).unwrap(); // only pk=3, score=30
    assert_eq!(deleted, 1);
    assert!(eng.get(3).unwrap().is_none());
    assert!(eng.get(2).unwrap().is_some());
    assert!(eng.get(4).unwrap().is_some());
}

#[test]
fn delete_range_all_rows() {
    let (eng, _dir) = open();
    seed(&eng, 5);
    let deleted = eng.delete_range(1, 10, 50).unwrap();
    assert_eq!(deleted, 5);
    for pk in 1u64..=5 {
        assert!(eng.get(pk).unwrap().is_none());
    }
}

// ── edge cases ────────────────────────────────────────────────────────────────

#[test]
fn delete_range_inverted_range_returns_zero() {
    let (eng, _dir) = open();
    seed(&eng, 5);
    let deleted = eng.delete_range(1, 50, 10).unwrap();
    assert_eq!(deleted, 0);
    // All rows intact
    for pk in 1u64..=5 {
        assert!(eng.get(pk).unwrap().is_some());
    }
}

#[test]
fn delete_range_no_matches_returns_zero() {
    let (eng, _dir) = open();
    seed(&eng, 5);
    // score range [999, 9999] — no rows
    let deleted = eng.delete_range(1, 999, 9999).unwrap();
    assert_eq!(deleted, 0);
}

#[test]
fn delete_range_empty_table_returns_zero() {
    let (eng, _dir) = open();
    let deleted = eng.delete_range(1, 0, 1000).unwrap();
    assert_eq!(deleted, 0);
}

#[test]
fn delete_range_bad_col_returns_error() {
    let (eng, _dir) = open();
    assert!(eng.delete_range(99, 0, 100).is_err());
}

// ── filter on different columns ───────────────────────────────────────────────

#[test]
fn delete_range_filter_on_category_col() {
    let (eng, _dir) = open();
    seed(&eng, 12);
    // category=0 for pks 3,6,9,12; delete_range on col=2 (category) in [0,0]
    let deleted = eng.delete_range(2, 0, 0).unwrap();
    assert_eq!(deleted, 4);
    for pk in [3u64, 6, 9, 12] {
        assert!(eng.get(pk).unwrap().is_none());
    }
    // Other rows intact
    for pk in [1u64, 2, 4, 5, 7, 8, 10, 11] {
        assert!(eng.get(pk).unwrap().is_some());
    }
}

#[test]
fn delete_range_filter_on_region_col() {
    let (eng, _dir) = open();
    seed(&eng, 8);
    // region=pk%4; region=1 for pks 1,5; region=2 for pks 2,6
    // delete region in [1,2] → pks 1,2,5,6
    let deleted = eng.delete_range(3, 1, 2).unwrap();
    assert_eq!(deleted, 4);
    for pk in [1u64, 2, 5, 6] {
        assert!(eng.get(pk).unwrap().is_none());
    }
    for pk in [3u64, 4, 7, 8] {
        assert!(eng.get(pk).unwrap().is_some());
    }
}

// ── idempotency ───────────────────────────────────────────────────────────────

#[test]
fn delete_range_twice_second_call_returns_zero() {
    let (eng, _dir) = open();
    seed(&eng, 5);
    assert_eq!(eng.delete_range(1, 10, 50).unwrap(), 5);
    // Second call: rows already gone, PkNotFound silently skipped
    assert_eq!(eng.delete_range(1, 10, 50).unwrap(), 0);
}

// ── multi-partition ───────────────────────────────────────────────────────────

#[test]
fn delete_range_multi_partition_correct() {
    let dir = TempDir::new().unwrap();
    let eng = TableEngine::open_with(
        vec!["pk".into(), "score".into(), "cat".into(), "reg".into()],
        dir.path(),
        EngineOptions::new(4).with_durability(Durability::WriteOnly),
    )
    .unwrap();
    // 20 rows across multiple partitions
    for pk in 1u64..=20 {
        eng.insert(pk, vec![pk, pk, pk % 2, 0]).unwrap();
    }
    // Delete score in [5, 15] → pks 5..=15 (11 rows)
    let deleted = eng.delete_range(1, 5, 15).unwrap();
    assert_eq!(deleted, 11);
    for pk in 5u64..=15 {
        assert!(eng.get(pk).unwrap().is_none());
    }
    for pk in (1u64..=4).chain(16..=20) {
        assert!(eng.get(pk).unwrap().is_some());
    }
}

#[test]
fn delete_range_multi_partition_zone_prunes_correctly() {
    let dir = TempDir::new().unwrap();
    let eng = TableEngine::open_with(
        vec!["pk".into(), "score".into(), "cat".into(), "reg".into()],
        dir.path(),
        EngineOptions::new(4).with_durability(Durability::WriteOnly),
    )
    .unwrap();
    for pk in 1u64..=20 {
        eng.insert(pk, vec![pk, pk * 10, 0, 0]).unwrap();
    }
    // score in [150, 200] → pks 15..=20 → 6 rows
    // Partitions holding pks 1..=14 (scores 10..=140) are zone-pruned
    let deleted = eng.delete_range(1, 150, 200).unwrap();
    assert_eq!(deleted, 6);
    for pk in 1u64..=14 {
        assert!(eng.get(pk).unwrap().is_some());
    }
    for pk in 15u64..=20 {
        assert!(eng.get(pk).unwrap().is_none());
    }
}
