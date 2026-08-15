/// Layer 27 — sorted scan + top-N integration tests.
///
/// `scan_sorted(filter_col, lo, hi, sort_col, ascending, limit)` filters by a
/// range predicate on one column, sorts matches on another column, and returns
/// at most `limit` full rows.
use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};

/// Schema: pk | score | category | ts
fn open() -> (TableEngine, TempDir) {
    let dir = TempDir::new().unwrap();
    let eng = TableEngine::open_with(
        vec!["pk".into(), "score".into(), "category".into(), "ts".into()],
        dir.path(),
        EngineOptions::new(1024).with_durability(Durability::WriteOnly),
    )
    .unwrap();
    (eng, dir)
}

fn seed(eng: &TableEngine) {
    // pk=1..=10, score=pk*10, category=pk%3, ts=100+pk
    for pk in 1u64..=10 {
        eng.insert(pk, vec![pk, pk * 10, pk % 3, 100 + pk]).unwrap();
    }
}

// ── basic ordering ────────────────────────────────────────────────────────────

#[test]
fn scan_sorted_ascending_returns_sorted_rows() {
    let (eng, _dir) = open();
    seed(&eng);
    // filter col=1 (score) in [10, 100]; sort col=1 (score) ascending
    let rows = eng.scan_sorted(1, 10, 100, 1, true, usize::MAX).unwrap();
    assert_eq!(rows.len(), 10);
    let scores: Vec<u64> = rows.iter().map(|r| r.values[1]).collect();
    let mut sorted = scores.clone();
    sorted.sort_unstable();
    assert_eq!(scores, sorted, "rows must be in ascending score order");
}

#[test]
fn scan_sorted_descending_returns_sorted_rows() {
    let (eng, _dir) = open();
    seed(&eng);
    let rows = eng.scan_sorted(1, 10, 100, 1, false, usize::MAX).unwrap();
    assert_eq!(rows.len(), 10);
    let scores: Vec<u64> = rows.iter().map(|r| r.values[1]).collect();
    let mut sorted = scores.clone();
    sorted.sort_unstable_by(|a, b| b.cmp(a));
    assert_eq!(scores, sorted, "rows must be in descending score order");
}

// ── limit ─────────────────────────────────────────────────────────────────────

#[test]
fn scan_sorted_limit_top3_ascending() {
    let (eng, _dir) = open();
    seed(&eng);
    let rows = eng.scan_sorted(1, 10, 100, 1, true, 3).unwrap();
    assert_eq!(rows.len(), 3);
    // Ascending top-3 scores are 10, 20, 30
    assert_eq!(rows[0].values[1], 10);
    assert_eq!(rows[1].values[1], 20);
    assert_eq!(rows[2].values[1], 30);
}

#[test]
fn scan_sorted_limit_top3_descending() {
    let (eng, _dir) = open();
    seed(&eng);
    let rows = eng.scan_sorted(1, 10, 100, 1, false, 3).unwrap();
    assert_eq!(rows.len(), 3);
    // Descending top-3 scores are 100, 90, 80
    assert_eq!(rows[0].values[1], 100);
    assert_eq!(rows[1].values[1], 90);
    assert_eq!(rows[2].values[1], 80);
}

#[test]
fn scan_sorted_limit_larger_than_matches_returns_all() {
    let (eng, _dir) = open();
    seed(&eng);
    let rows = eng.scan_sorted(1, 10, 100, 1, true, 1000).unwrap();
    assert_eq!(rows.len(), 10);
}

#[test]
fn scan_sorted_limit_zero_returns_empty() {
    let (eng, _dir) = open();
    seed(&eng);
    let rows = eng.scan_sorted(1, 10, 100, 1, true, 0).unwrap();
    assert!(rows.is_empty());
}

// ── range predicate ───────────────────────────────────────────────────────────

#[test]
fn scan_sorted_range_filters_correctly() {
    let (eng, _dir) = open();
    seed(&eng);
    // score in [30, 70] → pks 3..=7, scores 30/40/50/60/70
    let rows = eng.scan_sorted(1, 30, 70, 1, true, usize::MAX).unwrap();
    assert_eq!(rows.len(), 5);
    assert_eq!(rows[0].values[1], 30);
    assert_eq!(rows[4].values[1], 70);
}

#[test]
fn scan_sorted_inverted_range_returns_empty() {
    let (eng, _dir) = open();
    seed(&eng);
    let rows = eng.scan_sorted(1, 100, 10, 1, true, usize::MAX).unwrap();
    assert!(rows.is_empty());
}

#[test]
fn scan_sorted_empty_table_returns_empty() {
    let (eng, _dir) = open();
    let rows = eng.scan_sorted(1, 0, 1000, 1, true, usize::MAX).unwrap();
    assert!(rows.is_empty());
}

// ── filter_col != sort_col ────────────────────────────────────────────────────

#[test]
fn scan_sorted_filter_on_ts_sort_by_score() {
    let (eng, _dir) = open();
    seed(&eng);
    // filter col=3 (ts) in [103, 107] → pks 3..=7
    // sort  col=1 (score) descending → 70, 60, 50, 40, 30
    let rows = eng.scan_sorted(3, 103, 107, 1, false, usize::MAX).unwrap();
    assert_eq!(rows.len(), 5);
    assert_eq!(rows[0].values[1], 70);
    assert_eq!(rows[4].values[1], 30);
}

#[test]
fn scan_sorted_filter_on_score_sort_by_category() {
    let (eng, _dir) = open();
    seed(&eng);
    // score in [10, 60] → pks 1..=6
    // sort by category ascending
    let rows = eng.scan_sorted(1, 10, 60, 2, true, usize::MAX).unwrap();
    assert_eq!(rows.len(), 6);
    let cats: Vec<u64> = rows.iter().map(|r| r.values[2]).collect();
    let mut sorted = cats.clone();
    sorted.sort_unstable();
    assert_eq!(cats, sorted);
}

#[test]
fn scan_sorted_sort_col_same_as_filter_col() {
    let (eng, _dir) = open();
    seed(&eng);
    // filter col=1 = sort col=1 — should work identically to basic case
    let rows = eng.scan_sorted(1, 10, 50, 1, true, usize::MAX).unwrap();
    assert_eq!(rows.len(), 5);
    assert_eq!(rows[0].values[1], 10);
    assert_eq!(rows[4].values[1], 50);
}

// ── error cases ───────────────────────────────────────────────────────────────

#[test]
fn scan_sorted_bad_filter_col_returns_error() {
    let (eng, _dir) = open();
    let result = eng.scan_sorted(99, 0, 100, 1, true, usize::MAX);
    assert!(result.is_err(), "out-of-range filter_col must return Err");
}

#[test]
fn scan_sorted_bad_sort_col_returns_error() {
    let (eng, _dir) = open();
    let result = eng.scan_sorted(1, 0, 100, 99, true, usize::MAX);
    assert!(result.is_err(), "out-of-range sort_col must return Err");
}

// ── multi-partition ───────────────────────────────────────────────────────────

#[test]
fn scan_sorted_multi_partition_ordering_correct() {
    let dir = TempDir::new().unwrap();
    // Small partition size forces multiple partitions
    let eng = TableEngine::open_with(
        vec!["pk".into(), "score".into(), "tag".into()],
        dir.path(),
        EngineOptions::new(4).with_durability(Durability::WriteOnly),
    )
    .unwrap();

    // Insert 20 rows with scores 20..=1 (reverse order)
    for pk in 1u64..=20 {
        eng.insert(pk, vec![pk, 21 - pk, pk % 4]).unwrap();
    }

    // scan_sorted ascending on score — must be 1, 2, ..., 20 regardless of partition order
    let rows = eng.scan_sorted(1, 1, 20, 1, true, usize::MAX).unwrap();
    assert_eq!(rows.len(), 20);
    let scores: Vec<u64> = rows.iter().map(|r| r.values[1]).collect();
    let expected: Vec<u64> = (1..=20).collect();
    assert_eq!(scores, expected);
}

#[test]
fn scan_sorted_multi_partition_top5_descending() {
    let dir = TempDir::new().unwrap();
    let eng = TableEngine::open_with(
        vec!["pk".into(), "score".into(), "tag".into()],
        dir.path(),
        EngineOptions::new(4).with_durability(Durability::WriteOnly),
    )
    .unwrap();

    for pk in 1u64..=20 {
        eng.insert(pk, vec![pk, pk * 3, pk % 5]).unwrap();
    }

    let rows = eng.scan_sorted(1, 3, 60, 1, false, 5).unwrap();
    assert_eq!(rows.len(), 5);
    // scores descending: 60, 57, 54, 51, 48
    assert_eq!(rows[0].values[1], 60);
    assert_eq!(rows[4].values[1], 48);
}
