//! Layer 90 — YAML entity schema loader tests (VV-ENG-020).

use tempfile::TempDir;
use vectorvolt_engine::schema::ColumnType;
use vectorvolt_engine::yaml_schema::{load_yaml_schema, parse_yaml_schema, yaml_def_to_schema};
use vectorvolt_engine::{Durability, EngineOptions, TableRegistry};

fn opts() -> EngineOptions {
    EngineOptions::default().with_durability(Durability::WriteOnly)
}

fn write_yaml(dir: &TempDir, filename: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(filename);
    std::fs::write(&path, content).unwrap();
    path
}

const BASIC_YAML: &str = r#"
table: orders
columns:
  - name: id
    type: u64
  - name: amount
    type: f64
  - name: ts
    type: timestamp
"#;

// ── parse_yaml_schema (in-memory) ────────────────────────────────────────────

#[test]
fn parse_valid_yaml_returns_table_name() {
    let def = parse_yaml_schema(BASIC_YAML).unwrap();
    assert_eq!(def.table, "orders");
}

#[test]
fn parse_valid_yaml_returns_column_count() {
    let def = parse_yaml_schema(BASIC_YAML).unwrap();
    assert_eq!(def.columns.len(), 3);
}

#[test]
fn parse_valid_yaml_column_names_in_order() {
    let def = parse_yaml_schema(BASIC_YAML).unwrap();
    assert_eq!(def.columns[0].name, "id");
    assert_eq!(def.columns[1].name, "amount");
    assert_eq!(def.columns[2].name, "ts");
}

#[test]
fn parse_valid_yaml_column_types() {
    let def = parse_yaml_schema(BASIC_YAML).unwrap();
    assert_eq!(def.columns[0].col_type, "u64");
    assert_eq!(def.columns[1].col_type, "f64");
    assert_eq!(def.columns[2].col_type, "timestamp");
}

#[test]
fn parse_invalid_yaml_returns_error() {
    let err = parse_yaml_schema("not: valid: yaml: [[[").unwrap_err();
    assert!(
        format!("{err}").contains("YAML"),
        "error should mention YAML"
    );
}

#[test]
fn parse_yaml_missing_table_field_returns_error() {
    let yaml = "columns:\n  - name: id\n    type: u64\n";
    assert!(parse_yaml_schema(yaml).is_err());
}

#[test]
fn parse_yaml_unknown_type_defaults_to_u64() {
    let yaml = "table: t\ncolumns:\n  - name: x\n    type: magic\n";
    let def = parse_yaml_schema(yaml).unwrap();
    let schema = yaml_def_to_schema(&def).unwrap();
    assert_eq!(schema.get(0).unwrap().col_type, ColumnType::U64);
}

#[test]
fn parse_yaml_missing_type_defaults_to_u64() {
    let yaml = "table: t\ncolumns:\n  - name: x\n";
    let def = parse_yaml_schema(yaml).unwrap();
    let schema = yaml_def_to_schema(&def).unwrap();
    assert_eq!(schema.get(0).unwrap().col_type, ColumnType::U64);
}

// ── yaml_def_to_schema ────────────────────────────────────────────────────────

#[test]
fn yaml_def_to_schema_maps_column_types() {
    let def = parse_yaml_schema(BASIC_YAML).unwrap();
    let schema = yaml_def_to_schema(&def).unwrap();
    assert_eq!(schema.col_count(), 3);
    assert_eq!(schema.get(0).unwrap().col_type, ColumnType::U64);
    assert_eq!(schema.get(1).unwrap().col_type, ColumnType::F64);
    assert_eq!(schema.get(2).unwrap().col_type, ColumnType::Timestamp);
}

#[test]
fn yaml_def_duplicate_column_names_returns_error() {
    let yaml = "table: t\ncolumns:\n  - name: x\n    type: u64\n  - name: x\n    type: i64\n";
    let def = parse_yaml_schema(yaml).unwrap();
    assert!(yaml_def_to_schema(&def).is_err());
}

// ── load_yaml_schema (file I/O) ───────────────────────────────────────────────

#[test]
fn load_yaml_schema_from_file() {
    let dir = TempDir::new().unwrap();
    let path = write_yaml(&dir, "orders.yaml", BASIC_YAML);
    let def = load_yaml_schema(&path).unwrap();
    assert_eq!(def.table, "orders");
    assert_eq!(def.columns.len(), 3);
}

#[test]
fn load_yaml_schema_missing_file_returns_io_error() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("does_not_exist.yaml");
    assert!(load_yaml_schema(&path).is_err());
}

// ── TableRegistry::create_from_yaml_file ─────────────────────────────────────

#[test]
fn create_from_yaml_file_registers_table() {
    let dir = TempDir::new().unwrap();
    let yaml_path = write_yaml(&dir, "orders.yaml", BASIC_YAML);
    let reg = TableRegistry::new(dir.path(), opts());
    reg.create_from_yaml_file(&yaml_path).unwrap();
    assert!(reg.contains("orders"));
}

#[test]
fn create_from_yaml_file_schema_has_correct_types() {
    let dir = TempDir::new().unwrap();
    let yaml_path = write_yaml(&dir, "orders.yaml", BASIC_YAML);
    let reg = TableRegistry::new(dir.path(), opts());
    reg.create_from_yaml_file(&yaml_path).unwrap();
    let schema = reg.read_schema("orders").unwrap();
    assert_eq!(schema.get(1).unwrap().col_type, ColumnType::F64);
}

#[test]
fn create_from_yaml_file_duplicate_returns_error() {
    let dir = TempDir::new().unwrap();
    let yaml_path = write_yaml(&dir, "orders.yaml", BASIC_YAML);
    let reg = TableRegistry::new(dir.path(), opts());
    reg.create_from_yaml_file(&yaml_path).unwrap();
    assert!(reg.create_from_yaml_file(&yaml_path).is_err());
}
