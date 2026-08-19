//! Layer 81 — TableSchema wired into TableEngine write path (VV-ENG-008).
//!
//! Verifies that insert validates column types, typed_schema() returns the
//! live schema, and all other engine operations remain unaffected.

use vectorvolt_engine::schema::{ColumnDef, ColumnType, TableSchema};
use vectorvolt_engine::{EngineError, EngineOptions, TableEngine, TableRegistry};

fn buffered_opts() -> EngineOptions {
    EngineOptions::new(1_000_000).with_durability(vectorvolt_engine::Durability::Buffered)
}

fn open_engine(schema: Vec<String>) -> TableEngine {
    let dir = tempfile::tempdir().unwrap();
    #[allow(deprecated)]
    TableEngine::open_with(schema, dir.into_path(), buffered_opts()).unwrap()
}

fn typed_engine(cols: Vec<ColumnDef>) -> (TableEngine, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let schema = TableSchema::new(cols).unwrap();
    let engine = TableEngine::open_typed(schema, dir.path(), buffered_opts()).unwrap();
    (engine, dir)
}

// ── typed_schema() accessor ───────────────────────────────────────────────────

#[test]
fn typed_schema_returns_correct_col_count() {
    let e = open_engine(vec!["pk".into(), "age".into(), "score".into()]);
    assert_eq!(e.typed_schema().col_count(), 3);
}

#[test]
fn typed_schema_col_names_match_open_schema() {
    let names = vec!["pk".into(), "qty".into()];
    let e = open_engine(names.clone());
    assert_eq!(e.typed_schema().names(), names);
}

#[test]
fn typed_schema_all_cols_are_u64_for_legacy_open() {
    let e = open_engine(vec!["pk".into(), "val".into()]);
    for i in 0..2 {
        assert_eq!(e.typed_schema().get(i).unwrap().col_type, ColumnType::U64);
    }
}

// ── insert validation — length ────────────────────────────────────────────────

#[test]
fn insert_correct_length_succeeds() {
    let e = open_engine(vec!["pk".into(), "val".into()]);
    assert!(e.insert(1, vec![1, 100]).is_ok());
}

#[test]
fn insert_too_few_values_returns_col_out_of_range() {
    let e = open_engine(vec!["pk".into(), "a".into(), "b".into()]);
    let err = e.insert(1, vec![1, 2]).unwrap_err();
    assert!(matches!(err, EngineError::ColOutOfRange { .. }));
}

#[test]
fn insert_too_many_values_returns_col_out_of_range() {
    let e = open_engine(vec!["pk".into(), "a".into()]);
    let err = e.insert(1, vec![1, 2, 3]).unwrap_err();
    assert!(matches!(err, EngineError::ColOutOfRange { .. }));
}

// ── insert validation — Bool type ─────────────────────────────────────────────

#[test]
fn insert_bool_col_value_0_succeeds() {
    let (e, _dir) = typed_engine(vec![
        ColumnDef::u64("pk"),
        ColumnDef::new("active", ColumnType::Bool),
    ]);
    assert!(e.insert(1, vec![1, 0]).is_ok());
}

#[test]
fn insert_bool_col_value_1_succeeds() {
    let (e, _dir) = typed_engine(vec![
        ColumnDef::u64("pk"),
        ColumnDef::new("active", ColumnType::Bool),
    ]);
    assert!(e.insert(1, vec![1, 1]).is_ok());
}

#[test]
fn insert_bool_col_value_2_returns_schema_mismatch() {
    let (e, _dir) = typed_engine(vec![
        ColumnDef::u64("pk"),
        ColumnDef::new("active", ColumnType::Bool),
    ]);
    let err = e.insert(1, vec![1, 2]).unwrap_err();
    assert!(matches!(
        err,
        EngineError::SchemaMismatch {
            col_idx: 1,
            col_type: "bool",
            ..
        }
    ));
}

#[test]
fn insert_bool_col_large_value_returns_schema_mismatch() {
    let (e, _dir) = typed_engine(vec![
        ColumnDef::u64("pk"),
        ColumnDef::new("flag", ColumnType::Bool),
    ]);
    let err = e.insert(1, vec![1, u64::MAX]).unwrap_err();
    assert!(matches!(err, EngineError::SchemaMismatch { .. }));
}

// ── insert with non-Bool typed cols (all u64 bit-patterns valid) ──────────────

#[test]
fn insert_i64_col_accepts_any_u64() {
    let (e, _dir) = typed_engine(vec![
        ColumnDef::u64("pk"),
        ColumnDef::new("signed", ColumnType::I64),
    ]);
    assert!(e.insert(1, vec![1, u64::MAX]).is_ok());
}

#[test]
fn insert_timestamp_col_accepts_any_u64() {
    let (e, _dir) = typed_engine(vec![
        ColumnDef::u64("pk"),
        ColumnDef::new("ts", ColumnType::Timestamp),
    ]);
    assert!(e.insert(1, vec![1, 1_700_000_000_000_000_000]).is_ok());
}

// ── round-trip: insert + get with typed engine ────────────────────────────────

#[test]
fn insert_and_get_roundtrip_with_typed_schema() {
    let (e, _dir) = typed_engine(vec![
        ColumnDef::u64("pk"),
        ColumnDef::new("score", ColumnType::F64),
        ColumnDef::new("active", ColumnType::Bool),
    ]);
    e.insert(42, vec![42, f64::to_bits(2.71), 1]).unwrap();
    let row = e.get(42).unwrap().unwrap();
    assert_eq!(row.pk, 42);
    assert_eq!(row.values[2], 1);
}

// ── stats() schema field unchanged ───────────────────────────────────────────

#[test]
fn stats_schema_field_returns_col_names() {
    let e = open_engine(vec!["pk".into(), "age".into()]);
    let stats = e.stats();
    assert_eq!(stats.schema, vec!["pk", "age"]);
}

// ── typed_schema via registry ─────────────────────────────────────────────────

#[test]
fn typed_engine_schema_survives_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let schema = TableSchema::new(vec![
        ColumnDef::u64("pk"),
        ColumnDef::new("active", ColumnType::Bool),
    ])
    .unwrap();
    {
        let reg = TableRegistry::new(dir.path(), buffered_opts());
        reg.create_table_typed("t", schema.clone()).unwrap();
    }
    let reg2 = TableRegistry::open(dir.path(), buffered_opts()).unwrap();
    let restored = reg2.read_schema("t").unwrap();
    assert_eq!(restored, schema);
}
