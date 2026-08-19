//! Layer 80 — Column type system tests (VV-ENG-008).

use vectorvolt_engine::schema::{ColumnDef, ColumnType, TableSchema};
use vectorvolt_engine::{EngineError, EngineOptions, TableRegistry};

// ── ColumnType ────────────────────────────────────────────────────────────────

#[test]
fn col_type_from_str_all_variants() {
    assert_eq!(ColumnType::from_token("u64"), ColumnType::U64);
    assert_eq!(ColumnType::from_token("i64"), ColumnType::I64);
    assert_eq!(ColumnType::from_token("f64"), ColumnType::F64);
    assert_eq!(ColumnType::from_token("bool"), ColumnType::Bool);
    assert_eq!(ColumnType::from_token("timestamp"), ColumnType::Timestamp);
    assert_eq!(ColumnType::from_token("decimal"), ColumnType::Decimal);
}

// ── TableSchema construction ──────────────────────────────────────────────────

#[test]
fn schema_col_count_matches_definitions() {
    let s = TableSchema::new(vec![
        ColumnDef::u64("pk"),
        ColumnDef::new("price", ColumnType::Decimal),
        ColumnDef::new("active", ColumnType::Bool),
    ])
    .unwrap();
    assert_eq!(s.col_count(), 3);
}

#[test]
fn schema_find_col_by_name_hit() {
    let s = TableSchema::new(vec![
        ColumnDef::u64("pk"),
        ColumnDef::new("qty", ColumnType::I64),
    ])
    .unwrap();
    let (idx, def) = s.find("qty").expect("must find qty");
    assert_eq!(idx, 1);
    assert_eq!(def.col_type, ColumnType::I64);
}

#[test]
fn schema_find_col_by_name_miss_returns_none() {
    let s = TableSchema::new(vec![ColumnDef::u64("pk")]).unwrap();
    assert!(s.find("missing").is_none());
}

#[test]
fn schema_get_by_index_in_bounds() {
    let s = TableSchema::new(vec![
        ColumnDef::u64("pk"),
        ColumnDef::new("ts", ColumnType::Timestamp),
    ])
    .unwrap();
    assert_eq!(s.get(1).unwrap().col_type, ColumnType::Timestamp);
}

// ── Row validation ────────────────────────────────────────────────────────────

#[test]
fn schema_validate_row_correct_length_passes() {
    let s = TableSchema::new(vec![
        ColumnDef::u64("pk"),
        ColumnDef::new("active", ColumnType::Bool),
    ])
    .unwrap();
    assert!(s.validate_row(&[1, 0]).is_ok());
    assert!(s.validate_row(&[2, 1]).is_ok());
}

#[test]
fn schema_validate_row_wrong_length_errors() {
    let s = TableSchema::new(vec![ColumnDef::u64("pk"), ColumnDef::u64("val")]).unwrap();
    assert!(s.validate_row(&[1]).is_err());
    assert!(s.validate_row(&[1, 2, 3]).is_err());
}

#[test]
fn schema_bool_col_rejects_value_gt_1() {
    let s = TableSchema::new(vec![
        ColumnDef::u64("pk"),
        ColumnDef::new("flag", ColumnType::Bool),
    ])
    .unwrap();
    let err = s.validate_row(&[1, 2]).unwrap_err();
    assert!(matches!(
        err,
        EngineError::SchemaMismatch {
            col_idx: 1,
            col_type: "bool",
            value: 2
        }
    ));
}

#[test]
fn schema_bool_col_accepts_0_and_1() {
    let s = TableSchema::new(vec![
        ColumnDef::u64("pk"),
        ColumnDef::new("flag", ColumnType::Bool),
    ])
    .unwrap();
    assert!(s.validate_row(&[1, 0]).is_ok());
    assert!(s.validate_row(&[1, 1]).is_ok());
}

// ── Serialization ─────────────────────────────────────────────────────────────

#[test]
fn schema_roundtrip_to_text_and_back() {
    let original = TableSchema::new(vec![
        ColumnDef::u64("pk"),
        ColumnDef::new("score", ColumnType::F64),
        ColumnDef::new("active", ColumnType::Bool),
        ColumnDef::new("created_at", ColumnType::Timestamp),
    ])
    .unwrap();
    let text = original.to_text();
    let restored = TableSchema::from_text(&text).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn schema_from_names_produces_all_u64() {
    let names = vec!["pk".into(), "val".into(), "qty".into()];
    let s = TableSchema::from_names(&names);
    for i in 0..3 {
        assert_eq!(s.get(i).unwrap().col_type, ColumnType::U64);
    }
    assert_eq!(s.names(), names);
}

// ── Invariants ────────────────────────────────────────────────────────────────

#[test]
fn schema_pk_col_is_always_index_zero() {
    let s = TableSchema::new(vec![
        ColumnDef::new("id", ColumnType::U64),
        ColumnDef::new("name", ColumnType::U64),
    ])
    .unwrap();
    assert_eq!(s.get(0).unwrap().name, "id");
    assert_eq!(s.pk_type(), ColumnType::U64);
}

#[test]
fn schema_duplicate_col_name_is_detected() {
    let err = TableSchema::new(vec![ColumnDef::u64("pk"), ColumnDef::u64("pk")]).unwrap_err();
    assert!(matches!(err, EngineError::SchemaMismatch { .. }));
}

// ── Registry integration ──────────────────────────────────────────────────────

#[test]
fn registry_create_table_typed_and_read_schema() {
    let dir = tempfile::tempdir().unwrap();
    let reg = TableRegistry::new(dir.path(), EngineOptions::default());

    let schema = TableSchema::new(vec![
        ColumnDef::u64("pk"),
        ColumnDef::new("price", ColumnType::Decimal),
        ColumnDef::new("active", ColumnType::Bool),
    ])
    .unwrap();
    reg.create_table_typed("orders", schema.clone()).unwrap();

    let read_back = reg.read_schema("orders").unwrap();
    assert_eq!(read_back, schema);
}

#[test]
fn registry_schema_persisted_across_reopen() {
    let dir = tempfile::tempdir().unwrap();

    let schema = TableSchema::new(vec![
        ColumnDef::u64("pk"),
        ColumnDef::new("ts", ColumnType::Timestamp),
    ])
    .unwrap();
    {
        let reg = TableRegistry::new(dir.path(), EngineOptions::default());
        reg.create_table_typed("events", schema.clone()).unwrap();
    }

    let reg2 = TableRegistry::open(dir.path(), EngineOptions::default()).unwrap();
    let read_back = reg2.read_schema("events").unwrap();
    assert_eq!(read_back, schema);
}
