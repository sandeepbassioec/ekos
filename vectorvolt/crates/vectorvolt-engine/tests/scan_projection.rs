/// Layer 24 — column projection integration tests.
///
/// `scan_project(filter_col, lo, hi, projection)` filters rows by a column
/// range and returns only the projected columns, decoding no others.
use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};

// Schema: pk(0) | category(1) | price(2) | qty(3) | rating(4)
fn open() -> (TableEngine, TempDir) {
    let dir = TempDir::new().unwrap();
    let eng = TableEngine::open_with(
        vec![
            "pk".into(),
            "category".into(),
            "price".into(),
            "qty".into(),
            "rating".into(),
        ],
        dir.path(),
        EngineOptions::new(1024).with_durability(Durability::WriteOnly),
    )
    .unwrap();
    (eng, dir)
}

fn populate(eng: &TableEngine, rows: &[(u64, u64, u64, u64, u64)]) {
    for &(pk, cat, price, qty, rating) in rows {
        eng.insert(pk, vec![pk, cat, price, qty, rating]).unwrap();
    }
    eng.flush().unwrap();
}

// ── basic projection ──────────────────────────────────────────────────────────

#[test]
fn project_single_column() {
    let (eng, _dir) = open();
    // filter on category(1), project only price(2)
    populate(
        &eng,
        &[(1, 1, 100, 5, 4), (2, 1, 200, 3, 5), (3, 2, 300, 1, 3)],
    );
    let rows = eng.scan_project(1, 1, 1, &[2]).unwrap();
    assert_eq!(rows.len(), 2); // category == 1
    let prices: Vec<u64> = rows.iter().map(|r| r.values[0]).collect();
    assert!(prices.contains(&100));
    assert!(prices.contains(&200));
}

#[test]
fn project_multiple_columns() {
    let (eng, _dir) = open();
    populate(&eng, &[(1, 1, 100, 5, 4), (2, 2, 200, 3, 5)]);
    // filter cat==1, project price(2) and qty(3)
    let rows = eng.scan_project(1, 1, 1, &[2, 3]).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].values.len(), 2); // only 2 columns returned
    assert_eq!(rows[0].values[0], 100); // price
    assert_eq!(rows[0].values[1], 5); // qty
}

#[test]
fn project_preserves_column_order() {
    let (eng, _dir) = open();
    populate(&eng, &[(1, 1, 50, 10, 3)]);
    // project [4, 2, 1] — reverse order
    let rows = eng.scan_project(1, 1, 1, &[4, 2, 1]).unwrap();
    assert_eq!(rows[0].values[0], 3); // rating
    assert_eq!(rows[0].values[1], 50); // price
    assert_eq!(rows[0].values[2], 1); // category
}

#[test]
fn project_pk_is_always_set() {
    let (eng, _dir) = open();
    populate(&eng, &[(42, 1, 99, 7, 5)]);
    // project only rating(4) — pk must still be set from col 0
    let rows = eng.scan_project(1, 1, 1, &[4]).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].pk, 42);
    assert_eq!(rows[0].values[0], 5); // rating
}

#[test]
fn project_empty_projection_returns_pks_only() {
    let (eng, _dir) = open();
    populate(
        &eng,
        &[(10, 1, 0, 0, 0), (11, 1, 0, 0, 0), (12, 2, 0, 0, 0)],
    );
    let rows = eng.scan_project(1, 1, 1, &[]).unwrap();
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|r| r.values.is_empty()));
    let pks: Vec<u64> = rows.iter().map(|r| r.pk).collect();
    assert!(pks.contains(&10));
    assert!(pks.contains(&11));
}

#[test]
fn project_no_match_returns_empty() {
    let (eng, _dir) = open();
    populate(&eng, &[(1, 1, 10, 0, 0)]);
    let rows = eng.scan_project(1, 99, 100, &[2]).unwrap();
    assert!(rows.is_empty());
}

// ── filter col not in projection ──────────────────────────────────────────────

#[test]
fn filter_col_not_in_projection() {
    let (eng, _dir) = open();
    // filter on price(2), project only qty(3) and rating(4)
    populate(
        &eng,
        &[(1, 1, 100, 5, 4), (2, 1, 200, 3, 5), (3, 2, 500, 1, 2)],
    );
    let rows = eng.scan_project(2, 100, 200, &[3, 4]).unwrap();
    assert_eq!(rows.len(), 2);
    // values contain qty and rating only, not price
    for r in &rows {
        assert_eq!(r.values.len(), 2);
    }
}

// ── filter col in projection ──────────────────────────────────────────────────

#[test]
fn filter_col_included_in_projection() {
    let (eng, _dir) = open();
    populate(&eng, &[(1, 1, 100, 5, 4), (2, 2, 200, 3, 5)]);
    // filter on price(2), project price(2) and qty(3)
    let rows = eng.scan_project(2, 100, 100, &[2, 3]).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].values[0], 100); // price returned
    assert_eq!(rows[0].values[1], 5); // qty
}

// ── multi-partition ───────────────────────────────────────────────────────────

#[test]
fn project_spans_multiple_partitions() {
    let dir = TempDir::new().unwrap();
    let eng = TableEngine::open_with(
        vec![
            "pk".into(),
            "cat".into(),
            "price".into(),
            "qty".into(),
            "r".into(),
        ],
        dir.path(),
        EngineOptions::new(5).with_durability(Durability::WriteOnly),
    )
    .unwrap();

    // Partition 1
    for pk in 1u64..=3 {
        eng.insert(pk, vec![pk, 1, pk * 10, pk, 5]).unwrap();
    }
    eng.flush().unwrap();

    // Partition 2
    for pk in 4u64..=6 {
        eng.insert(pk, vec![pk, 1, pk * 10, pk, 5]).unwrap();
    }
    eng.flush().unwrap();

    // filter cat==1 (all 6 rows), project price(2) only
    let rows = eng.scan_project(1, 1, 1, &[2]).unwrap();
    assert_eq!(rows.len(), 6);
    let prices: Vec<u64> = rows.iter().map(|r| r.values[0]).collect();
    for pk in 1u64..=6 {
        assert!(prices.contains(&(pk * 10)), "missing price {}", pk * 10);
    }
}

// ── error cases ───────────────────────────────────────────────────────────────

#[test]
fn project_invalid_filter_col_returns_error() {
    let (eng, _dir) = open();
    populate(&eng, &[(1, 1, 0, 0, 0)]);
    assert!(eng.scan_project(99, 0, 100, &[1]).is_err());
}

#[test]
fn project_invalid_projection_col_returns_error() {
    let (eng, _dir) = open();
    populate(&eng, &[(1, 1, 0, 0, 0)]);
    assert!(eng.scan_project(1, 0, 5, &[99]).is_err());
}

// ── vs full scan correctness ──────────────────────────────────────────────────

#[test]
fn project_all_cols_matches_full_scan() {
    let (eng, _dir) = open();
    populate(
        &eng,
        &[(1, 1, 10, 2, 3), (2, 1, 20, 4, 5), (3, 2, 30, 6, 1)],
    );
    let full = eng.scan(1, 1, 1).unwrap();
    let projected = eng.scan_project(1, 1, 1, &[0, 1, 2, 3, 4]).unwrap();
    assert_eq!(full.len(), projected.len());
    // values in scan_project([0,1,2,3,4]) must match the full scan values
    let mut full_sorted = full.clone();
    let mut proj_sorted = projected.clone();
    full_sorted.sort_by_key(|r| r.pk);
    proj_sorted.sort_by_key(|r| r.pk);
    for (f, p) in full_sorted.iter().zip(proj_sorted.iter()) {
        assert_eq!(f.values, p.values);
    }
}
