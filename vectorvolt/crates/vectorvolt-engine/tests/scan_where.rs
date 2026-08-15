/// Layer 34 — multi-predicate compound scan integration tests.
///
/// Schema: pk | col_a | col_b | col_c
use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};

fn open() -> (TableEngine, TempDir) {
    let dir = TempDir::new().unwrap();
    let eng = TableEngine::open_with(
        vec!["pk".into(), "col_a".into(), "col_b".into(), "col_c".into()],
        dir.path(),
        EngineOptions::new(1024).with_durability(Durability::WriteOnly),
    )
    .unwrap();
    (eng, dir)
}

fn pks(rows: &[vectorvolt_engine::EngineRow]) -> Vec<u64> {
    let mut v: Vec<u64> = rows.iter().map(|r| r.pk).collect();
    v.sort_unstable();
    v
}

// ── Empty predicates = full scan ──────────────────────────────────────────────

#[test]
fn empty_predicates_returns_all_rows() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 10, 20, 30]).unwrap();
    eng.insert(2, vec![2, 11, 21, 31]).unwrap();
    eng.insert(3, vec![3, 12, 22, 32]).unwrap();
    eng.flush().unwrap();

    let rows = eng.scan_where(&[]).unwrap();
    assert_eq!(pks(&rows), vec![1, 2, 3]);
}

// ── Single predicate ──────────────────────────────────────────────────────────

#[test]
fn single_predicate_exact_match() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 5, 0, 0]).unwrap();
    eng.insert(2, vec![2, 9, 0, 0]).unwrap();
    eng.insert(3, vec![3, 5, 0, 0]).unwrap();
    eng.flush().unwrap();

    let rows = eng.scan_where(&[(1, 5, 5)]).unwrap();
    assert_eq!(pks(&rows), vec![1, 3]);
}

#[test]
fn single_predicate_range() {
    let (eng, _dir) = open();
    for i in 1u64..=10 {
        eng.insert(i, vec![i, i * 10, 0, 0]).unwrap();
    }
    eng.flush().unwrap();

    let rows = eng.scan_where(&[(1, 30, 60)]).unwrap();
    // col_a values 30,40,50,60 → pks 3,4,5,6
    assert_eq!(pks(&rows), vec![3, 4, 5, 6]);
}

// ── Two-predicate AND ─────────────────────────────────────────────────────────

#[test]
fn two_predicates_intersection() {
    let (eng, _dir) = open();
    //                      pk  col_a  col_b  col_c
    eng.insert(1, vec![1, 10, 100, 0]).unwrap(); // a in range, b in range  ✓
    eng.insert(2, vec![2, 10, 200, 0]).unwrap(); // a in range, b NOT       ✗
    eng.insert(3, vec![3, 20, 100, 0]).unwrap(); // a NOT,      b in range  ✗
    eng.insert(4, vec![4, 20, 200, 0]).unwrap(); // both NOT                ✗
    eng.flush().unwrap();

    let rows = eng.scan_where(&[(1, 10, 15), (2, 50, 150)]).unwrap();
    assert_eq!(pks(&rows), vec![1]);
}

#[test]
fn two_predicates_first_matches_none() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 5, 100, 0]).unwrap();
    eng.insert(2, vec![2, 6, 100, 0]).unwrap();
    eng.flush().unwrap();

    // col_a predicate [99,99] matches nothing.
    let rows = eng.scan_where(&[(1, 99, 99), (2, 50, 150)]).unwrap();
    assert!(rows.is_empty());
}

#[test]
fn two_predicates_second_matches_none() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 5, 1, 0]).unwrap();
    eng.insert(2, vec![2, 5, 2, 0]).unwrap();
    eng.flush().unwrap();

    // col_a predicate matches both, col_b [999,999] matches nothing.
    let rows = eng.scan_where(&[(1, 5, 5), (2, 999, 999)]).unwrap();
    assert!(rows.is_empty());
}

// ── Three predicates ──────────────────────────────────────────────────────────

#[test]
fn three_predicates_all_must_pass() {
    let (eng, _dir) = open();
    //                      pk  col_a  col_b  col_c
    eng.insert(1, vec![1, 10, 100, 1000]).unwrap(); // all pass ✓
    eng.insert(2, vec![2, 10, 100, 9999]).unwrap(); // col_c fails ✗
    eng.insert(3, vec![3, 99, 100, 1000]).unwrap(); // col_a fails ✗
    eng.insert(4, vec![4, 10, 999, 1000]).unwrap(); // col_b fails ✗
    eng.flush().unwrap();

    let rows = eng
        .scan_where(&[(1, 5, 15), (2, 50, 200), (3, 500, 2000)])
        .unwrap();
    assert_eq!(pks(&rows), vec![1]);
}

// ── lo > hi fast-exit ─────────────────────────────────────────────────────────

#[test]
fn lo_gt_hi_returns_empty_immediately() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 5, 0, 0]).unwrap();
    eng.flush().unwrap();

    let rows = eng.scan_where(&[(1, 100, 50)]).unwrap(); // lo > hi
    assert!(rows.is_empty());
}

#[test]
fn lo_gt_hi_in_second_predicate_returns_empty() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 5, 0, 0]).unwrap();
    eng.flush().unwrap();

    let rows = eng.scan_where(&[(1, 5, 5), (2, 100, 0)]).unwrap();
    assert!(rows.is_empty());
}

// ── Predicate on pk column (col 0) ───────────────────────────────────────────

#[test]
fn predicate_on_pk_column() {
    let (eng, _dir) = open();
    for i in 1u64..=6 {
        eng.insert(i, vec![i, 0, 0, 0]).unwrap();
    }
    eng.flush().unwrap();

    let rows = eng.scan_where(&[(0, 2, 4)]).unwrap();
    assert_eq!(pks(&rows), vec![2, 3, 4]);
}

// ── Error handling ────────────────────────────────────────────────────────────

#[test]
fn bad_col_idx_returns_error() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 0, 0, 0]).unwrap();
    eng.flush().unwrap();

    assert!(eng.scan_where(&[(99, 0, 100)]).is_err());
}

#[test]
fn bad_col_idx_in_second_predicate_returns_error() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 5, 0, 0]).unwrap();
    eng.flush().unwrap();

    assert!(eng.scan_where(&[(1, 5, 5), (99, 0, 100)]).is_err());
}

// ── Correctness of returned values ────────────────────────────────────────────

#[test]
fn full_row_values_are_correct() {
    let (eng, _dir) = open();
    eng.insert(7, vec![7, 42, 99, 333]).unwrap();
    eng.insert(8, vec![8, 42, 0, 0]).unwrap();
    eng.flush().unwrap();

    // Only pk=7 passes both predicates.
    let rows = eng.scan_where(&[(1, 42, 42), (2, 90, 99)]).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].pk, 7);
    assert_eq!(rows[0].values, vec![7, 42, 99, 333]);
}

// ── Empty table ───────────────────────────────────────────────────────────────

#[test]
fn empty_table_returns_empty() {
    let (eng, _dir) = open();
    // Don't insert anything; flush creates no partition.
    let rows = eng.scan_where(&[(1, 0, 100)]).unwrap();
    assert!(rows.is_empty());
}

// ── Zone pruning across partitions ────────────────────────────────────────────

#[test]
fn zone_prune_skips_non_overlapping_partition() {
    let dir = TempDir::new().unwrap();
    // Small flush threshold so we get two separate partitions.
    let eng = TableEngine::open_with(
        vec!["pk".into(), "col_a".into(), "col_b".into(), "col_c".into()],
        dir.path(),
        EngineOptions::new(2).with_durability(Durability::WriteOnly),
    )
    .unwrap();

    // Partition 1: col_a in [1, 5].
    eng.insert(1, vec![1, 1, 0, 0]).unwrap();
    eng.insert(2, vec![2, 5, 0, 0]).unwrap();
    eng.flush().unwrap();

    // Partition 2: col_a in [100, 200].
    eng.insert(3, vec![3, 100, 0, 0]).unwrap();
    eng.insert(4, vec![4, 200, 0, 0]).unwrap();
    eng.flush().unwrap();

    // Predicate col_a in [90, 110] — must find pk=3 only, not pk=1 or pk=2.
    let rows = eng.scan_where(&[(1, 90, 110)]).unwrap();
    assert_eq!(pks(&rows), vec![3]);
}

// ── Scale test ────────────────────────────────────────────────────────────────

#[test]
fn many_rows_correct_subset_returned() {
    let (eng, _dir) = open();
    for i in 1u64..=60 {
        eng.insert(i, vec![i, i % 10, i % 5, 0]).unwrap();
    }
    eng.flush().unwrap();

    // col_a (i % 10) == 3 AND col_b (i % 5) == 3:
    // col_a == 3 when i % 10 == 3 → i = 3, 13, 23, 33, 43, 53
    // col_b == 3 when i % 5  == 3 → i = 3, 8, 13, 18, 23, 28, 33, 38, 43, 48, 53, 58
    // intersection: 3, 13, 23, 33, 43, 53
    let rows = eng.scan_where(&[(1, 3, 3), (2, 3, 3)]).unwrap();
    assert_eq!(pks(&rows), vec![3, 13, 23, 33, 43, 53]);
}

// ── No duplicates ─────────────────────────────────────────────────────────────

#[test]
fn no_duplicate_rows_in_result() {
    let (eng, _dir) = open();
    for i in 1u64..=10 {
        eng.insert(i, vec![i, 5, 5, 0]).unwrap();
    }
    eng.flush().unwrap();

    let rows = eng.scan_where(&[(1, 5, 5), (2, 5, 5)]).unwrap();
    let pk_list = pks(&rows);
    let mut unique = pk_list.clone();
    unique.dedup();
    assert_eq!(pk_list, unique, "no duplicate pks");
    assert_eq!(pk_list.len(), 10);
}
