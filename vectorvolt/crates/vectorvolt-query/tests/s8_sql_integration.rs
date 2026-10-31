//! S8 integration tests — SQL + Vector end-to-end (VV-QRY-020, VV-IDX-012).
//!
//! These tests exercise the full SQL executor and HNSW vector index through
//! the `SqlExecutor` and `TableEngine` APIs.

use std::sync::Arc;
use tempfile::TempDir;
use vectorvolt_engine::{EngineOptions, TableRegistry};
use vectorvolt_query::sql::executor::{QueryResult, SqlExecutor};

fn make_executor(dir: &std::path::Path) -> SqlExecutor {
    let registry = TableRegistry::open(dir, EngineOptions::default()).unwrap();
    SqlExecutor::new(Arc::new(registry))
}

// -- SQL DDL -----------------------------------------------------------------

#[test]
fn test_create_table_and_drop() {
    let d = TempDir::new().unwrap();
    let e = make_executor(d.path());
    let r = e.execute("CREATE TABLE users (id u64, age u64)").unwrap();
    assert!(matches!(r, QueryResult::Created));
    let r2 = e.execute("DROP TABLE users").unwrap();
    assert!(matches!(r2, QueryResult::Dropped));
}

#[test]
fn test_drop_nonexistent_table_errors() {
    let d = TempDir::new().unwrap();
    let e = make_executor(d.path());
    assert!(e.execute("DROP TABLE ghost").is_err());
}

// -- SQL DML -----------------------------------------------------------------

#[test]
fn test_insert_and_select_star() {
    let d = TempDir::new().unwrap();
    let e = make_executor(d.path());
    e.execute("CREATE TABLE t (id u64, val u64)").unwrap();
    e.execute("INSERT INTO t (id, val) VALUES (1, 100)")
        .unwrap();
    e.execute("INSERT INTO t (id, val) VALUES (2, 200)")
        .unwrap();
    let r = e.execute("SELECT * FROM t").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 2),
        _ => panic!("expected rows"),
    }
}

#[test]
fn test_select_where_eq() {
    let d = TempDir::new().unwrap();
    let e = make_executor(d.path());
    e.execute("CREATE TABLE t (id u64, val u64)").unwrap();
    e.execute("INSERT INTO t (id, val) VALUES (1, 42)").unwrap();
    e.execute("INSERT INTO t (id, val) VALUES (2, 99)").unwrap();
    let r = e.execute("SELECT * FROM t WHERE id = 1").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0][0], "1");
        }
        _ => panic!("expected rows"),
    }
}

#[test]
fn test_delete_where() {
    let d = TempDir::new().unwrap();
    let e = make_executor(d.path());
    e.execute("CREATE TABLE t (id u64, val u64)").unwrap();
    e.execute("INSERT INTO t (id, val) VALUES (1, 10)").unwrap();
    e.execute("INSERT INTO t (id, val) VALUES (2, 20)").unwrap();
    e.execute("DELETE FROM t WHERE id = 1").unwrap();
    let r = e.execute("SELECT * FROM t").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 1),
        _ => panic!("expected rows"),
    }
}

#[test]
fn test_select_order_by_desc() {
    let d = TempDir::new().unwrap();
    let e = make_executor(d.path());
    e.execute("CREATE TABLE t (id u64, val u64)").unwrap();
    for i in 1u64..=5 {
        e.execute(&format!("INSERT INTO t (id, val) VALUES ({i}, {i})"))
            .unwrap();
    }
    let r = e.execute("SELECT * FROM t ORDER BY id DESC").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => {
            assert_eq!(rows[0][0], "5");
        }
        _ => panic!("expected rows"),
    }
}

#[test]
fn test_select_limit() {
    let d = TempDir::new().unwrap();
    let e = make_executor(d.path());
    e.execute("CREATE TABLE t (id u64, val u64)").unwrap();
    for i in 1u64..=10 {
        e.execute(&format!("INSERT INTO t (id, val) VALUES ({i}, {i})"))
            .unwrap();
    }
    let r = e.execute("SELECT * FROM t LIMIT 3").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 3),
        _ => panic!("expected rows"),
    }
}

#[test]
fn test_select_named_columns() {
    let d = TempDir::new().unwrap();
    let e = make_executor(d.path());
    e.execute("CREATE TABLE t (id u64, name u64, age u64)")
        .unwrap();
    e.execute("INSERT INTO t (id, name, age) VALUES (1, 42, 30)")
        .unwrap();
    let r = e.execute("SELECT id, age FROM t").unwrap();
    match r {
        QueryResult::Rows { columns, rows } => {
            assert_eq!(columns, vec!["id", "age"]);
            assert_eq!(rows[0].len(), 2);
        }
        _ => panic!("expected rows"),
    }
}

#[test]
fn test_insert_affected_count() {
    let d = TempDir::new().unwrap();
    let e = make_executor(d.path());
    e.execute("CREATE TABLE t (id u64)").unwrap();
    let r = e.execute("INSERT INTO t (id) VALUES (1)").unwrap();
    assert!(matches!(r, QueryResult::Affected(1)));
}

#[test]
fn test_select_from_unknown_table() {
    let d = TempDir::new().unwrap();
    let e = make_executor(d.path());
    assert!(e.execute("SELECT * FROM nosuchtable").is_err());
}

// -- Vector ------------------------------------------------------------------

#[test]
fn test_vector_insert_and_get() {
    let d = TempDir::new().unwrap();
    let registry = TableRegistry::open(d.path(), EngineOptions::default()).unwrap();
    let registry = Arc::new(registry);
    let e = SqlExecutor::new(Arc::clone(&registry));
    e.execute("CREATE TABLE items (id u64, emb vector(3))")
        .unwrap();
    let engine = registry.get("items").unwrap();
    engine.insert_vector(1, 1, vec![1.0_f32, 0.0, 0.0]).unwrap();
    let got = engine.get_vector(1, 1).unwrap();
    assert_eq!(got.len(), 3);
    assert!((got[0] - 1.0).abs() < 1e-6);
}

#[test]
fn test_vector_search_nearest() {
    let d = TempDir::new().unwrap();
    let registry = TableRegistry::open(d.path(), EngineOptions::default()).unwrap();
    let registry = Arc::new(registry);
    let e = SqlExecutor::new(Arc::clone(&registry));
    e.execute("CREATE TABLE items (id u64, emb vector(2))")
        .unwrap();
    let engine = registry.get("items").unwrap();
    engine.insert_vector(1, 1, vec![1.0_f32, 0.0]).unwrap();
    engine.insert_vector(1, 2, vec![0.0_f32, 1.0]).unwrap();
    engine.insert_vector(1, 3, vec![-1.0_f32, 0.0]).unwrap();
    let results = engine.search_vector(1, &[1.0_f32, 0.0], 1).unwrap();
    assert_eq!(results[0].0, 1);
}

#[test]
fn test_vector_get_missing_returns_none() {
    let d = TempDir::new().unwrap();
    let registry = TableRegistry::open(d.path(), EngineOptions::default()).unwrap();
    let registry = Arc::new(registry);
    let e = SqlExecutor::new(Arc::clone(&registry));
    e.execute("CREATE TABLE items (id u64, emb vector(3))")
        .unwrap();
    let engine = registry.get("items").unwrap();
    assert!(engine.get_vector(1, 999).is_none());
}
