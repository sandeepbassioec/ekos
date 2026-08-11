/// Layer 21 — aggregate pushdown integration tests.
///
/// Verifies that count_range / sum_range / min_range / max_range / avg_range
/// produce correct results and (where verifiable) never materialise full rows
/// — only the target column is decoded per partition.
use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};

fn open() -> (TableEngine, TempDir) {
    let dir = TempDir::new().unwrap();
    let eng = TableEngine::open_with(
        vec!["pk".into(), "age".into(), "score".into()],
        dir.path(),
        EngineOptions::new(128).with_durability(Durability::WriteOnly),
    )
    .unwrap();
    (eng, dir)
}

/// Insert rows with age and score, then flush to create a partition.
/// flush_threshold=128 ensures rows are flushed after enough inserts.
fn populate(eng: &TableEngine, rows: &[(u64, u64, u64)]) {
    for &(pk, age, score) in rows {
        eng.insert(pk, vec![pk, age, score]).unwrap();
    }
    eng.flush().unwrap();
}

// ── count_range ───────────────────────────────────────────────────────────────

#[test]
fn count_range_exact_match() {
    let (eng, _dir) = open();
    populate(&eng, &[(1, 25, 0), (2, 30, 0), (3, 25, 0), (4, 40, 0)]);
    assert_eq!(eng.count_range(1, 25, 25).unwrap(), 2);
}

#[test]
fn count_range_inclusive_bounds() {
    let (eng, _dir) = open();
    populate(&eng, &[(1, 10, 0), (2, 20, 0), (3, 30, 0), (4, 40, 0)]);
    assert_eq!(eng.count_range(1, 20, 30).unwrap(), 2);
}

#[test]
fn count_range_empty_result() {
    let (eng, _dir) = open();
    populate(&eng, &[(1, 10, 0), (2, 20, 0)]);
    assert_eq!(eng.count_range(1, 100, 200).unwrap(), 0);
}

#[test]
fn count_range_all_rows() {
    let (eng, _dir) = open();
    let rows: Vec<(u64, u64, u64)> = (1..=20).map(|i| (i, i * 5, 0)).collect();
    populate(&eng, &rows);
    assert_eq!(eng.count_range(1, 5, 100).unwrap(), 20);
}

// ── sum_range ─────────────────────────────────────────────────────────────────

#[test]
fn sum_range_basic() {
    let (eng, _dir) = open();
    // ages: 10, 20, 30, 40; filter lo=20 hi=30 → 20+30 = 50
    populate(&eng, &[(1, 10, 0), (2, 20, 0), (3, 30, 0), (4, 40, 0)]);
    assert_eq!(eng.sum_range(1, 20, 30).unwrap(), 50);
}

#[test]
fn sum_range_no_match_returns_zero() {
    let (eng, _dir) = open();
    populate(&eng, &[(1, 5, 0), (2, 10, 0)]);
    assert_eq!(eng.sum_range(1, 100, 200).unwrap(), 0);
}

#[test]
fn sum_range_on_score_column() {
    let (eng, _dir) = open();
    // filter ages 1..=5 (all), sum of scores
    populate(&eng, &[(1, 1, 100), (2, 2, 200), (3, 3, 300)]);
    let total = eng.sum_range(2, 100, 300).unwrap();
    assert_eq!(total, 600);
}

// ── min_range ─────────────────────────────────────────────────────────────────

#[test]
fn min_range_returns_smallest() {
    let (eng, _dir) = open();
    populate(&eng, &[(1, 30, 0), (2, 10, 0), (3, 20, 0)]);
    assert_eq!(eng.min_range(1, 0, 100).unwrap(), Some(10));
}

#[test]
fn min_range_returns_none_on_empty() {
    let (eng, _dir) = open();
    populate(&eng, &[(1, 50, 0)]);
    assert_eq!(eng.min_range(1, 100, 200).unwrap(), None);
}

#[test]
fn min_range_respects_filter() {
    let (eng, _dir) = open();
    // Only ages 30,40 pass lo=30 hi=40; min of those is 30.
    populate(&eng, &[(1, 10, 0), (2, 30, 0), (3, 40, 0), (4, 50, 0)]);
    assert_eq!(eng.min_range(1, 30, 40).unwrap(), Some(30));
}

// ── max_range ─────────────────────────────────────────────────────────────────

#[test]
fn max_range_returns_largest() {
    let (eng, _dir) = open();
    populate(&eng, &[(1, 5, 0), (2, 99, 0), (3, 42, 0)]);
    assert_eq!(eng.max_range(1, 0, 200).unwrap(), Some(99));
}

#[test]
fn max_range_returns_none_on_empty() {
    let (eng, _dir) = open();
    populate(&eng, &[(1, 5, 0)]);
    assert_eq!(eng.max_range(1, 100, 200).unwrap(), None);
}

// ── avg_range ─────────────────────────────────────────────────────────────────

#[test]
fn avg_range_exact() {
    let (eng, _dir) = open();
    // ages 10, 20, 30 → avg = 20.0
    populate(&eng, &[(1, 10, 0), (2, 20, 0), (3, 30, 0)]);
    let avg = eng.avg_range(1, 10, 30).unwrap().unwrap();
    assert!((avg - 20.0).abs() < 1e-9, "expected 20.0 got {avg}");
}

#[test]
fn avg_range_none_on_empty() {
    let (eng, _dir) = open();
    populate(&eng, &[(1, 5, 0)]);
    assert!(eng.avg_range(1, 100, 200).unwrap().is_none());
}

// ── multi-partition ───────────────────────────────────────────────────────────

#[test]
fn aggregates_span_multiple_partitions() {
    let dir = TempDir::new().unwrap();
    // flush_threshold=5: 3 inserts each round → 2 separate partitions
    let eng = TableEngine::open_with(
        vec!["pk".into(), "age".into(), "score".into()],
        dir.path(),
        EngineOptions::new(5).with_durability(Durability::WriteOnly),
    )
    .unwrap();

    // First partition: pks 1-5, ages 10-50
    for pk in 1u64..=5 {
        eng.insert(pk, vec![pk, pk * 10, 0]).unwrap();
    }
    eng.flush().unwrap();

    // Second partition: pks 6-10, ages 60-100
    for pk in 6u64..=10 {
        eng.insert(pk, vec![pk, pk * 10, 0]).unwrap();
    }
    eng.flush().unwrap();

    // count all: 10 rows total
    assert_eq!(eng.count_range(1, 10, 100).unwrap(), 10);
    // sum: 10+20+30+40+50+60+70+80+90+100 = 550
    assert_eq!(eng.sum_range(1, 10, 100).unwrap(), 550);
    assert_eq!(eng.min_range(1, 10, 100).unwrap(), Some(10));
    assert_eq!(eng.max_range(1, 10, 100).unwrap(), Some(100));
    let avg = eng.avg_range(1, 10, 100).unwrap().unwrap();
    assert!((avg - 55.0).abs() < 1e-9, "expected avg=55, got {avg}");
}

// ── error: col out of range ───────────────────────────────────────────────────

#[test]
fn count_range_rejects_bad_col_index() {
    let (eng, _dir) = open();
    populate(&eng, &[(1, 1, 0)]);
    assert!(eng.count_range(99, 0, 100).is_err());
    assert!(eng.sum_range(99, 0, 100).is_err());
    assert!(eng.min_range(99, 0, 100).is_err());
    assert!(eng.max_range(99, 0, 100).is_err());
    assert!(eng.avg_range(99, 0, 100).is_err());
}
