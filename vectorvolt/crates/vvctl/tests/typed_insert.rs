//! Integration tests for L130: vvctl table typed-insert (VV-ENG-008 typed write side).

use std::path::PathBuf;

use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}

fn table_dir(dir: &TempDir) -> PathBuf {
    dir.path().join("tbl")
}

fn create_table(dir: &TempDir, cols: &[&str]) {
    run(TableCommand::Create {
        dir: table_dir(dir),
        columns: cols.iter().map(|s| s.to_string()).collect(),
    })
    .unwrap();
}

// ── happy path ────────────────────────────────────────────────────────────────

#[test]
fn typed_insert_u64_only() {
    let dir = make_dir();
    create_table(&dir, &["pk:u64", "count:u64"]);
    run(TableCommand::TypedInsert {
        dir: table_dir(&dir),
        pk: "1".into(),
        values: vec!["42".into()],
    })
    .unwrap();
    run(TableCommand::Get {
        dir: table_dir(&dir),
        pk: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn typed_insert_bool_true() {
    let dir = make_dir();
    create_table(&dir, &["pk:u64", "active:bool"]);
    run(TableCommand::TypedInsert {
        dir: table_dir(&dir),
        pk: "10".into(),
        values: vec!["true".into()],
    })
    .unwrap();
    run(TableCommand::Get {
        dir: table_dir(&dir),
        pk: 10,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn typed_insert_bool_false() {
    let dir = make_dir();
    create_table(&dir, &["pk:u64", "active:bool"]);
    run(TableCommand::TypedInsert {
        dir: table_dir(&dir),
        pk: "2".into(),
        values: vec!["false".into()],
    })
    .unwrap();
}

#[test]
fn typed_insert_bool_via_numeric() {
    let dir = make_dir();
    create_table(&dir, &["pk:u64", "flag:bool"]);
    run(TableCommand::TypedInsert {
        dir: table_dir(&dir),
        pk: "3".into(),
        values: vec!["1".into()],
    })
    .unwrap();
}

#[test]
fn typed_insert_i64_negative() {
    let dir = make_dir();
    create_table(&dir, &["pk:u64", "score:i64"]);
    run(TableCommand::TypedInsert {
        dir: table_dir(&dir),
        pk: "5".into(),
        values: vec!["-99".into()],
    })
    .unwrap();
    run(TableCommand::Get {
        dir: table_dir(&dir),
        pk: 5,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn typed_insert_f64_value() {
    let dir = make_dir();
    create_table(&dir, &["pk:u64", "price:f64"]);
    run(TableCommand::TypedInsert {
        dir: table_dir(&dir),
        pk: "7".into(),
        values: vec!["1.25".into()],
    })
    .unwrap();
}

#[test]
fn typed_insert_decimal_value() {
    let dir = make_dir();
    create_table(&dir, &["pk:u64", "amount:decimal"]);
    run(TableCommand::TypedInsert {
        dir: table_dir(&dir),
        pk: "8".into(),
        values: vec!["1.500000".into()],
    })
    .unwrap();
}

#[test]
fn typed_insert_timestamp_value() {
    let dir = make_dir();
    create_table(&dir, &["pk:u64", "ts:timestamp"]);
    run(TableCommand::TypedInsert {
        dir: table_dir(&dir),
        pk: "9".into(),
        values: vec!["1700000000000000000".into()],
    })
    .unwrap();
}

#[test]
fn typed_insert_multi_column() {
    let dir = make_dir();
    create_table(
        &dir,
        &["pk:u64", "name_hash:u64", "active:bool", "score:i64"],
    );
    run(TableCommand::TypedInsert {
        dir: table_dir(&dir),
        pk: "100".into(),
        values: vec!["12345".into(), "true".into(), "-7".into()],
    })
    .unwrap();
    run(TableCommand::Get {
        dir: table_dir(&dir),
        pk: 100,
        output: OutputFormat::Table,
    })
    .unwrap();
}

// ── error cases ───────────────────────────────────────────────────────────────

#[test]
fn typed_insert_wrong_value_count() {
    let dir = make_dir();
    create_table(&dir, &["pk:u64", "a:u64", "b:u64"]);
    let result = run(TableCommand::TypedInsert {
        dir: table_dir(&dir),
        pk: "1".into(),
        values: vec!["5".into()], // should be 2
    });
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("2") && msg.contains("1"));
}

#[test]
fn typed_insert_invalid_bool() {
    let dir = make_dir();
    create_table(&dir, &["pk:u64", "flag:bool"]);
    let result = run(TableCommand::TypedInsert {
        dir: table_dir(&dir),
        pk: "1".into(),
        values: vec!["yes".into()],
    });
    assert!(result.is_err());
}

#[test]
fn typed_insert_invalid_i64() {
    let dir = make_dir();
    create_table(&dir, &["pk:u64", "score:i64"]);
    let result = run(TableCommand::TypedInsert {
        dir: table_dir(&dir),
        pk: "1".into(),
        values: vec!["abc".into()],
    });
    assert!(result.is_err());
}

#[test]
fn typed_insert_roundtrip_decimal_then_scan() {
    let dir = make_dir();
    create_table(&dir, &["pk:u64", "price:decimal"]);
    run(TableCommand::TypedInsert {
        dir: table_dir(&dir),
        pk: "20".into(),
        values: vec!["2.750000".into()],
    })
    .unwrap();
    run(TableCommand::Flush {
        dir: table_dir(&dir),
    })
    .unwrap();
    run(TableCommand::Scan {
        dir: table_dir(&dir),
        col: 0,
        lo: 20,
        hi: 20,
        output: OutputFormat::Table,
    })
    .unwrap();
}
