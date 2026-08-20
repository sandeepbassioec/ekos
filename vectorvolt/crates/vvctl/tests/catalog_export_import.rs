//! Integration tests for L110: `vvctl catalog export` / `import` (VV-CTL-006).

use std::path::PathBuf;

use tempfile::TempDir;
use vvctl::commands::catalog::{CatalogArgs, CatalogCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}

fn args(dir: &TempDir, cmd: CatalogCommand) -> CatalogArgs {
    CatalogArgs {
        dir: dir.path().to_path_buf(),
        command: cmd,
    }
}

fn create_args(dir: &TempDir, name: &str) -> CatalogArgs {
    args(
        dir,
        CatalogCommand::CreateTable {
            name: name.to_string(),
            columns: vec!["pk".to_string(), "val".to_string()],
            flush_threshold: 1024,
            partitions: 1,
        },
    )
}

fn export_to_file(dir: &TempDir, out: PathBuf) -> CatalogArgs {
    args(dir, CatalogCommand::Export { output: Some(out) })
}

fn export_to_stdout(dir: &TempDir) -> CatalogArgs {
    args(dir, CatalogCommand::Export { output: None })
}

fn import_from(dir: &TempDir, file: PathBuf) -> CatalogArgs {
    args(
        dir,
        CatalogCommand::Import {
            file,
            flush_threshold: 1024,
            partitions: 1,
            skip_existing: false,
        },
    )
}

fn import_skip(dir: &TempDir, file: PathBuf) -> CatalogArgs {
    args(
        dir,
        CatalogCommand::Import {
            file,
            flush_threshold: 1024,
            partitions: 1,
            skip_existing: true,
        },
    )
}

// ── export ────────────────────────────────────────────────────────────────────

#[test]
fn export_empty_catalog_succeeds() {
    let dir = make_dir();
    let out = dir.path().join("export.json");
    run(export_to_file(&dir, out.clone())).unwrap();
    assert!(out.exists());
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("\"version\""));
    assert!(content.contains("\"tables\""));
}

#[test]
fn export_stdout_does_not_error() {
    let dir = make_dir();
    run(create_args(&dir, "items")).unwrap();
    run(export_to_stdout(&dir)).unwrap();
}

#[test]
fn export_single_table_json_has_correct_name() {
    let dir = make_dir();
    run(create_args(&dir, "orders")).unwrap();
    let out = dir.path().join("out.json");
    run(export_to_file(&dir, out.clone())).unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(
        content.contains("\"orders\""),
        "JSON should contain table name"
    );
}

#[test]
fn export_multiple_tables_all_present_in_json() {
    let dir = make_dir();
    run(create_args(&dir, "alpha")).unwrap();
    run(create_args(&dir, "beta")).unwrap();
    run(create_args(&dir, "gamma")).unwrap();
    let out = dir.path().join("out.json");
    run(export_to_file(&dir, out.clone())).unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("\"alpha\""));
    assert!(content.contains("\"beta\""));
    assert!(content.contains("\"gamma\""));
}

#[test]
fn export_json_is_valid_catalog_export() {
    use vectorvolt_engine::CatalogExport;

    let dir = make_dir();
    run(create_args(&dir, "events")).unwrap();
    let out = dir.path().join("out.json");
    run(export_to_file(&dir, out.clone())).unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    let parsed = CatalogExport::from_json(&content).unwrap();
    assert_eq!(parsed.version, 1);
    assert_eq!(parsed.tables.len(), 1);
    assert_eq!(parsed.tables[0].name, "events");
}

// ── import ────────────────────────────────────────────────────────────────────

#[test]
fn import_creates_table_in_fresh_catalog() {
    let src = make_dir();
    run(create_args(&src, "orders")).unwrap();
    let out = src.path().join("export.json");
    run(export_to_file(&src, out.clone())).unwrap();

    let dst = make_dir();
    run(import_from(&dst, out)).unwrap();

    // Verify via list-tables that the table is present.
    let reg = dst.path().join("registry");
    let registry = vectorvolt_engine::SchemaRegistry::new(reg);
    let schema = registry.load("orders").unwrap();
    assert_eq!(schema.col_count(), 2);
}

#[test]
fn import_multiple_tables_roundtrip() {
    let src = make_dir();
    run(create_args(&src, "a")).unwrap();
    run(create_args(&src, "b")).unwrap();
    run(create_args(&src, "c")).unwrap();
    let out = src.path().join("export.json");
    run(export_to_file(&src, out.clone())).unwrap();

    let dst = make_dir();
    run(import_from(&dst, out)).unwrap();

    let reg = dst.path().join("registry");
    let registry = vectorvolt_engine::SchemaRegistry::new(reg);
    assert!(registry.load("a").is_ok());
    assert!(registry.load("b").is_ok());
    assert!(registry.load("c").is_ok());
}

#[test]
fn import_skip_existing_does_not_error_on_duplicate() {
    let src = make_dir();
    run(create_args(&src, "users")).unwrap();
    let out = src.path().join("export.json");
    run(export_to_file(&src, out.clone())).unwrap();

    let dst = make_dir();
    run(import_from(&dst, out.clone())).unwrap();
    // Second import with --skip-existing must succeed.
    run(import_skip(&dst, out)).unwrap();
}

#[test]
fn import_conflict_without_skip_returns_error() {
    let src = make_dir();
    run(create_args(&src, "t")).unwrap();
    let out = src.path().join("export.json");
    run(export_to_file(&src, out.clone())).unwrap();

    let dst = make_dir();
    run(import_from(&dst, out.clone())).unwrap();
    let result = run(import_from(&dst, out));
    assert!(
        result.is_err(),
        "duplicate import without skip_existing must error"
    );
}

#[test]
fn export_then_import_preserves_column_types() {
    let src = make_dir();
    run(args(
        &src,
        CatalogCommand::CreateTable {
            name: "metrics".to_string(),
            columns: vec!["ts:u64".to_string(), "val:f64".to_string()],
            flush_threshold: 1024,
            partitions: 1,
        },
    ))
    .unwrap();
    let out = src.path().join("export.json");
    run(export_to_file(&src, out.clone())).unwrap();

    let dst = make_dir();
    run(import_from(&dst, out)).unwrap();

    let schema = vectorvolt_engine::SchemaRegistry::new(dst.path().join("registry"))
        .load("metrics")
        .unwrap();
    assert_eq!(schema.col_count(), 2);
    let col1 = schema.get(0).unwrap();
    let col2 = schema.get(1).unwrap();
    assert_eq!(col1.col_type.as_str(), "u64");
    assert_eq!(col2.col_type.as_str(), "f64");
}

#[test]
fn import_missing_file_returns_error() {
    let dst = make_dir();
    let result = run(import_from(&dst, dst.path().join("nonexistent.json")));
    assert!(result.is_err());
}

#[test]
fn import_malformed_json_returns_error() {
    let dst = make_dir();
    let bad = dst.path().join("bad.json");
    std::fs::write(&bad, b"not json at all {{{").unwrap();
    let result = run(import_from(&dst, bad));
    assert!(result.is_err());
}

#[test]
fn export_empty_then_import_is_noop() {
    let src = make_dir();
    let out = src.path().join("empty.json");
    run(export_to_file(&src, out.clone())).unwrap();

    let dst = make_dir();
    run(import_from(&dst, out)).unwrap();
    // No tables should have been created.
    let cat = vectorvolt_engine::ShardCatalog::with_registry(
        dst.path().join("catalog"),
        dst.path().join("registry"),
    )
    .unwrap();
    assert_eq!(cat.table_count(), 0);
}
