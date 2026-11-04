//! E1 — `SET` only accepts settings that do something. Real settings change
//! behaviour, standard PostgreSQL client-compat settings are accepted as
//! documented no-ops, and everything else fails with SQLSTATE 0A000 naming
//! the setting instead of silently succeeding.

use std::sync::Arc;
use tempfile::TempDir;
use vectorvolt_engine::{EngineOptions, TableRegistry};
use vectorvolt_query::sql::executor::{QueryResult, SqlExecutor};

fn setup() -> (SqlExecutor, TempDir) {
    let dir = TempDir::new().unwrap();
    let registry = TableRegistry::open(dir.path(), EngineOptions::default()).unwrap();
    (SqlExecutor::new(Arc::new(registry)), dir)
}

fn q(ex: &SqlExecutor, sql: &str) -> QueryResult {
    ex.execute(sql).unwrap()
}

fn assert_0a000(r: &QueryResult, name: &str) {
    match r {
        QueryResult::Error(e) => {
            assert!(e.starts_with("[0A000]"), "expected SQLSTATE 0A000, got {e}");
            assert!(e.contains(name), "error should name `{name}`: {e}");
        }
        other => panic!("expected 0A000 error for `{name}`, got {other:?}"),
    }
}

fn show_value(ex: &SqlExecutor, name: &str) -> String {
    match q(ex, &format!("SHOW {name}")) {
        QueryResult::Rows { rows, .. } => rows[0][0].clone(),
        other => panic!("expected rows, got {other:?}"),
    }
}

#[test]
fn real_setting_read_only_blocks_writes() {
    let (ex, _d) = setup();
    q(&ex, "CREATE TABLE t (pk U64, v U64)");
    q(&ex, "INSERT INTO t (pk, v) VALUES (1, 1)");
    q(&ex, "SET READ ONLY");
    let blocked = q(&ex, "INSERT INTO t (pk, v) VALUES (2, 2)");
    assert!(
        matches!(&blocked, QueryResult::Error(e) if e.contains("25006")),
        "read-only session must reject writes, got {blocked:?}"
    );
    q(&ex, "SET READ WRITE");
    assert!(matches!(
        q(&ex, "INSERT INTO t (pk, v) VALUES (2, 2)"),
        QueryResult::Affected(1)
    ));
}

#[test]
fn real_setting_statement_timeout_is_shown() {
    let (ex, _d) = setup();
    assert_eq!(show_value(&ex, "statement_timeout"), "0");
    q(&ex, "SET STATEMENT_TIMEOUT 60000");
    assert_eq!(show_value(&ex, "statement_timeout"), "60000ms");
}

#[test]
fn real_setting_search_path_is_shown() {
    let (ex, _d) = setup();
    q(&ex, "SET search_path TO analytics");
    assert_eq!(show_value(&ex, "search_path"), "analytics");
}

#[test]
fn compat_settings_are_accepted_and_echoed() {
    let (ex, _d) = setup();
    for sql in [
        "SET client_encoding = 'UTF8'",
        "SET DateStyle TO 'ISO, MDY'",
        "SET extra_float_digits = 3",
        "SET application_name = 'psql'",
        "SET standard_conforming_strings = on",
        "SET intervalstyle = 'iso_8601'",
        "SET SESSION client_min_messages TO warning",
        "SET LOCAL bytea_output = 'escape'",
        "SET TIME ZONE 'Europe/Berlin'",
        "SET NAMES 'utf8'",
        "set timezone = 'UTC';",
    ] {
        let r = q(&ex, sql);
        assert!(
            !matches!(r, QueryResult::Error(_)),
            "`{sql}` should be accepted, got {r:?}"
        );
    }
    assert_eq!(show_value(&ex, "extra_float_digits"), "3");
    assert_eq!(show_value(&ex, "application_name"), "psql");
    assert_eq!(show_value(&ex, "IntervalStyle"), "iso_8601");
    assert_eq!(show_value(&ex, "client_encoding"), "utf8");
    assert_eq!(show_value(&ex, "TimeZone"), "UTC");
    assert_eq!(show_value(&ex, "bytea_output"), "escape");
    // Defaults are visible before anything is set.
    let (ex2, _d2) = setup();
    assert_eq!(show_value(&ex2, "client_encoding"), "UTF8");
    assert_eq!(show_value(&ex2, "standard_conforming_strings"), "on");
}

#[test]
fn compat_settings_do_not_leak_across_sessions() {
    let (ex, _d) = setup();
    let other = ex.new_session();
    q(&ex, "SET application_name = 'a'");
    assert_eq!(show_value(&other, "application_name"), "");
}

#[test]
fn unimplemented_settings_fail_with_0a000() {
    let (ex, _d) = setup();
    for (sql, name) in [
        ("SET QUERY TIMEOUT 5000", "QUERY TIMEOUT"),
        ("SET POOL SIZE 20", "POOL SIZE"),
        ("SET OUTPUT FORMAT JSON", "OUTPUT FORMAT"),
        ("SET STREAMING ON", "STREAMING"),
        ("SET QUERY PRIORITY HIGH", "QUERY PRIORITY"),
        ("SET WAL LEVEL replica", "WAL LEVEL"),
        ("SET CURRENT TENANT acme", "CURRENT TENANT"),
        ("SET TABLE t STORAGE COLUMNAR", "TABLE t STORAGE COLUMNAR"),
        ("SET work_mem = '64MB'", "work_mem"),
        ("SET enable_seqscan = off", "enable_seqscan"),
        (
            "SET TRANSACTION ISOLATION LEVEL SERIALIZABLE",
            "TRANSACTION",
        ),
    ] {
        let r = q(&ex, sql);
        assert_0a000(&r, name);
    }
}

#[test]
fn rejected_setting_has_no_side_effect() {
    let (ex, _d) = setup();
    let r = q(&ex, "SET QUERY TIMEOUT 5000");
    assert_0a000(&r, "QUERY TIMEOUT");
    // The old handler would have made this report 5000.
    assert!(!matches!(
        q(&ex, "SHOW QUERY TIMEOUT"),
        QueryResult::Ok(ref v) if v == "5000"
    ));
}
