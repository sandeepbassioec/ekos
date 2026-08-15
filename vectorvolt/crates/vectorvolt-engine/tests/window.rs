/// Layer 35 — window function integration tests.
///
/// Schema: pk | filter | score | extra
use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine, WindowRow};

fn open() -> (TableEngine, TempDir) {
    let dir = TempDir::new().unwrap();
    let eng = TableEngine::open_with(
        vec!["pk".into(), "filter".into(), "score".into(), "extra".into()],
        dir.path(),
        EngineOptions::new(1024).with_durability(Durability::WriteOnly),
    )
    .unwrap();
    (eng, dir)
}

// ── running_sum ───────────────────────────────────────────────────────────────

#[test]
fn running_sum_prefix_sums_in_pk_order() {
    let (eng, _dir) = open();
    //                      pk  filter  score  extra
    eng.insert(3, vec![3, 1, 30, 0]).unwrap();
    eng.insert(1, vec![1, 1, 10, 0]).unwrap();
    eng.insert(2, vec![2, 1, 20, 0]).unwrap();
    eng.flush().unwrap();

    let result = eng.running_sum(1, 1, 1, 2).unwrap();
    // Sorted by pk: 1(score=10), 2(score=20), 3(score=30)
    // Prefix sums:          10,           30,          60
    assert_eq!(
        result,
        vec![
            WindowRow {
                pk: 1,
                window_val: 10
            },
            WindowRow {
                pk: 2,
                window_val: 30
            },
            WindowRow {
                pk: 3,
                window_val: 60
            },
        ]
    );
}

#[test]
fn running_sum_filter_col_subsets_rows() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 1, 5, 0]).unwrap(); // filter=1 ✓
    eng.insert(2, vec![2, 2, 99, 0]).unwrap(); // filter=2 ✗ (excluded)
    eng.insert(3, vec![3, 1, 10, 0]).unwrap(); // filter=1 ✓
    eng.flush().unwrap();

    let result = eng.running_sum(1, 1, 1, 2).unwrap();
    assert_eq!(
        result,
        vec![
            WindowRow {
                pk: 1,
                window_val: 5
            },
            WindowRow {
                pk: 3,
                window_val: 15
            },
        ]
    );
}

#[test]
fn running_sum_wraps_on_u64_overflow() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 1, u64::MAX, 0]).unwrap();
    eng.insert(2, vec![2, 1, 1, 0]).unwrap();
    eng.flush().unwrap();

    let result = eng.running_sum(1, 1, 1, 2).unwrap();
    // u64::MAX + 1 wraps to 0
    assert_eq!(result[0].window_val, u64::MAX);
    assert_eq!(result[1].window_val, 0);
}

#[test]
fn running_sum_single_row() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 1, 42, 0]).unwrap();
    eng.flush().unwrap();

    let result = eng.running_sum(1, 1, 1, 2).unwrap();
    assert_eq!(
        result,
        vec![WindowRow {
            pk: 1,
            window_val: 42
        }]
    );
}

#[test]
fn running_sum_no_matching_rows_is_empty() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 5, 10, 0]).unwrap();
    eng.flush().unwrap();

    let result = eng.running_sum(1, 99, 99, 2).unwrap();
    assert!(result.is_empty());
}

// ── running_count ─────────────────────────────────────────────────────────────

#[test]
fn running_count_sequential_row_numbers() {
    let (eng, _dir) = open();
    for i in [5u64, 3, 1, 4, 2] {
        eng.insert(i, vec![i, 1, 0, 0]).unwrap();
    }
    eng.flush().unwrap();

    let result = eng.running_count(1, 1, 1).unwrap();
    // Sorted by pk: 1,2,3,4,5 → row numbers 1,2,3,4,5
    assert_eq!(result.len(), 5);
    for (i, row) in result.iter().enumerate() {
        assert_eq!(row.pk, i as u64 + 1);
        assert_eq!(row.window_val, i as u64 + 1);
    }
}

#[test]
fn running_count_filter_shrinks_numbering() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 1, 0, 0]).unwrap(); // included
    eng.insert(2, vec![2, 2, 0, 0]).unwrap(); // excluded (filter=2)
    eng.insert(3, vec![3, 1, 0, 0]).unwrap(); // included
    eng.flush().unwrap();

    let result = eng.running_count(1, 1, 1).unwrap();
    // Only pk 1 and pk 3 included; row numbers restart at 1.
    assert_eq!(
        result,
        vec![
            WindowRow {
                pk: 1,
                window_val: 1
            },
            WindowRow {
                pk: 3,
                window_val: 2
            },
        ]
    );
}

// ── rank_by ───────────────────────────────────────────────────────────────────

#[test]
fn rank_by_no_ties_ascending() {
    let (eng, _dir) = open();
    //                      pk  filter  score  extra
    eng.insert(1, vec![1, 1, 30, 0]).unwrap();
    eng.insert(2, vec![2, 1, 10, 0]).unwrap();
    eng.insert(3, vec![3, 1, 20, 0]).unwrap();
    eng.flush().unwrap();

    let result = eng.rank_by(1, 1, 1, 2, true).unwrap();
    // score order asc: pk=2(10)→rank1, pk=3(20)→rank2, pk=1(30)→rank3
    assert_eq!(
        result,
        vec![
            WindowRow {
                pk: 2,
                window_val: 1
            },
            WindowRow {
                pk: 3,
                window_val: 2
            },
            WindowRow {
                pk: 1,
                window_val: 3
            },
        ]
    );
}

#[test]
fn rank_by_no_ties_descending() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 1, 30, 0]).unwrap();
    eng.insert(2, vec![2, 1, 10, 0]).unwrap();
    eng.insert(3, vec![3, 1, 20, 0]).unwrap();
    eng.flush().unwrap();

    let result = eng.rank_by(1, 1, 1, 2, false).unwrap();
    // score desc: pk=1(30)→rank1, pk=3(20)→rank2, pk=2(10)→rank3
    assert_eq!(
        result,
        vec![
            WindowRow {
                pk: 1,
                window_val: 1
            },
            WindowRow {
                pk: 3,
                window_val: 2
            },
            WindowRow {
                pk: 2,
                window_val: 3
            },
        ]
    );
}

#[test]
fn rank_by_ties_get_same_rank_with_gap() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 1, 10, 0]).unwrap();
    eng.insert(2, vec![2, 1, 10, 0]).unwrap(); // ties with pk=1
    eng.insert(3, vec![3, 1, 20, 0]).unwrap();
    eng.flush().unwrap();

    let result = eng.rank_by(1, 1, 1, 2, true).unwrap();
    // score asc: pk=1 score=10 → rank 1
    //            pk=2 score=10 → rank 1 (tie)
    //            pk=3 score=20 → rank 3 (gap: 1,1,3)
    assert_eq!(result[0].window_val, 1);
    assert_eq!(result[1].window_val, 1);
    assert_eq!(result[2].window_val, 3);
    assert_eq!(result[2].pk, 3);
}

#[test]
fn rank_by_single_row_gets_rank_one() {
    let (eng, _dir) = open();
    eng.insert(7, vec![7, 1, 50, 0]).unwrap();
    eng.flush().unwrap();

    let result = eng.rank_by(1, 1, 1, 2, true).unwrap();
    assert_eq!(
        result,
        vec![WindowRow {
            pk: 7,
            window_val: 1
        }]
    );
}

#[test]
fn rank_by_empty_filter_is_empty() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 5, 0, 0]).unwrap();
    eng.flush().unwrap();

    let result = eng.rank_by(1, 99, 99, 2, true).unwrap();
    assert!(result.is_empty());
}

// ── dense_rank_by ─────────────────────────────────────────────────────────────

#[test]
fn dense_rank_no_ties_same_as_rank() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 1, 10, 0]).unwrap();
    eng.insert(2, vec![2, 1, 20, 0]).unwrap();
    eng.insert(3, vec![3, 1, 30, 0]).unwrap();
    eng.flush().unwrap();

    let rank = eng.rank_by(1, 1, 1, 2, true).unwrap();
    let dense = eng.dense_rank_by(1, 1, 1, 2, true).unwrap();
    assert_eq!(rank, dense, "no ties: rank == dense_rank");
}

#[test]
fn dense_rank_ties_no_gap() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 1, 10, 0]).unwrap();
    eng.insert(2, vec![2, 1, 10, 0]).unwrap(); // tie
    eng.insert(3, vec![3, 1, 20, 0]).unwrap();
    eng.flush().unwrap();

    let result = eng.dense_rank_by(1, 1, 1, 2, true).unwrap();
    // score asc: 10 → dense rank 1, 10 → dense rank 1, 20 → dense rank 2 (no gap)
    assert_eq!(result[0].window_val, 1);
    assert_eq!(result[1].window_val, 1);
    assert_eq!(result[2].window_val, 2); // 2, not 3
}

#[test]
fn dense_rank_all_tied_all_get_rank_one() {
    let (eng, _dir) = open();
    for pk in 1u64..=5 {
        eng.insert(pk, vec![pk, 1, 42, 0]).unwrap(); // all score=42
    }
    eng.flush().unwrap();

    let result = eng.dense_rank_by(1, 1, 1, 2, true).unwrap();
    assert!(result.iter().all(|r| r.window_val == 1));
}

#[test]
fn rank_vs_dense_rank_gap_difference() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 1, 5, 0]).unwrap();
    eng.insert(2, vec![2, 1, 5, 0]).unwrap(); // tie with pk=1
    eng.insert(3, vec![3, 1, 5, 0]).unwrap(); // three-way tie
    eng.insert(4, vec![4, 1, 10, 0]).unwrap(); // distinct
    eng.flush().unwrap();

    let rank = eng.rank_by(1, 1, 1, 2, true).unwrap();
    let dense = eng.dense_rank_by(1, 1, 1, 2, true).unwrap();

    // Standard rank: 1,1,1,4 (gap of 3)
    assert!(rank.iter().take(3).all(|r| r.window_val == 1));
    assert_eq!(rank[3].window_val, 4);

    // Dense rank: 1,1,1,2 (no gap)
    assert!(dense.iter().take(3).all(|r| r.window_val == 1));
    assert_eq!(dense[3].window_val, 2);
}

// ── Error paths ───────────────────────────────────────────────────────────────

#[test]
fn bad_filter_col_returns_error() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 1, 0, 0]).unwrap();
    eng.flush().unwrap();

    assert!(eng.running_sum(99, 1, 1, 2).is_err());
    assert!(eng.running_count(99, 1, 1).is_err());
    assert!(eng.rank_by(99, 1, 1, 2, true).is_err());
    assert!(eng.dense_rank_by(99, 1, 1, 2, true).is_err());
}

#[test]
fn bad_sort_or_agg_col_returns_error() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 1, 0, 0]).unwrap();
    eng.flush().unwrap();

    assert!(eng.running_sum(1, 1, 1, 99).is_err());
    assert!(eng.rank_by(1, 1, 1, 99, true).is_err());
    assert!(eng.dense_rank_by(1, 1, 1, 99, true).is_err());
}
