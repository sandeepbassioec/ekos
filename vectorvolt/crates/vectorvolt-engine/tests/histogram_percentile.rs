/// Layer 23 — histogram and percentile integration tests.
///
/// Both `histogram_range` and `percentile_range` reuse the zone-prune +
/// single-column-decode path from Layer 21.  Tests verify bucket counts,
/// edge-case inputs (empty range, single bucket, out-of-range percentile),
/// and correctness across multiple flushed partitions.
use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, HistogramBucket, TableEngine};

fn open() -> (TableEngine, TempDir) {
    let dir = TempDir::new().unwrap();
    let eng = TableEngine::open_with(
        vec!["pk".into(), "age".into(), "score".into()],
        dir.path(),
        EngineOptions::new(1024).with_durability(Durability::WriteOnly),
    )
    .unwrap();
    (eng, dir)
}

fn populate(eng: &TableEngine, rows: &[(u64, u64, u64)]) {
    for &(pk, age, score) in rows {
        eng.insert(pk, vec![pk, age, score]).unwrap();
    }
    eng.flush().unwrap();
}

// ── histogram_range ───────────────────────────────────────────────────────────

#[test]
fn histogram_equal_distribution() {
    let (eng, _dir) = open();
    // ages 10..=40, 4 values, 4 buckets → 1 per bucket
    populate(&eng, &[(1, 10, 0), (2, 20, 0), (3, 30, 0), (4, 40, 0)]);
    let h = eng.histogram_range(1, 10, 40, 4).unwrap();
    assert_eq!(h.len(), 4);
    assert!(
        h.iter().all(|b| b.count == 1),
        "expected 1 per bucket: {h:?}"
    );
}

#[test]
fn histogram_all_in_one_bucket() {
    let (eng, _dir) = open();
    populate(&eng, &[(1, 5, 0), (2, 5, 0), (3, 5, 0)]);
    // 3 buckets over [5, 5] — all values land in the first bucket
    let h = eng.histogram_range(1, 5, 5, 3).unwrap();
    // Only one bucket actually exists (lo==hi collapses to 1)
    let total: u64 = h.iter().map(|b| b.count).sum();
    assert_eq!(total, 3);
}

#[test]
fn histogram_single_bucket() {
    let (eng, _dir) = open();
    populate(&eng, &[(1, 10, 0), (2, 50, 0), (3, 90, 0)]);
    let h = eng.histogram_range(1, 10, 90, 1).unwrap();
    assert_eq!(h.len(), 1);
    assert_eq!(h[0].count, 3);
    assert_eq!(h[0].lo, 10);
    assert_eq!(h[0].hi, 90);
}

#[test]
fn histogram_empty_when_no_match() {
    let (eng, _dir) = open();
    populate(&eng, &[(1, 5, 0), (2, 10, 0)]);
    let h = eng.histogram_range(1, 100, 200, 4).unwrap();
    let total: u64 = h.iter().map(|b| b.count).sum();
    assert_eq!(total, 0);
}

#[test]
fn histogram_returns_empty_vec_for_zero_buckets() {
    let (eng, _dir) = open();
    populate(&eng, &[(1, 10, 0)]);
    let h = eng.histogram_range(1, 0, 100, 0).unwrap();
    assert!(h.is_empty());
}

#[test]
fn histogram_returns_empty_vec_for_inverted_range() {
    let (eng, _dir) = open();
    populate(&eng, &[(1, 10, 0)]);
    let h = eng.histogram_range(1, 100, 50, 4).unwrap();
    assert!(h.is_empty());
}

#[test]
fn histogram_last_bucket_ends_at_hi() {
    let (eng, _dir) = open();
    populate(&eng, &[(1, 10, 0), (2, 19, 0), (3, 20, 0)]);
    let h = eng.histogram_range(1, 10, 20, 2).unwrap();
    assert_eq!(h.last().unwrap().hi, 20, "last bucket must end at hi");
}

#[test]
fn histogram_spans_multiple_partitions() {
    let dir = TempDir::new().unwrap();
    let eng = TableEngine::open_with(
        vec!["pk".into(), "age".into(), "score".into()],
        dir.path(),
        EngineOptions::new(5).with_durability(Durability::WriteOnly),
    )
    .unwrap();

    // Partition 1: ages 10, 20
    for pk in 1u64..=2 {
        eng.insert(pk, vec![pk, pk * 10, 0]).unwrap();
    }
    eng.flush().unwrap();

    // Partition 2: ages 30, 40
    for pk in 3u64..=4 {
        eng.insert(pk, vec![pk, pk * 10, 0]).unwrap();
    }
    eng.flush().unwrap();

    // 2 buckets: [10,24] and [25,40]
    let h = eng.histogram_range(1, 10, 40, 2).unwrap();
    let total: u64 = h.iter().map(|b| b.count).sum();
    assert_eq!(total, 4, "all 4 rows must be counted across partitions");
}

#[test]
fn histogram_rejects_bad_col_index() {
    let (eng, _dir) = open();
    populate(&eng, &[(1, 1, 0)]);
    assert!(eng.histogram_range(99, 0, 100, 4).is_err());
}

// ── percentile_range ──────────────────────────────────────────────────────────

#[test]
fn percentile_p0_is_minimum() {
    let (eng, _dir) = open();
    populate(&eng, &[(1, 10, 0), (2, 50, 0), (3, 90, 0)]);
    assert_eq!(eng.percentile_range(1, 0, 100, 0.0).unwrap(), Some(10));
}

#[test]
fn percentile_p100_is_maximum() {
    let (eng, _dir) = open();
    populate(&eng, &[(1, 10, 0), (2, 50, 0), (3, 90, 0)]);
    assert_eq!(eng.percentile_range(1, 0, 100, 100.0).unwrap(), Some(90));
}

#[test]
fn percentile_p50_is_median() {
    let (eng, _dir) = open();
    // sorted: 10, 20, 30, 40, 50 — P50 = value at index ceil(0.5*5)-1 = 2 → 30
    populate(
        &eng,
        &[(1, 30, 0), (2, 10, 0), (3, 50, 0), (4, 20, 0), (5, 40, 0)],
    );
    assert_eq!(eng.percentile_range(1, 0, 100, 50.0).unwrap(), Some(30));
}

#[test]
fn percentile_none_on_empty_range() {
    let (eng, _dir) = open();
    populate(&eng, &[(1, 5, 0)]);
    assert_eq!(eng.percentile_range(1, 100, 200, 50.0).unwrap(), None);
}

#[test]
fn percentile_out_of_range_returns_error() {
    let (eng, _dir) = open();
    populate(&eng, &[(1, 1, 0)]);
    assert!(eng.percentile_range(1, 0, 100, -1.0).is_err());
    assert!(eng.percentile_range(1, 0, 100, 101.0).is_err());
}

#[test]
fn percentile_single_row() {
    let (eng, _dir) = open();
    populate(&eng, &[(1, 42, 0)]);
    assert_eq!(eng.percentile_range(1, 0, 100, 75.0).unwrap(), Some(42));
}

#[test]
fn percentile_spans_multiple_partitions() {
    let dir = TempDir::new().unwrap();
    let eng = TableEngine::open_with(
        vec!["pk".into(), "val".into(), "z".into()],
        dir.path(),
        EngineOptions::new(5).with_durability(Durability::WriteOnly),
    )
    .unwrap();

    // Partition 1: vals 10, 20, 30
    for pk in 1u64..=3 {
        eng.insert(pk, vec![pk, pk * 10, 0]).unwrap();
    }
    eng.flush().unwrap();

    // Partition 2: vals 40, 50, 60, 70, 80, 90, 100
    for pk in 4u64..=10 {
        eng.insert(pk, vec![pk, pk * 10, 0]).unwrap();
    }
    eng.flush().unwrap();

    // sorted: 10,20,30,40,50,60,70,80,90,100 (10 values)
    // P90 = ceil(0.9*10)-1 = 9-1 = index 8 = 90
    assert_eq!(eng.percentile_range(1, 10, 100, 90.0).unwrap(), Some(90));
    // P10 = ceil(0.1*10)-1 = 1-1 = index 0 = 10
    assert_eq!(eng.percentile_range(1, 10, 100, 10.0).unwrap(), Some(10));
}

#[test]
fn percentile_rejects_bad_col_index() {
    let (eng, _dir) = open();
    populate(&eng, &[(1, 1, 0)]);
    assert!(eng.percentile_range(99, 0, 100, 50.0).is_err());
}

// ── HistogramBucket struct fields ─────────────────────────────────────────────

#[test]
fn histogram_bucket_fields_are_correct() {
    let (eng, _dir) = open();
    // ages: 1, 2, 3 → 3 buckets [1,1], [2,2], [3,3]
    populate(&eng, &[(1, 1, 0), (2, 2, 0), (3, 3, 0)]);
    let h = eng.histogram_range(1, 1, 3, 3).unwrap();
    assert_eq!(h.len(), 3);
    assert_eq!(
        h[0],
        HistogramBucket {
            lo: 1,
            hi: 1,
            count: 1
        }
    );
    assert_eq!(h[2].hi, 3);
}
