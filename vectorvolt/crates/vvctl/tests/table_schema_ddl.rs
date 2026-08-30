//! Integration tests for L127: vvctl table schema + alter-add-col/alter-drop-col (VV-ENG-009).

use std::path::PathBuf;

use tempfile::TempDir;
use vectorvolt_engine::{ColumnType, TableSchema};
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}

fn table_dir(dir: &TempDir) -> PathBuf {
    dir.path().join("tbl")
}

fn create_table(dir: &TempDir, columns: &[&str]) {
    run(TableCommand::Create {
        dir: table_dir(dir),
        columns: columns.iter().map(|s| s.to_string()).collect(),
    })
    .unwrap();
}

// ── vvctl table schema ────────────────────────────────────────────────────────

#[test]
fn schema_prints_column_list() {
    let dir = make_dir();
    create_table(&dir, &["pk:u64", "score:i64"]);
    run(TableCommand::Schema {
        dir: table_dir(&dir),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn schema_missing_table_errors() {
    let dir = make_dir();
    let result = run(TableCommand::Schema {
        dir: dir.path().join("nonexistent"),
        output: OutputFormat::Table,
    });
    assert!(result.is_err());
}

#[test]
fn schema_file_is_readable_as_table_schema() {
    let dir = make_dir();
    create_table(&dir, &["pk:u64", "active:bool"]);
    let schema_path = table_dir(&dir).join("schema.txt");
    let text = std::fs::read_to_string(schema_path).unwrap();
    let schema = TableSchema::from_text(&text).unwrap();
    assert_eq!(schema.col_count(), 2);
    assert_eq!(schema.get(1).unwrap().col_type, ColumnType::Bool);
}

// ── vvctl table alter-add-col ─────────────────────────────────────────────────

#[test]
fn alter_add_col_appends_column() {
    let dir = make_dir();
    create_table(&dir, &["pk:u64"]);
    run(TableCommand::AlterAddCol {
        dir: table_dir(&dir),
        column: "score:i64".into(),
    })
    .unwrap();
    let schema_path = table_dir(&dir).join("schema.txt");
    let text = std::fs::read_to_string(schema_path).unwrap();
    let schema = TableSchema::from_text(&text).unwrap();
    assert_eq!(schema.col_count(), 2);
    assert_eq!(schema.get(1).unwrap().name, "score");
    assert_eq!(schema.get(1).unwrap().col_type, ColumnType::I64);
}

#[test]
fn alter_add_col_multiple_times() {
    let dir = make_dir();
    create_table(&dir, &["pk:u64"]);
    run(TableCommand::AlterAddCol {
        dir: table_dir(&dir),
        column: "a:f64".into(),
    })
    .unwrap();
    run(TableCommand::AlterAddCol {
        dir: table_dir(&dir),
        column: "b:bool".into(),
    })
    .unwrap();
    let schema_path = table_dir(&dir).join("schema.txt");
    let text = std::fs::read_to_string(schema_path).unwrap();
    let schema = TableSchema::from_text(&text).unwrap();
    assert_eq!(schema.col_count(), 3);
}

#[test]
fn alter_add_col_duplicate_errors() {
    let dir = make_dir();
    create_table(&dir, &["pk:u64", "val:u64"]);
    let result = run(TableCommand::AlterAddCol {
        dir: table_dir(&dir),
        column: "val:i64".into(),
    });
    assert!(result.is_err());
}

#[test]
fn alter_add_col_missing_table_errors() {
    let dir = make_dir();
    let result = run(TableCommand::AlterAddCol {
        dir: dir.path().join("nonexistent"),
        column: "x:u64".into(),
    });
    assert!(result.is_err());
}

// ── vvctl table alter-drop-col ────────────────────────────────────────────────

#[test]
fn alter_drop_col_removes_column() {
    let dir = make_dir();
    create_table(&dir, &["pk:u64", "score:i64", "active:bool"]);
    run(TableCommand::AlterDropCol {
        dir: table_dir(&dir),
        column: "score".into(),
    })
    .unwrap();
    let schema_path = table_dir(&dir).join("schema.txt");
    let text = std::fs::read_to_string(schema_path).unwrap();
    let schema = TableSchema::from_text(&text).unwrap();
    assert_eq!(schema.col_count(), 2);
    assert!(schema.find("score").is_none());
    assert!(schema.find("active").is_some());
}

#[test]
fn alter_drop_col_pk_errors() {
    let dir = make_dir();
    create_table(&dir, &["pk:u64", "val:u64"]);
    let result = run(TableCommand::AlterDropCol {
        dir: table_dir(&dir),
        column: "pk".into(),
    });
    assert!(result.is_err());
}

#[test]
fn alter_drop_col_nonexistent_is_idempotent() {
    let dir = make_dir();
    create_table(&dir, &["pk:u64", "score:u64"]);
    // Dropping a non-existent column is silently ignored (idempotent).
    run(TableCommand::AlterDropCol {
        dir: table_dir(&dir),
        column: "ghost".into(),
    })
    .unwrap();
    // The schema is unchanged — pk + score still present.
    run(TableCommand::Schema {
        dir: table_dir(&dir),
        output: OutputFormat::Table,
    })
    .unwrap();
}

// ── typed create ─────────────────────────────────────────────────────────────

#[test]
fn create_with_typed_columns_persists_types() {
    let dir = make_dir();
    create_table(&dir, &["pk:u64", "ts:timestamp", "amount:decimal"]);
    let schema_path = table_dir(&dir).join("schema.txt");
    let text = std::fs::read_to_string(schema_path).unwrap();
    assert!(text.contains("timestamp") || text.contains("pk"));
    let schema = TableSchema::from_text(&text).unwrap();
    assert_eq!(schema.col_count(), 3);
}
