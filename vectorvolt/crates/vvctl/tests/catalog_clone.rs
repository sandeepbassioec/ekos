//! Integration tests for L118: `vvctl catalog clone-table` (VV-CTL-014).

use tempfile::TempDir;
use vectorvolt_engine::{ColumnDef, SchemaRegistry, ShardCatalog, TableSchema, clone_table};
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

fn clone_args(dir: &TempDir, source: &str, dest: &str, overwrite: bool) -> CatalogArgs {
    args(
        dir,
        CatalogCommand::CloneTable {
            source: source.to_string(),
            dest: dest.to_string(),
            flush_threshold: 1024,
            partitions: 1,
            overwrite,
        },
    )
}

fn engine_paths(dir: &TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
    (dir.path().join("catalog"), dir.path().join("registry"))
}

fn make_table(dir: &TempDir, name: &str) {
    let (cat, reg) = engine_paths(dir);
    let catalog = ShardCatalog::with_registry(&cat, &reg).unwrap();
    let schema = TableSchema::new(vec![ColumnDef::u64("pk"), ColumnDef::u64("val")]).unwrap();
    catalog.create_table(name, schema, 1024, 1).unwrap();
}

// â”€â”€ CLI integration â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn clone_via_cli_succeeds() {
    let dir = make_dir();
    run(create_args(&dir, "src")).unwrap();
    run(clone_args(&dir, "src", "dst", false)).unwrap();
    let reg = SchemaRegistry::new(dir.path().join("registry"));
    assert!(reg.exists("dst"));
}

#[test]
fn clone_preserves_source_via_cli() {
    let dir = make_dir();
    run(create_args(&dir, "src")).unwrap();
    run(clone_args(&dir, "src", "dst", false)).unwrap();
    let reg = SchemaRegistry::new(dir.path().join("registry"));
    assert!(reg.exists("src"));
}

#[test]
fn clone_creates_catalog_dir_via_cli() {
    let dir = make_dir();
    run(create_args(&dir, "src")).unwrap();
    run(clone_args(&dir, "src", "dst", false)).unwrap();
    assert!(dir.path().join("catalog").join("dst").is_dir());
}

#[test]
fn clone_nonexistent_source_fails() {
    let dir = make_dir();
    let result = run(clone_args(&dir, "ghost", "dst", false));
    assert!(result.is_err());
}

#[test]
fn clone_to_existing_dest_fails_without_overwrite() {
    let dir = make_dir();
    run(create_args(&dir, "src")).unwrap();
    run(create_args(&dir, "dst")).unwrap();
    let result = run(clone_args(&dir, "src", "dst", false));
    assert!(result.is_err());
}

#[test]
fn clone_with_overwrite_replaces_dest() {
    let dir = make_dir();
    run(create_args(&dir, "src")).unwrap();
    run(create_args(&dir, "dst")).unwrap();
    run(clone_args(&dir, "src", "dst", true)).unwrap();
    let reg = SchemaRegistry::new(dir.path().join("registry"));
    assert!(reg.exists("dst"));
    assert!(reg.exists("src"));
}

// â”€â”€ engine API â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn engine_clone_creates_dest_schema() {
    let dir = make_dir();
    make_table(&dir, "orders");
    let (cat, reg) = engine_paths(&dir);
    clone_table(&cat, &reg, "orders", "orders_copy", 1024, 1, false).unwrap();
    let registry = SchemaRegistry::new(&reg);
    assert!(registry.exists("orders_copy"));
}

#[test]
fn engine_clone_preserves_source_schema() {
    let dir = make_dir();
    make_table(&dir, "orders");
    let (cat, reg) = engine_paths(&dir);
    clone_table(&cat, &reg, "orders", "orders_copy", 1024, 1, false).unwrap();
    let registry = SchemaRegistry::new(&reg);
    assert!(registry.exists("orders"));
}

#[test]
fn engine_clone_dest_has_same_columns() {
    let dir = make_dir();
    make_table(&dir, "src");
    let (cat, reg) = engine_paths(&dir);
    clone_table(&cat, &reg, "src", "dst", 1024, 1, false).unwrap();
    let registry = SchemaRegistry::new(&reg);
    let schema = registry.load("dst").unwrap();
    assert_eq!(schema.col_count(), 2);
}

#[test]
fn engine_clone_creates_catalog_dir() {
    let dir = make_dir();
    make_table(&dir, "src");
    let (cat, reg) = engine_paths(&dir);
    clone_table(&cat, &reg, "src", "dst", 1024, 1, false).unwrap();
    assert!(cat.join("dst").is_dir());
}

#[test]
fn engine_clone_nonexistent_errors() {
    let dir = make_dir();
    let (cat, reg) = engine_paths(&dir);
    std::fs::create_dir_all(&cat).unwrap();
    let err = clone_table(&cat, &reg, "ghost", "dst", 1024, 1, false).unwrap_err();
    assert!(err.to_string().contains("ghost"));
}

#[test]
fn engine_clone_to_existing_errors_without_overwrite() {
    let dir = make_dir();
    make_table(&dir, "src");
    make_table(&dir, "dst");
    let (cat, reg) = engine_paths(&dir);
    let err = clone_table(&cat, &reg, "src", "dst", 1024, 1, false).unwrap_err();
    assert!(err.to_string().contains("dst"));
}

#[test]
fn engine_clone_with_overwrite_succeeds() {
    let dir = make_dir();
    make_table(&dir, "src");
    make_table(&dir, "dst");
    let (cat, reg) = engine_paths(&dir);
    clone_table(&cat, &reg, "src", "dst", 1024, 1, true).unwrap();
    let registry = SchemaRegistry::new(&reg);
    assert!(registry.exists("dst"));
    assert!(registry.exists("src"));
}
