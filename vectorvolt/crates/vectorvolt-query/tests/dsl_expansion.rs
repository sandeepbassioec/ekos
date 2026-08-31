/// Layer 43 — DSL expansion: analytical + write/paginate methods.
///
/// Schema: pk | val | score  (3 columns).
use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};
use vectorvolt_query::{QueryResponse, run};

fn open() -> (TableEngine, TempDir) {
    let dir = TempDir::new().unwrap();
    let eng = TableEngine::open_with(
        vec!["pk".into(), "val".into(), "score".into()],
        dir.path(),
        EngineOptions::new(1024).with_durability(Durability::WriteOnly),
    )
    .unwrap();
    (eng, dir)
}

// ── COUNT ─────────────────────────────────────────────────────────────────────

#[test]
fn count_returns_correct_value() {
    let (eng, _dir) = open();
    for i in 1u64..=5 {
        eng.insert(i, vec![i, i * 10, 0]).unwrap();
    }
    let resp = run(&eng, "COUNT col=1 lo=10 hi=30").unwrap();
    match resp {
        QueryResponse::Count(n) => assert_eq!(n, 3),
        other => panic!("expected Count, got {other:?}"),
    }
}

#[test]
fn count_returns_zero_when_no_match() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 99, 0]).unwrap();
    let resp = run(&eng, "COUNT col=1 lo=0 hi=10").unwrap();
    match resp {
        QueryResponse::Count(n) => assert_eq!(n, 0),
        other => panic!("expected Count, got {other:?}"),
    }
}

// ── SUM ───────────────────────────────────────────────────────────────────────

#[test]
fn sum_returns_correct_total() {
    let (eng, _dir) = open();
    for i in 1u64..=4 {
        eng.insert(i, vec![i, i, 0]).unwrap(); // val = 1+2+3+4 = 10
    }
    let resp = run(&eng, "SUM col=1 lo=1 hi=4").unwrap();
    match resp {
        QueryResponse::Aggregate(Some(v)) => assert_eq!(v, 10),
        other => panic!("expected Aggregate(Some(10)), got {other:?}"),
    }
}

// ── MIN ───────────────────────────────────────────────────────────────────────

#[test]
fn min_returns_smallest_value() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 50, 0]).unwrap();
    eng.insert(2, vec![2, 20, 0]).unwrap();
    eng.insert(3, vec![3, 80, 0]).unwrap();
    let resp = run(&eng, "MIN col=1 lo=1 hi=100").unwrap();
    match resp {
        QueryResponse::Aggregate(Some(v)) => assert_eq!(v, 20),
        other => panic!("expected Aggregate(Some(20)), got {other:?}"),
    }
}

// ── MAX ───────────────────────────────────────────────────────────────────────

#[test]
fn max_returns_largest_value() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 50, 0]).unwrap();
    eng.insert(2, vec![2, 20, 0]).unwrap();
    eng.insert(3, vec![3, 80, 0]).unwrap();
    let resp = run(&eng, "MAX col=1 lo=1 hi=100").unwrap();
    match resp {
        QueryResponse::Aggregate(Some(v)) => assert_eq!(v, 80),
        other => panic!("expected Aggregate(Some(80)), got {other:?}"),
    }
}

// ── AVG ───────────────────────────────────────────────────────────────────────

#[test]
fn avg_returns_rounded_mean() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 10, 0]).unwrap();
    eng.insert(2, vec![2, 20, 0]).unwrap();
    eng.insert(3, vec![3, 30, 0]).unwrap(); // avg = 20
    let resp = run(&eng, "AVG col=1 lo=1 hi=100").unwrap();
    match resp {
        QueryResponse::Aggregate(Some(v)) => assert_eq!(v, 20),
        other => panic!("expected Aggregate(Some(20)), got {other:?}"),
    }
}

#[test]
fn avg_returns_none_when_no_rows() {
    let (eng, _dir) = open();
    let resp = run(&eng, "AVG col=1 lo=1 hi=100").unwrap();
    match resp {
        QueryResponse::Aggregate(None) => {}
        other => panic!("expected Aggregate(None), got {other:?}"),
    }
}

// ── SCAN SORTED ──────────────────────────────────────────────────────────────

#[test]
fn scan_sorted_returns_rows_in_order() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 30, 0]).unwrap();
    eng.insert(2, vec![2, 10, 0]).unwrap();
    eng.insert(3, vec![3, 20, 0]).unwrap();
    let resp = run(&eng, "SCAN SORTED col=0 lo=1 hi=3 sort=1 asc=1 limit=10").unwrap();
    match resp {
        QueryResponse::Rows(rows) => {
            let vals: Vec<u64> = rows.iter().map(|r| r.values[1]).collect();
            assert_eq!(vals, vec![10, 20, 30]);
        }
        other => panic!("expected Rows, got {other:?}"),
    }
}

#[test]
fn scan_sorted_descending() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 30, 0]).unwrap();
    eng.insert(2, vec![2, 10, 0]).unwrap();
    eng.insert(3, vec![3, 20, 0]).unwrap();
    let resp = run(&eng, "SCAN SORTED col=0 lo=1 hi=3 sort=1 asc=0 limit=10").unwrap();
    match resp {
        QueryResponse::Rows(rows) => {
            let vals: Vec<u64> = rows.iter().map(|r| r.values[1]).collect();
            assert_eq!(vals, vec![30, 20, 10]);
        }
        other => panic!("expected Rows, got {other:?}"),
    }
}

// ── SCAN PAGE ─────────────────────────────────────────────────────────────────

#[test]
fn scan_page_first_page() {
    let (eng, _dir) = open();
    for i in 1u64..=5 {
        eng.insert(i, vec![i, i, 0]).unwrap();
    }
    let resp = run(&eng, "SCAN PAGE col=0 lo=1 hi=5 after=0 limit=3").unwrap();
    match resp {
        QueryResponse::Page(page) => {
            assert_eq!(page.rows.len(), 3);
            assert!(page.next_cursor.is_some(), "must have next cursor");
        }
        other => panic!("expected Page, got {other:?}"),
    }
}

#[test]
fn scan_page_second_page() {
    let (eng, _dir) = open();
    for i in 1u64..=5 {
        eng.insert(i, vec![i, i, 0]).unwrap();
    }
    let cursor = match run(&eng, "SCAN PAGE col=0 lo=1 hi=5 after=0 limit=3").unwrap() {
        QueryResponse::Page(p) => p.next_cursor.unwrap(),
        other => panic!("{other:?}"),
    };
    let resp = run(
        &eng,
        &format!("SCAN PAGE col=0 lo=1 hi=5 after={cursor} limit=3"),
    )
    .unwrap();
    match resp {
        QueryResponse::Page(page) => {
            assert_eq!(page.rows.len(), 2, "second page has remaining 2 rows");
            assert!(page.next_cursor.is_none(), "no more pages");
        }
        other => panic!("expected Page, got {other:?}"),
    }
}

// ── UPSERT ────────────────────────────────────────────────────────────────────

#[test]
fn upsert_inserts_when_absent() {
    let (eng, _dir) = open();
    let resp = run(&eng, "UPSERT 1 1 42 0").unwrap();
    assert!(matches!(resp, QueryResponse::Ok));
    let row = eng.get(1).unwrap().unwrap();
    assert_eq!(row.values[1], 42);
}

#[test]
fn upsert_updates_when_present() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 10, 0]).unwrap();
    run(&eng, "UPSERT 1 1 99 0").unwrap();
    let row = eng.get(1).unwrap().unwrap();
    assert_eq!(row.values[1], 99, "upsert must overwrite existing value");
}

// ── UPDATE FIELD ──────────────────────────────────────────────────────────────

#[test]
fn update_field_changes_value() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 5, 0]).unwrap();
    let resp = run(&eng, "UPDATE FIELD 1 col=1 val=77").unwrap();
    assert!(matches!(resp, QueryResponse::Ok));
    let row = eng.get(1).unwrap().unwrap();
    assert_eq!(row.values[1], 77);
}

// ── DELETE RANGE ──────────────────────────────────────────────────────────────

#[test]
fn delete_range_removes_matching_rows() {
    let (eng, _dir) = open();
    for i in 1u64..=5 {
        eng.insert(i, vec![i, i, 0]).unwrap();
    }
    let resp = run(&eng, "DELETE RANGE col=0 lo=2 hi=4").unwrap();
    match resp {
        QueryResponse::Count(n) => assert_eq!(n, 3, "pks 2,3,4 must be deleted"),
        other => panic!("expected Count, got {other:?}"),
    }
    assert!(eng.get(1).unwrap().is_some());
    assert!(eng.get(2).unwrap().is_none());
    assert!(eng.get(5).unwrap().is_some());
}
