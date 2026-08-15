/// Layer 31 — secondary index integration tests.
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

/// pk=1..=12, score=pk*10, category=pk%3 (0/1/2), region=pk%4 (0/1/2/3)
fn seed(eng: &TableEngine) {
    for pk in 1u64..=12 {
        eng.insert(pk, vec![pk, pk * 10, pk % 3, pk % 4]).unwrap();
    }
}

// ── build / has / drop ────────────────────────────────────────────────────────

#[test]
fn build_secondary_index_creates_index() {
    let (eng, _dir) = open();
    seed(&eng);
    assert!(!eng.has_secondary_index(2));
    eng.build_secondary_index(2).unwrap();
    assert!(eng.has_secondary_index(2));
}

#[test]
fn drop_secondary_index_removes_it() {
    let (eng, _dir) = open();
    seed(&eng);
    eng.build_secondary_index(2).unwrap();
    eng.drop_secondary_index(2);
    assert!(!eng.has_secondary_index(2));
}

#[test]
fn drop_nonexistent_index_is_noop() {
    let (eng, _dir) = open();
    eng.drop_secondary_index(1); // no panic
}

#[test]
fn build_secondary_index_bad_col_returns_error() {
    let (eng, _dir) = open();
    assert!(eng.build_secondary_index(99).is_err());
}

#[test]
fn entry_count_reflects_distinct_values() {
    let (eng, _dir) = open();
    seed(&eng);
    eng.build_secondary_index(2).unwrap(); // category has 3 distinct values: 0,1,2
    assert_eq!(eng.secondary_index_entry_count(2), 3);
}

#[test]
fn entry_count_zero_when_no_index() {
    let (eng, _dir) = open();
    assert_eq!(eng.secondary_index_entry_count(2), 0);
}

// ── secondary_lookup (exact) ──────────────────────────────────────────────────

#[test]
fn secondary_lookup_returns_correct_pks() {
    let (eng, _dir) = open();
    seed(&eng);
    eng.build_secondary_index(2).unwrap();
    // category=0: pks 3,6,9,12
    let mut pks = eng.secondary_lookup(2, 0).unwrap();
    pks.sort_unstable();
    assert_eq!(pks, vec![3, 6, 9, 12]);
}

#[test]
fn secondary_lookup_all_categories_correct() {
    let (eng, _dir) = open();
    seed(&eng);
    eng.build_secondary_index(2).unwrap();
    // category=1: pks 1,4,7,10
    let mut pks1 = eng.secondary_lookup(2, 1).unwrap();
    pks1.sort_unstable();
    assert_eq!(pks1, vec![1, 4, 7, 10]);
    // category=2: pks 2,5,8,11
    let mut pks2 = eng.secondary_lookup(2, 2).unwrap();
    pks2.sort_unstable();
    assert_eq!(pks2, vec![2, 5, 8, 11]);
}

#[test]
fn secondary_lookup_no_match_returns_empty() {
    let (eng, _dir) = open();
    seed(&eng);
    eng.build_secondary_index(2).unwrap();
    assert!(eng.secondary_lookup(2, 99).unwrap().is_empty());
}

#[test]
fn secondary_lookup_without_index_returns_empty() {
    let (eng, _dir) = open();
    seed(&eng);
    // No index built — returns empty, not error
    assert!(eng.secondary_lookup(2, 0).unwrap().is_empty());
}

#[test]
fn secondary_lookup_bad_col_returns_error() {
    let (eng, _dir) = open();
    assert!(eng.secondary_lookup(99, 0).is_err());
}

// ── secondary_range_lookup ────────────────────────────────────────────────────

#[test]
fn secondary_range_lookup_returns_pks_in_range() {
    let (eng, _dir) = open();
    seed(&eng);
    eng.build_secondary_index(1).unwrap(); // index on score
    // score in [30, 70] → pks 3,4,5,6,7
    let mut pks = eng.secondary_range_lookup(1, 30, 70).unwrap();
    pks.sort_unstable();
    assert_eq!(pks, vec![3, 4, 5, 6, 7]);
}

#[test]
fn secondary_range_lookup_single_value_range() {
    let (eng, _dir) = open();
    seed(&eng);
    eng.build_secondary_index(1).unwrap();
    // score in [50, 50] → only pk=5
    let pks = eng.secondary_range_lookup(1, 50, 50).unwrap();
    assert_eq!(pks, vec![5]);
}

#[test]
fn secondary_range_lookup_full_range_returns_all() {
    let (eng, _dir) = open();
    seed(&eng);
    eng.build_secondary_index(1).unwrap();
    let mut pks = eng.secondary_range_lookup(1, 10, 120).unwrap();
    pks.sort_unstable();
    assert_eq!(pks.len(), 12);
}

#[test]
fn secondary_range_lookup_inverted_range_returns_empty() {
    let (eng, _dir) = open();
    seed(&eng);
    eng.build_secondary_index(1).unwrap();
    assert!(eng.secondary_range_lookup(1, 100, 10).unwrap().is_empty());
}

#[test]
fn secondary_range_lookup_without_index_returns_empty() {
    let (eng, _dir) = open();
    seed(&eng);
    assert!(eng.secondary_range_lookup(1, 10, 100).unwrap().is_empty());
}

// ── secondary_get ─────────────────────────────────────────────────────────────

#[test]
fn secondary_get_materialises_full_rows() {
    let (eng, _dir) = open();
    seed(&eng);
    eng.build_secondary_index(2).unwrap();
    let rows = eng.secondary_get(2, 0).unwrap();
    assert_eq!(rows.len(), 4); // pks 3,6,9,12
    for row in &rows {
        assert_eq!(row.values[2], 0, "category must be 0");
    }
}

#[test]
fn secondary_get_skips_deleted_rows() {
    let (eng, _dir) = open();
    seed(&eng);
    eng.build_secondary_index(2).unwrap();
    // Delete pk=3 (category=0) — index still lists it, secondary_get skips it
    eng.delete(3).unwrap();
    let rows = eng.secondary_get(2, 0).unwrap();
    assert_eq!(rows.len(), 3); // 6, 9, 12 remain
    assert!(rows.iter().all(|r| r.pk != 3));
}

// ── rebuild replaces previous index ──────────────────────────────────────────

#[test]
fn rebuild_index_reflects_new_data() {
    let (eng, _dir) = open();
    // Build index with 3 rows
    for pk in 1u64..=3 {
        eng.insert(pk, vec![pk, pk * 10, pk % 2, 0]).unwrap();
    }
    eng.build_secondary_index(2).unwrap();
    assert_eq!(eng.secondary_index_entry_count(2), 2); // values 0 and 1

    // Add more rows and rebuild
    for pk in 4u64..=6 {
        eng.insert(pk, vec![pk, pk * 10, 2, 0]).unwrap(); // new category value=2
    }
    eng.build_secondary_index(2).unwrap();
    assert_eq!(eng.secondary_index_entry_count(2), 3); // now 0, 1, and 2
    let mut pks = eng.secondary_lookup(2, 2).unwrap();
    pks.sort_unstable();
    assert_eq!(pks, vec![4, 5, 6]);
}

// ── multiple indexes on different columns ─────────────────────────────────────

#[test]
fn multiple_indexes_on_different_columns_coexist() {
    let (eng, _dir) = open();
    seed(&eng);
    eng.build_secondary_index(2).unwrap(); // category
    eng.build_secondary_index(3).unwrap(); // region
    assert!(eng.has_secondary_index(2));
    assert!(eng.has_secondary_index(3));
    eng.drop_secondary_index(2);
    assert!(!eng.has_secondary_index(2));
    assert!(eng.has_secondary_index(3)); // unaffected
}

// ── multi-partition ───────────────────────────────────────────────────────────

#[test]
fn secondary_index_multi_partition_complete() {
    let dir = TempDir::new().unwrap();
    let eng = TableEngine::open_with(
        vec!["pk".into(), "score".into(), "cat".into(), "reg".into()],
        dir.path(),
        EngineOptions::new(4).with_durability(Durability::WriteOnly),
    )
    .unwrap();
    // 20 rows across 5 partitions; cat=pk%2
    for pk in 1u64..=20 {
        eng.insert(pk, vec![pk, pk, pk % 2, 0]).unwrap();
    }
    eng.build_secondary_index(2).unwrap();
    // cat=0: even pks 2,4,...,20 → 10 PKs
    let mut pks0 = eng.secondary_lookup(2, 0).unwrap();
    pks0.sort_unstable();
    assert_eq!(pks0.len(), 10);
    assert!(pks0.iter().all(|pk| pk % 2 == 0));

    // cat=1: odd pks 1,3,...,19 → 10 PKs
    let mut pks1 = eng.secondary_lookup(2, 1).unwrap();
    pks1.sort_unstable();
    assert_eq!(pks1.len(), 10);
    assert!(pks1.iter().all(|pk| pk % 2 == 1));
}
