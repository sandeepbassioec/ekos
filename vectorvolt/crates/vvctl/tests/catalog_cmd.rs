//! Integration tests for L106: `vvctl catalog` subcommand (VV-CTL-002).
//!
//! Tests call the command `run()` function directly to avoid spawning a process.

use tempfile::TempDir;
use vvctl::commands::catalog::{CatalogArgs, CatalogCommand, run};

fn args(dir: &TempDir, cmd: CatalogCommand) -> CatalogArgs {
    CatalogArgs {
        dir: dir.path().to_path_buf(),
        command: cmd,
    }
}

fn create_args(dir: &TempDir, name: &str, columns: Vec<&str>) -> CatalogArgs {
    args(
        dir,
        CatalogCommand::CreateTable {
            name: name.to_string(),
            columns: columns.into_iter().map(str::to_string).collect(),
            flush_threshold: 1024,
            partitions: 2,
        },
    )
}

// ── create-table ─────────────────────────────────────────────────────────────

#[test]
fn create_table_succeeds() {
    let dir = TempDir::new().unwrap();
    run(create_args(&dir, "users", vec!["id", "age"])).unwrap();
}

#[test]
fn create_table_catalog_dir_created() {
    let dir = TempDir::new().unwrap();
    run(create_args(&dir, "orders", vec!["id", "total"])).unwrap();
    assert!(dir.path().join("catalog").exists());
}

#[test]
fn create_table_registry_dir_created() {
    let dir = TempDir::new().unwrap();
    run(create_args(&dir, "events", vec!["id"])).unwrap();
    assert!(dir.path().join("registry").exists());
}

#[test]
fn create_table_schema_persisted_to_registry() {
    use vectorvolt_engine::SchemaRegistry;

    let dir = TempDir::new().unwrap();
    run(create_args(&dir, "items", vec!["id", "price"])).unwrap();

    let registry = SchemaRegistry::new(dir.path().join("registry"));
    assert!(registry.exists("items"));
    let schema = registry.load("items").unwrap();
    assert_eq!(schema.col_count(), 2);
}

#[test]
fn create_table_explicit_type_accepted() {
    let dir = TempDir::new().unwrap();
    run(create_args(&dir, "metrics", vec!["id:u64", "score:f64"])).unwrap();
}

#[test]
fn create_table_no_columns_errors() {
    let dir = TempDir::new().unwrap();
    let result = run(args(
        &dir,
        CatalogCommand::CreateTable {
            name: "bad".into(),
            columns: vec![],
            flush_threshold: 1024,
            partitions: 2,
        },
    ));
    assert!(result.is_err());
}

// ── list-tables ───────────────────────────────────────────────────────────────

#[test]
fn list_tables_empty_catalog_prints_none() {
    let dir = TempDir::new().unwrap();
    run(args(&dir, CatalogCommand::ListTables)).unwrap();
}

#[test]
fn list_tables_shows_created_table() {
    use vectorvolt_engine::SchemaRegistry;

    let dir = TempDir::new().unwrap();
    run(create_args(&dir, "alpha", vec!["id"])).unwrap();

    let registry = SchemaRegistry::new(dir.path().join("registry"));
    let tables = registry.list_tables().unwrap();
    assert!(tables.contains(&"alpha".to_string()));
}

// ── drop-table ────────────────────────────────────────────────────────────────

#[test]
fn drop_table_removes_from_registry() {
    use vectorvolt_engine::SchemaRegistry;

    let dir = TempDir::new().unwrap();
    run(create_args(&dir, "tmp", vec!["id"])).unwrap();
    run(args(&dir, CatalogCommand::DropTable { name: "tmp".into() })).unwrap();

    let registry = SchemaRegistry::new(dir.path().join("registry"));
    assert!(!registry.exists("tmp"));
}

#[test]
fn drop_nonexistent_table_errors() {
    let dir = TempDir::new().unwrap();
    let result = run(args(
        &dir,
        CatalogCommand::DropTable {
            name: "ghost".into(),
        },
    ));
    assert!(result.is_err());
}

// ── describe-table ────────────────────────────────────────────────────────────

#[test]
fn describe_table_succeeds_after_create() {
    let dir = TempDir::new().unwrap();
    run(create_args(&dir, "products", vec!["id", "price"])).unwrap();
    run(args(
        &dir,
        CatalogCommand::DescribeTable {
            name: "products".into(),
        },
    ))
    .unwrap();
}

#[test]
fn describe_nonexistent_table_errors() {
    let dir = TempDir::new().unwrap();
    let result = run(args(
        &dir,
        CatalogCommand::DescribeTable {
            name: "ghost".into(),
        },
    ));
    assert!(result.is_err());
}

// ── round-trip ───────────────────────────────────────────────────────────────

#[test]
fn create_multiple_tables_all_persist() {
    use vectorvolt_engine::SchemaRegistry;

    let dir = TempDir::new().unwrap();
    run(create_args(&dir, "t1", vec!["id"])).unwrap();
    run(create_args(&dir, "t2", vec!["id", "val"])).unwrap();
    run(create_args(&dir, "t3", vec!["id", "a", "b"])).unwrap();

    let registry = SchemaRegistry::new(dir.path().join("registry"));
    let tables = registry.list_tables().unwrap();
    assert!(tables.contains(&"t1".to_string()));
    assert!(tables.contains(&"t2".to_string()));
    assert!(tables.contains(&"t3".to_string()));
}
