//! Layer 82 — DDL statements in query DSL (VV-QRY-011..015 + supporting cases).
//!
//! Verifies CREATE TABLE and DROP TABLE via the text DSL through
//! `run_ddl` and the corresponding `QueryResponse` variants.

use vectorvolt_engine::{EngineOptions, TableRegistry};
use vectorvolt_query::{QueryResponse, run, run_ddl};

fn registry() -> (TableRegistry, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let opts =
        EngineOptions::new(1_000_000).with_durability(vectorvolt_engine::Durability::Buffered);
    let reg = TableRegistry::new(dir.path(), opts);
    (reg, dir)
}

// ── CREATE TABLE ─────────────────────────────────────────────────────────────

#[test]
fn create_table_single_col_succeeds() {
    let (reg, _dir) = registry();
    let resp = run_ddl(&reg, "CREATE TABLE events COL ts U64").unwrap();
    assert!(matches!(resp, QueryResponse::TableCreated { name } if name == "events"));
}

#[test]
fn create_table_multi_col_succeeds() {
    let (reg, _dir) = registry();
    let resp = run_ddl(
        &reg,
        "CREATE TABLE orders COL id U64 COL qty I64 COL price F64",
    )
    .unwrap();
    assert!(matches!(resp, QueryResponse::TableCreated { name } if name == "orders"));
}

#[test]
fn create_table_bool_col_succeeds() {
    let (reg, _dir) = registry();
    let resp = run_ddl(&reg, "CREATE TABLE flags COL pk U64 COL active BOOL").unwrap();
    assert!(matches!(resp, QueryResponse::TableCreated { name } if name == "flags"));
}

#[test]
fn create_table_timestamp_col_succeeds() {
    let (reg, _dir) = registry();
    let resp = run_ddl(&reg, "CREATE TABLE events COL pk U64 COL ts TIMESTAMP").unwrap();
    assert!(matches!(resp, QueryResponse::TableCreated { name } if name == "events"));
}

#[test]
fn create_table_decimal_col_succeeds() {
    let (reg, _dir) = registry();
    let resp = run_ddl(&reg, "CREATE TABLE ledger COL pk U64 COL amount DECIMAL").unwrap();
    assert!(matches!(resp, QueryResponse::TableCreated { name } if name == "ledger"));
}

#[test]
fn create_table_duplicate_fails() {
    let (reg, _dir) = registry();
    run_ddl(&reg, "CREATE TABLE t COL pk U64").unwrap();
    let err = run_ddl(&reg, "CREATE TABLE t COL pk U64").unwrap_err();
    assert!(
        err.to_string().contains("already exists")
            || format!("{err:?}").contains("TableAlreadyExists")
    );
}

#[test]
fn create_table_no_col_clause_is_syntax_error() {
    let (reg, _dir) = registry();
    let err = run_ddl(&reg, "CREATE TABLE t").unwrap_err();
    assert!(format!("{err:?}").contains("Syntax") || err.to_string().contains("COL"));
}

#[test]
fn created_table_is_reachable_via_get() {
    let (reg, _dir) = registry();
    run_ddl(&reg, "CREATE TABLE sensor COL pk U64 COL temp U64").unwrap();
    let engine = reg.get("sensor").unwrap();
    engine.insert(1, vec![1, 42]).unwrap();
    let row = engine.get(1).unwrap().unwrap();
    assert_eq!(row.values[1], 42);
}

#[test]
fn created_table_bool_col_rejects_invalid_value() {
    let (reg, _dir) = registry();
    run_ddl(&reg, "CREATE TABLE t COL pk U64 COL flag BOOL").unwrap();
    let engine = reg.get("t").unwrap();
    let err = engine.insert(1, vec![1, 2]).unwrap_err();
    assert!(matches!(
        err,
        vectorvolt_engine::EngineError::SchemaMismatch { .. }
    ));
}

// ── DROP TABLE ────────────────────────────────────────────────────────────────

#[test]
fn drop_existing_table_succeeds() {
    let (reg, _dir) = registry();
    run_ddl(&reg, "CREATE TABLE t COL pk U64").unwrap();
    let resp = run_ddl(&reg, "DROP TABLE t").unwrap();
    assert!(matches!(resp, QueryResponse::TableDropped { name } if name == "t"));
}

#[test]
fn drop_nonexistent_table_fails() {
    let (reg, _dir) = registry();
    let err = run_ddl(&reg, "DROP TABLE ghost").unwrap_err();
    assert!(format!("{err:?}").contains("TableNotFound") || err.to_string().contains("not found"));
}

#[test]
fn dropped_table_no_longer_in_list() {
    let (reg, _dir) = registry();
    run_ddl(&reg, "CREATE TABLE t COL pk U64").unwrap();
    run_ddl(&reg, "DROP TABLE t").unwrap();
    assert!(!reg.list_tables().contains(&"t".to_string()));
}

// ── DML on DDL-created table ─────────────────────────────────────────────────

#[test]
fn run_dml_via_run_after_create() {
    let (reg, _dir) = registry();
    run_ddl(&reg, "CREATE TABLE t COL pk U64 COL score U64").unwrap();
    let engine = reg.get("t").unwrap();
    run(&engine, "INSERT 10 10 999").unwrap();
    let resp = run(&engine, "GET 10").unwrap();
    assert!(matches!(resp, QueryResponse::Row(r) if r.values[1] == 999));
}

// ── case-insensitivity ────────────────────────────────────────────────────────

#[test]
fn create_table_lowercase_keywords_succeed() {
    let (reg, _dir) = registry();
    let resp = run_ddl(&reg, "create table mytable col pk u64 col score f64").unwrap();
    assert!(matches!(resp, QueryResponse::TableCreated { name } if name == "mytable"));
}

#[test]
fn drop_table_lowercase_keywords_succeed() {
    let (reg, _dir) = registry();
    run_ddl(&reg, "create table mytable col pk u64").unwrap();
    let resp = run_ddl(&reg, "drop table mytable").unwrap();
    assert!(matches!(resp, QueryResponse::TableDropped { name } if name == "mytable"));
}
