/// Layer 19 — query DSL integration tests.
///
/// Each test parses a query string and executes it against a live TableEngine
/// to verify that the full lexer → parser → executor pipeline works end-to-end.
use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};
use vectorvolt_query::{run, QueryResponse};

fn open() -> (TableEngine, TempDir) {
    let dir = TempDir::new().unwrap();
    let eng = TableEngine::open_with(
        vec!["pk".into(), "age".into(), "score".into()],
        dir.path(),
        EngineOptions::new(1_048_576).with_durability(Durability::WriteOnly),
    )
    .unwrap();
    (eng, dir)
}

// ── insert + get ──────────────────────────────────────────────────────────────

#[test]
fn insert_and_get_row() {
    let (eng, _dir) = open();
    run(&eng, "INSERT 1 1 25 99").unwrap();
    let resp = run(&eng, "GET 1").unwrap();
    match resp {
        QueryResponse::Row(row) => {
            assert_eq!(row.pk, 1);
            assert_eq!(row.values[1], 25);
            assert_eq!(row.values[2], 99);
        }
        other => panic!("expected Row, got {other:?}"),
    }
}

#[test]
fn get_missing_row_returns_not_found() {
    let (eng, _dir) = open();
    let resp = run(&eng, "GET 9999").unwrap();
    assert!(matches!(resp, QueryResponse::NotFound));
}

// ── delete ────────────────────────────────────────────────────────────────────

#[test]
fn delete_row_then_get_not_found() {
    let (eng, _dir) = open();
    run(&eng, "INSERT 5 5 0 0").unwrap();
    run(&eng, "DELETE 5").unwrap();
    let resp = run(&eng, "GET 5").unwrap();
    assert!(matches!(resp, QueryResponse::NotFound));
}

// ── scan ──────────────────────────────────────────────────────────────────────

#[test]
fn scan_returns_range() {
    let (eng, _dir) = open();
    for pk in 1u64..=5 {
        run(&eng, &format!("INSERT {pk} {pk} {} 0", pk * 10)).unwrap();
    }
    let resp = run(&eng, "SCAN col=1 lo=20 hi=30").unwrap();
    match resp {
        QueryResponse::Rows(rows) => {
            let mut pks: Vec<u64> = rows.iter().map(|r| r.pk).collect();
            pks.sort();
            assert_eq!(pks, vec![2, 3]);
        }
        other => panic!("expected Rows, got {other:?}"),
    }
}

// ── secondary ────────────────────────────────────────────────────────────────

#[test]
fn secondary_lookup_returns_matching_rows() {
    let (eng, _dir) = open();
    for pk in 1u64..=3 {
        run(&eng, &format!("INSERT {pk} {pk} 55 0")).unwrap();
    }
    run(&eng, "INSERT 4 4 99 0").unwrap();
    let resp = run(&eng, "SECONDARY col=1 val=55").unwrap();
    match resp {
        QueryResponse::Rows(rows) => {
            let mut pks: Vec<u64> = rows.iter().map(|r| r.pk).collect();
            pks.sort();
            assert_eq!(pks, vec![1, 2, 3]);
        }
        other => panic!("expected Rows, got {other:?}"),
    }
}

// ── describe + stats ──────────────────────────────────────────────────────────

#[test]
fn describe_returns_schema() {
    let (eng, _dir) = open();
    let resp = run(&eng, "DESCRIBE").unwrap();
    match resp {
        QueryResponse::Schema(cols) => {
            assert_eq!(cols, vec!["pk", "age", "score"]);
        }
        other => panic!("expected Schema, got {other:?}"),
    }
}

#[test]
fn stats_returns_engine_stats() {
    let (eng, _dir) = open();
    run(&eng, "INSERT 1 1 0 0").unwrap();
    let resp = run(&eng, "STATS").unwrap();
    match resp {
        QueryResponse::Stats(s) => {
            assert!(s.rows_buffered >= 1, "at least 1 row in buffer");
        }
        other => panic!("expected Stats, got {other:?}"),
    }
}

// ── transactions ──────────────────────────────────────────────────────────────

#[test]
fn begin_returns_txn_id() {
    let (eng, _dir) = open();
    let resp = run(&eng, "BEGIN").unwrap();
    assert!(matches!(resp, QueryResponse::TxnStarted(_)));
}

#[test]
fn txn_insert_commit_visible() {
    let (eng, _dir) = open();
    let txn_id = match run(&eng, "BEGIN").unwrap() {
        QueryResponse::TxnStarted(id) => id,
        other => panic!("expected TxnStarted, got {other:?}"),
    };
    run(&eng, &format!("INSERT TXN {txn_id} 10 10 50 1")).unwrap();
    // not yet visible
    assert!(matches!(
        run(&eng, "GET 10").unwrap(),
        QueryResponse::NotFound
    ));
    // commit
    run(&eng, &format!("COMMIT {txn_id}")).unwrap();
    // now visible
    match run(&eng, "GET 10").unwrap() {
        QueryResponse::Row(row) => assert_eq!(row.values[1], 50),
        other => panic!("expected Row after commit, got {other:?}"),
    }
}

#[test]
fn txn_rollback_discards_insert() {
    let (eng, _dir) = open();
    let txn_id = match run(&eng, "BEGIN").unwrap() {
        QueryResponse::TxnStarted(id) => id,
        other => panic!("{other:?}"),
    };
    run(&eng, &format!("INSERT TXN {txn_id} 20 20 0 0")).unwrap();
    run(&eng, &format!("ROLLBACK {txn_id}")).unwrap();
    assert!(matches!(
        run(&eng, "GET 20").unwrap(),
        QueryResponse::NotFound
    ));
}

#[test]
fn get_txn_sees_pending_insert() {
    let (eng, _dir) = open();
    let txn_id = match run(&eng, "BEGIN").unwrap() {
        QueryResponse::TxnStarted(id) => id,
        other => panic!("{other:?}"),
    };
    run(&eng, &format!("INSERT TXN {txn_id} 7 7 77 0")).unwrap();
    match run(&eng, &format!("GET TXN {txn_id} 7")).unwrap() {
        QueryResponse::Row(row) => assert_eq!(row.values[1], 77),
        other => panic!("expected Row, got {other:?}"),
    }
    run(&eng, &format!("ROLLBACK {txn_id}")).unwrap();
}

// ── parser error handling ─────────────────────────────────────────────────────

#[test]
fn parse_error_on_unknown_keyword() {
    let (eng, _dir) = open();
    let err = run(&eng, "FOOBAR 1").unwrap_err();
    assert!(matches!(err, vectorvolt_query::QueryError::Syntax(_)));
}

#[test]
fn parse_error_on_missing_pk() {
    let (eng, _dir) = open();
    let err = run(&eng, "GET").unwrap_err();
    assert!(matches!(err, vectorvolt_query::QueryError::UnexpectedEof));
}
