//! Layer 92 — ALTER TABLE: add / drop columns (VV-ENG-009).
//!
//! Tests for `TableRegistry::alter_table`, `TableSchema::with_added`,
//! `TableSchema::with_dropped`, and `Mutation::DdlAlter` encode/decode.

use tempfile::TempDir;
use vectorvolt_engine::{
    EngineOptions, TableRegistry,
    schema::{ColumnDef, ColumnType, TableSchema},
};
use vectorvolt_txn::{ops::Mutation, ops::OP_DDL_ALTER};
use vectorvolt_wal::{WalConfig, WalReader};

// ── helpers ───────────────────────────────────────────────────────────────────

fn two_col_schema() -> TableSchema {
    TableSchema::new(vec![ColumnDef::u64("pk"), ColumnDef::u64("val")]).unwrap()
}

fn make_registry() -> (TableRegistry, TempDir) {
    let dir = TempDir::new().unwrap();
    let reg = TableRegistry::new(dir.path(), EngineOptions::default());
    (reg, dir)
}

// ── Mutation::DdlAlter codec ──────────────────────────────────────────────────

#[test]
fn ddl_alter_opcode_value_is_0x12() {
    assert_eq!(OP_DDL_ALTER, 0x12);
}

#[test]
fn ddl_alter_encode_decode_round_trip() {
    let m = Mutation::DdlAlter {
        table_name: "orders".into(),
        add_columns: vec!["status".into(), "total".into()],
        drop_columns: vec!["old_col".into()],
    };
    assert_eq!(Mutation::decode(&m.encode()).unwrap(), m);
}

#[test]
fn ddl_alter_empty_lists_round_trip() {
    let m = Mutation::DdlAlter {
        table_name: "t".into(),
        add_columns: vec![],
        drop_columns: vec![],
    };
    assert_eq!(Mutation::decode(&m.encode()).unwrap(), m);
}

#[test]
fn ddl_alter_pk_returns_zero() {
    let m = Mutation::DdlAlter {
        table_name: "t".into(),
        add_columns: vec![],
        drop_columns: vec![],
    };
    assert_eq!(m.pk(), 0);
}

// ── TableSchema helpers ───────────────────────────────────────────────────────

#[test]
fn schema_with_added_appends_columns() {
    let s = two_col_schema();
    let s2 = s
        .with_added(&[ColumnDef::new("score", ColumnType::F64)])
        .unwrap();
    assert_eq!(s2.col_count(), 3);
    assert_eq!(s2.get(2).unwrap().name, "score");
    assert_eq!(s2.get(2).unwrap().col_type, ColumnType::F64);
}

#[test]
fn schema_with_added_rejects_duplicate_name() {
    let s = two_col_schema();
    assert!(s.with_added(&[ColumnDef::u64("val")]).is_err());
}

#[test]
fn schema_with_dropped_removes_column() {
    let s = two_col_schema();
    let s2 = s.with_dropped(&["val"]).unwrap();
    assert_eq!(s2.col_count(), 1);
    assert_eq!(s2.get(0).unwrap().name, "pk");
}

#[test]
fn schema_with_dropped_ignores_unknown_name() {
    let s = two_col_schema();
    let s2 = s.with_dropped(&["nonexistent"]).unwrap();
    assert_eq!(s2.col_count(), 2);
}

#[test]
fn schema_with_dropped_errors_if_all_removed() {
    let s = two_col_schema();
    assert!(s.with_dropped(&["pk", "val"]).is_err());
}

// ── TableRegistry::alter_table ────────────────────────────────────────────────

#[test]
fn alter_table_add_column_updates_schema() {
    let (reg, _dir) = make_registry();
    reg.create_table("t", vec!["pk".into(), "val".into()])
        .unwrap();
    let engine = reg
        .alter_table("t", &[ColumnDef::new("score", ColumnType::I64)], &[])
        .unwrap();
    assert_eq!(engine.typed_schema().col_count(), 3);
    assert_eq!(engine.typed_schema().get(2).unwrap().name, "score");
}

#[test]
fn alter_table_drop_column_updates_schema() {
    let (reg, _dir) = make_registry();
    reg.create_table("t", vec!["pk".into(), "val".into(), "extra".into()])
        .unwrap();
    let engine = reg.alter_table("t", &[], &["extra"]).unwrap();
    assert_eq!(engine.typed_schema().col_count(), 2);
    assert!(engine.typed_schema().find("extra").is_none());
}

#[test]
fn alter_table_add_and_drop_simultaneously() {
    let (reg, _dir) = make_registry();
    reg.create_table("t", vec!["pk".into(), "old".into()])
        .unwrap();
    let engine = reg
        .alter_table("t", &[ColumnDef::u64("new_col")], &["old"])
        .unwrap();
    assert_eq!(engine.typed_schema().col_count(), 2);
    assert!(engine.typed_schema().find("new_col").is_some());
    assert!(engine.typed_schema().find("old").is_none());
}

#[test]
fn alter_table_not_found_returns_error() {
    let (reg, _dir) = make_registry();
    assert!(reg.alter_table("ghost", &[], &[]).is_err());
}

#[test]
fn alter_table_schema_persists_across_open() {
    let dir = TempDir::new().unwrap();
    {
        let reg = TableRegistry::new(dir.path(), EngineOptions::default());
        reg.create_table("t", vec!["pk".into(), "val".into()])
            .unwrap();
        reg.alter_table("t", &[ColumnDef::u64("extra")], &[])
            .unwrap();
    }
    // Re-open registry and verify schema was persisted.
    let reg2 = TableRegistry::open(dir.path(), EngineOptions::default()).unwrap();
    let engine = reg2.get("t").unwrap();
    assert_eq!(engine.typed_schema().col_count(), 3);
    assert!(engine.typed_schema().find("extra").is_some());
}

#[test]
fn alter_table_logs_ddl_alter_to_catalog_wal() {
    let dir = TempDir::new().unwrap();
    let wal_dir = TempDir::new().unwrap();
    {
        let mut reg = TableRegistry::new(dir.path(), EngineOptions::default());
        reg.enable_catalog_wal(wal_dir.path()).unwrap();
        reg.create_table("t", vec!["pk".into(), "val".into()])
            .unwrap();
        reg.alter_table("t", &[ColumnDef::u64("extra")], &[])
            .unwrap();
    }
    let cfg = WalConfig::default()
        .with_dir(wal_dir.path())
        .with_fsync(false);
    let reader = WalReader::open(cfg);
    let mutations: Vec<Mutation> = reader
        .read_all()
        .unwrap()
        .into_iter()
        .filter_map(|f| Mutation::decode(&f.payload).ok())
        .collect();
    // First mutation: DdlCreate; second: DdlAlter.
    assert_eq!(mutations.len(), 2);
    assert!(matches!(mutations[0], Mutation::DdlCreate { .. }));
    assert!(matches!(
        &mutations[1],
        Mutation::DdlAlter {
            table_name,
            add_columns,
            ..
        } if table_name == "t" && add_columns == &["extra"]
    ));
}
