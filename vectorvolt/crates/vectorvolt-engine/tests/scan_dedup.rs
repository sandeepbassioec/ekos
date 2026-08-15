/// Layer 39 — scan+get dedup correctness across all read-path methods.
///
/// Each test exercises the "stale partition" scenario: a row is inserted and
/// flushed (partition A), then updated so its new version sits in the write
/// buffer, which is then flushed to partition B by the read method's
/// flush_if_needed call.  Before this layer, every affected method could
/// double-count or return stale data from partition A.
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

/// Insert pk=1 (cat=1, score=old_score), flush, then update score to new_score.
/// The write buffer is flushed lazily by the read method under test.
fn setup_stale(eng: &TableEngine, old_score: u64, new_score: u64) {
    eng.insert(1, vec![1, 1, old_score, 0]).unwrap();
    eng.flush().unwrap();
    eng.update_field(1, 2, new_score).unwrap();
    // do NOT flush here — the read method must handle it
}

// ── scan_page ─────────────────────────────────────────────────────────────────

#[test]
fn scan_page_returns_current_value_not_stale() {
    let (eng, _dir) = open();
    setup_stale(&eng, 10, 99);

    let page = eng.scan_page(1, 1, 1, 0, 10).unwrap();
    assert_eq!(page.rows.len(), 1, "pk=1 must appear exactly once");
    assert_eq!(page.rows[0].values[2], 99, "score must be current");
}

#[test]
fn scan_page_excludes_row_updated_out_of_filter() {
    let (eng, _dir) = open();
    // pk=1 starts with cat=1 (in filter range), gets updated to cat=2 (outside).
    eng.insert(1, vec![1, 1, 0, 0]).unwrap();
    eng.flush().unwrap();
    eng.update_field(1, 1, 2).unwrap(); // cat: 1 → 2

    // Filter for cat==1: pk=1 no longer qualifies.
    let page = eng.scan_page(1, 1, 1, 0, 10).unwrap();
    assert!(page.rows.is_empty(), "updated-out row must not appear");
}

// ── scan_sorted ───────────────────────────────────────────────────────────────

#[test]
fn scan_sorted_no_duplicate_rows_after_update() {
    let (eng, _dir) = open();
    setup_stale(&eng, 5, 50);
    eng.insert(2, vec![2, 1, 20, 0]).unwrap(); // second row

    let rows = eng.scan_sorted(1, 1, 1, 2, true, 10).unwrap();
    assert_eq!(rows.len(), 2, "must return exactly 2 unique rows");
    let pks: Vec<u64> = rows.iter().map(|r| r.pk).collect();
    assert_eq!(pks.iter().filter(|&&p| p == 1).count(), 1, "pk=1 once only");
}

// ── scan_where ────────────────────────────────────────────────────────────────

#[test]
fn scan_where_no_duplicates_after_update() {
    let (eng, _dir) = open();
    setup_stale(&eng, 10, 99);

    let rows = eng.scan_where(&[(1, 1, 1)]).unwrap();
    assert_eq!(rows.len(), 1, "pk=1 must appear once");
    assert_eq!(rows[0].values[2], 99);
}

#[test]
fn scan_where_excludes_row_updated_out_of_predicate() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 1, 0, 0]).unwrap();
    eng.flush().unwrap();
    eng.update_field(1, 1, 9).unwrap(); // cat: 1 → 9

    let rows = eng.scan_where(&[(1, 1, 1)]).unwrap();
    assert!(rows.is_empty());
}

// ── window functions ──────────────────────────────────────────────────────────

#[test]
fn running_sum_no_double_count_after_update() {
    let (eng, _dir) = open();
    setup_stale(&eng, 10, 5); // score was 10, now 5
    eng.insert(2, vec![2, 1, 3, 0]).unwrap();

    let ws = eng.running_sum(1, 1, 1, 2).unwrap();
    // pk=1 score=5, pk=2 score=3 → sorted by pk: cumsum = [5, 8]
    let vals: Vec<u64> = ws.iter().map(|w| w.window_val).collect();
    assert_eq!(
        vals.last().copied(),
        Some(8),
        "total must be 8, not 13+3=16"
    );
}

#[test]
fn running_count_no_duplicate_pk_after_update() {
    let (eng, _dir) = open();
    setup_stale(&eng, 0, 99);

    let ws = eng.running_count(1, 1, 1).unwrap();
    assert_eq!(ws.len(), 1);
    assert_eq!(ws[0].window_val, 1);
}

// ── group_by ──────────────────────────────────────────────────────────────────

#[test]
fn group_by_count_no_double_count_after_update() {
    let (eng, _dir) = open();
    setup_stale(&eng, 0, 0);
    eng.insert(2, vec![2, 1, 0, 0]).unwrap();

    // Both pks have category=1; each should count once → (cat=1, count=2).
    let counts = eng.group_by_count(1, 1, 1, 1).unwrap();
    assert_eq!(counts, vec![(1, 2)], "each pk counted once");
}

#[test]
fn group_by_sum_uses_current_value() {
    let (eng, _dir) = open();
    setup_stale(&eng, 10, 4); // score: 10 → 4
    eng.insert(2, vec![2, 1, 6, 0]).unwrap();

    let sums = eng.group_by_sum(1, 1, 1, 1, 2).unwrap();
    // pk=1 score=4, pk=2 score=6 → sum=10
    assert_eq!(sums, vec![(1, 10)]);
}

// ── scalar aggregates ─────────────────────────────────────────────────────────

#[test]
fn count_range_no_double_count_after_update() {
    let (eng, _dir) = open();
    setup_stale(&eng, 5, 5); // score stays 5
    eng.insert(2, vec![2, 1, 5, 0]).unwrap();

    let n = eng.count_range(2, 5, 5).unwrap();
    assert_eq!(n, 2, "count must be 2, not 3 from stale partition");
}

#[test]
fn sum_range_uses_current_value_not_stale() {
    let (eng, _dir) = open();
    setup_stale(&eng, 100, 7); // score: 100 → 7
    eng.insert(2, vec![2, 1, 3, 0]).unwrap();

    // Both scores ∈ [0, MAX]. Sum should be 7+3=10, not 100+7+3=110.
    let s = eng.sum_range(2, 0, u64::MAX).unwrap();
    assert_eq!(s, 10);
}

#[test]
fn min_range_uses_current_not_stale() {
    let (eng, _dir) = open();
    setup_stale(&eng, 1, 50); // score: 1 → 50

    // Current score is 50; stale was 1. min should be 50.
    let m = eng.min_range(2, 0, u64::MAX).unwrap();
    assert_eq!(m, Some(50));
}

// ── more scalar aggregates ────────────────────────────────────────────────────

#[test]
fn max_range_uses_current_not_stale() {
    let (eng, _dir) = open();
    setup_stale(&eng, 99, 7); // score: 99 → 7

    // Stale partition still has 99; current is 7. max should be 7.
    let m = eng.max_range(2, 0, u64::MAX).unwrap();
    assert_eq!(m, Some(7));
}

#[test]
fn distinct_values_counts_updated_row_once() {
    let (eng, _dir) = open();
    // pk=1: score 5 → 9 (two partition files for pk=1).
    eng.insert(1, vec![1, 1, 5, 0]).unwrap();
    eng.flush().unwrap();
    eng.update_field(1, 2, 9).unwrap();

    let vals = eng.distinct_values(2, 0, u64::MAX).unwrap();
    assert_eq!(vals, vec![9], "only current value 9, not stale 5");
}

// ── scan_project ──────────────────────────────────────────────────────────────

#[test]
fn scan_project_returns_current_projected_value() {
    let (eng, _dir) = open();
    setup_stale(&eng, 10, 77);

    let rows = eng.scan_project(1, 1, 1, &[2]).unwrap(); // project score only
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].values[0], 77, "projected score must be current");
}
