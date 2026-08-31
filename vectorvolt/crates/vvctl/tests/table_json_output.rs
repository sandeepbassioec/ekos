//! Integration tests for L131: --output json flag on vvctl table read commands (VV-CTL-016).

use std::path::PathBuf;

use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}

fn table_dir(dir: &TempDir) -> PathBuf {
    dir.path().join("tbl")
}

fn create_and_insert(dir: &TempDir, cols: &[&str], pk: u64, vals: &[u64]) {
    run(TableCommand::Create {
        dir: table_dir(dir),
        columns: cols.iter().map(|s| s.to_string()).collect(),
    })
    .unwrap();
    let mut values = vec![pk];
    values.extend_from_slice(vals);
    run(TableCommand::Insert {
        dir: table_dir(dir),
        pk,
        values: vals.to_vec(),
    })
    .unwrap();
    let _ = values;
}

// â”€â”€ get --output json â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn get_json_existing_row_succeeds() {
    let dir = make_dir();
    create_and_insert(&dir, &["pk:u64", "count:u64"], 1, &[42]);
    run(TableCommand::Get {
        dir: table_dir(&dir),
        pk: 1,
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn get_json_missing_row_prints_null() {
    let dir = make_dir();
    run(TableCommand::Create {
        dir: table_dir(&dir),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::Get {
        dir: table_dir(&dir),
        pk: 99,
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn get_json_bool_column_succeeds() {
    let dir = make_dir();
    create_and_insert(&dir, &["pk:u64", "active:bool"], 5, &[1]);
    run(TableCommand::Get {
        dir: table_dir(&dir),
        pk: 5,
        output: OutputFormat::Json,
    })
    .unwrap();
}

// â”€â”€ scan --output json â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn scan_json_empty_result_is_array() {
    let dir = make_dir();
    run(TableCommand::Create {
        dir: table_dir(&dir),
        columns: vec!["pk:u64".into(), "v:u64".into()],
    })
    .unwrap();
    run(TableCommand::Flush {
        dir: table_dir(&dir),
    })
    .unwrap();
    run(TableCommand::Scan {
        dir: table_dir(&dir),
        col: 0,
        lo: 0,
        hi: u64::MAX,
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn scan_json_with_rows_succeeds() {
    let dir = make_dir();
    create_and_insert(&dir, &["pk:u64", "score:i64"], 10, &[99]);
    run(TableCommand::Flush {
        dir: table_dir(&dir),
    })
    .unwrap();
    run(TableCommand::Scan {
        dir: table_dir(&dir),
        col: 0,
        lo: 0,
        hi: u64::MAX,
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn scan_json_typed_decimal_succeeds() {
    let dir = make_dir();
    create_and_insert(&dir, &["pk:u64", "amount:decimal"], 3, &[2_750_000u64]);
    run(TableCommand::Flush {
        dir: table_dir(&dir),
    })
    .unwrap();
    run(TableCommand::Scan {
        dir: table_dir(&dir),
        col: 0,
        lo: 0,
        hi: u64::MAX,
        output: OutputFormat::Json,
    })
    .unwrap();
}

// â”€â”€ stats --output json â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn stats_json_succeeds() {
    let dir = make_dir();
    create_and_insert(&dir, &["pk:u64", "v:u64"], 1, &[1]);
    run(TableCommand::Stats {
        dir: table_dir(&dir),
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn stats_table_still_works() {
    let dir = make_dir();
    run(TableCommand::Create {
        dir: table_dir(&dir),
        columns: vec!["pk:u64".into()],
    })
    .unwrap();
    run(TableCommand::Stats {
        dir: table_dir(&dir),
        output: OutputFormat::Table,
    })
    .unwrap();
}

// â”€â”€ schema --output json â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn schema_json_succeeds() {
    let dir = make_dir();
    run(TableCommand::Create {
        dir: table_dir(&dir),
        columns: vec!["pk:u64".into(), "ts:timestamp".into(), "active:bool".into()],
    })
    .unwrap();
    run(TableCommand::Schema {
        dir: table_dir(&dir),
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn schema_json_pk_column_only() {
    let dir = make_dir();
    run(TableCommand::Create {
        dir: table_dir(&dir),
        columns: vec!["id:u64".into()],
    })
    .unwrap();
    run(TableCommand::Schema {
        dir: table_dir(&dir),
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn get_table_format_still_works() {
    let dir = make_dir();
    create_and_insert(&dir, &["pk:u64", "v:u64"], 7, &[77]);
    run(TableCommand::Get {
        dir: table_dir(&dir),
        pk: 7,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn scan_json_multi_column_row() {
    let dir = make_dir();
    create_and_insert(
        &dir,
        &["pk:u64", "active:bool", "score:i64"],
        1,
        &[1, u64::MAX],
    );
    run(TableCommand::Flush {
        dir: table_dir(&dir),
    })
    .unwrap();
    run(TableCommand::Scan {
        dir: table_dir(&dir),
        col: 0,
        lo: 0,
        hi: u64::MAX,
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn stats_json_after_flush() {
    let dir = make_dir();
    create_and_insert(&dir, &["pk:u64", "x:u64"], 1, &[10]);
    run(TableCommand::Flush {
        dir: table_dir(&dir),
    })
    .unwrap();
    run(TableCommand::Stats {
        dir: table_dir(&dir),
        output: OutputFormat::Json,
    })
    .unwrap();
}
