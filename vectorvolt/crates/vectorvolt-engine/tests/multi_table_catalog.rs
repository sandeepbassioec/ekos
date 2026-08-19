//! Layer 88 — multi-table catalog integration tests (VV-ENG-018).
//!
//! Covers TableRegistry create / get / drop / list / contains / table_count,
//! typed-schema round-trips, persistence via open(), and error cases.

use tempfile::TempDir;
use vectorvolt_engine::schema::{ColumnDef, ColumnType, TableSchema};
use vectorvolt_engine::{Durability, EngineOptions, TableRegistry};

fn opts() -> EngineOptions {
    EngineOptions::default().with_durability(Durability::WriteOnly)
}

fn schema_names(cols: &[&str]) -> Vec<String> {
    cols.iter().map(|s| s.to_string()).collect()
}

// ── create / contains / table_count ──────────────────────────────────────────

#[test]
fn empty_registry_has_zero_tables() {
    let dir = TempDir::new().unwrap();
    let reg = TableRegistry::new(dir.path(), opts());
    assert_eq!(reg.table_count(), 0);
}

#[test]
fn create_table_increments_count() {
    let dir = TempDir::new().unwrap();
    let reg = TableRegistry::new(dir.path(), opts());
    reg.create_table("orders", schema_names(&["id", "amount"]))
        .unwrap();
    assert_eq!(reg.table_count(), 1);
    reg.create_table("users", schema_names(&["id", "score"]))
        .unwrap();
    assert_eq!(reg.table_count(), 2);
}

#[test]
fn contains_returns_true_after_create() {
    let dir = TempDir::new().unwrap();
    let reg = TableRegistry::new(dir.path(), opts());
    assert!(!reg.contains("products"));
    reg.create_table("products", schema_names(&["id", "price"]))
        .unwrap();
    assert!(reg.contains("products"));
}

#[test]
fn contains_returns_false_for_unknown_table() {
    let dir = TempDir::new().unwrap();
    let reg = TableRegistry::new(dir.path(), opts());
    assert!(!reg.contains("nonexistent"));
}

#[test]
fn duplicate_create_returns_error() {
    let dir = TempDir::new().unwrap();
    let reg = TableRegistry::new(dir.path(), opts());
    reg.create_table("foo", schema_names(&["x"])).unwrap();
    assert!(reg.create_table("foo", schema_names(&["y"])).is_err());
}

// ── get / list ────────────────────────────────────────────────────────────────

#[test]
fn get_returns_engine_for_existing_table() {
    let dir = TempDir::new().unwrap();
    let reg = TableRegistry::new(dir.path(), opts());
    reg.create_table("metrics", schema_names(&["ts", "val"]))
        .unwrap();
    let engine = reg.get("metrics").unwrap();
    // Engine opened cleanly — snapshot_info returns Ok (None = no partitions yet).
    let _ = engine.snapshot_info().unwrap();
}

#[test]
fn get_returns_error_for_unknown_table() {
    let dir = TempDir::new().unwrap();
    let reg = TableRegistry::new(dir.path(), opts());
    assert!(reg.get("ghost").is_err());
}

#[test]
fn list_tables_returns_sorted_names() {
    let dir = TempDir::new().unwrap();
    let reg = TableRegistry::new(dir.path(), opts());
    reg.create_table("zebra", schema_names(&["id"])).unwrap();
    reg.create_table("apple", schema_names(&["id"])).unwrap();
    reg.create_table("mango", schema_names(&["id"])).unwrap();
    assert_eq!(reg.list_tables(), vec!["apple", "mango", "zebra"]);
}

// ── drop ─────────────────────────────────────────────────────────────────────

#[test]
fn drop_table_removes_from_registry() {
    let dir = TempDir::new().unwrap();
    let reg = TableRegistry::new(dir.path(), opts());
    reg.create_table("tmp", schema_names(&["id"])).unwrap();
    assert!(reg.contains("tmp"));
    reg.drop_table("tmp").unwrap();
    assert!(!reg.contains("tmp"));
    assert_eq!(reg.table_count(), 0);
}

#[test]
fn drop_nonexistent_returns_error() {
    let dir = TempDir::new().unwrap();
    let reg = TableRegistry::new(dir.path(), opts());
    assert!(reg.drop_table("ghost").is_err());
}

// ── typed schema ──────────────────────────────────────────────────────────────

#[test]
fn create_typed_table_roundtrips_column_types() {
    let dir = TempDir::new().unwrap();
    let reg = TableRegistry::new(dir.path(), opts());

    let schema = TableSchema::new(vec![
        ColumnDef::new("ts", ColumnType::Timestamp),
        ColumnDef::new("price", ColumnType::F64),
        ColumnDef::new("qty", ColumnType::U64),
    ])
    .unwrap();
    reg.create_table_typed("trades", schema).unwrap();

    let read_back = reg.read_schema("trades").unwrap();
    assert_eq!(read_back.col_count(), 3);
    assert_eq!(read_back.get(1).unwrap().col_type, ColumnType::F64);
}

// ── persistence (open) ────────────────────────────────────────────────────────

#[test]
fn open_restores_persisted_tables() {
    let dir = TempDir::new().unwrap();
    {
        let reg = TableRegistry::new(dir.path(), opts());
        reg.create_table("accounts", schema_names(&["id", "balance"]))
            .unwrap();
        reg.create_table("sessions", schema_names(&["id", "token"]))
            .unwrap();
    }
    let reg2 = TableRegistry::open(dir.path(), opts()).unwrap();
    assert_eq!(reg2.table_count(), 2);
    assert!(reg2.contains("accounts"));
    assert!(reg2.contains("sessions"));
}

#[test]
fn open_skips_dirs_without_schema_file() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir(dir.path().join("orphan")).unwrap();
    let reg = TableRegistry::open(dir.path(), opts()).unwrap();
    assert_eq!(reg.table_count(), 0);
}

#[test]
fn drop_then_open_does_not_restore_dropped_table() {
    let dir = TempDir::new().unwrap();
    {
        let reg = TableRegistry::new(dir.path(), opts());
        reg.create_table("temp", schema_names(&["id"])).unwrap();
        reg.create_table("perm", schema_names(&["id"])).unwrap();
        reg.drop_table("temp").unwrap();
    }
    let reg2 = TableRegistry::open(dir.path(), opts()).unwrap();
    assert!(!reg2.contains("temp"));
    assert!(reg2.contains("perm"));
    assert_eq!(reg2.table_count(), 1);
}

// ── invalid names ─────────────────────────────────────────────────────────────

#[test]
fn create_with_empty_name_returns_error() {
    let dir = TempDir::new().unwrap();
    let reg = TableRegistry::new(dir.path(), opts());
    assert!(reg.create_table("", schema_names(&["id"])).is_err());
}

#[test]
fn create_with_invalid_chars_returns_error() {
    let dir = TempDir::new().unwrap();
    let reg = TableRegistry::new(dir.path(), opts());
    assert!(
        reg.create_table("bad name!", schema_names(&["id"]))
            .is_err()
    );
}
