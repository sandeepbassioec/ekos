//! Integration tests for L117: `vvctl catalog rename-table` (VV-CTL-013).

use tempfile::TempDir;
use vectorvolt_engine::{ColumnDef, SchemaRegistry, ShardCatalog, TableSchema, rename_table};
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

fn rename_args(dir: &TempDir, from: &str, to: &str, overwrite: bool) -> CatalogArgs {
    args(
        dir,
        CatalogCommand::RenameTable {
            from: from.to_string(),
            to: to.to_string(),
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

// ── CLI integration ───────────────────────────────────────────────────────────

#[test]
fn rename_via_cli_succeeds() {
    let dir = make_dir();
    run(create_args(&dir, "old_name")).unwrap();
    run(rename_args(&dir, "old_name", "new_name", false)).unwrap();
    let reg = SchemaRegistry::new(dir.path().join("registry"));
    assert!(reg.exists("new_name"));
    assert!(!reg.exists("old_name"));
}

#[test]
fn rename_via_cli_moves_catalog_dir() {
    let dir = make_dir();
    run(create_args(&dir, "src")).unwrap();
    run(rename_args(&dir, "src", "dst", false)).unwrap();
    assert!(dir.path().join("catalog").join("dst").is_dir());
    assert!(!dir.path().join("catalog").join("src").exists());
}

#[test]
fn rename_nonexistent_fails() {
    let dir = make_dir();
    let result = run(rename_args(&dir, "ghost", "new", false));
    assert!(result.is_err());
}

#[test]
fn rename_to_existing_name_fails_without_overwrite() {
    let dir = make_dir();
    run(create_args(&dir, "a")).unwrap();
    run(create_args(&dir, "b")).unwrap();
    let result = run(rename_args(&dir, "a", "b", false));
    assert!(result.is_err());
}

#[test]
fn rename_with_overwrite_succeeds() {
    let dir = make_dir();
    run(create_args(&dir, "a")).unwrap();
    run(create_args(&dir, "b")).unwrap();
    run(rename_args(&dir, "a", "b", true)).unwrap();
    let reg = SchemaRegistry::new(dir.path().join("registry"));
    assert!(reg.exists("b"));
    assert!(!reg.exists("a"));
}

// ── engine API ────────────────────────────────────────────────────────────────

#[test]
fn engine_rename_moves_schema() {
    let dir = make_dir();
    make_table(&dir, "orders");
    let (cat, reg) = engine_paths(&dir);
    rename_table(&cat, &reg, "orders", "invoices", false).unwrap();
    let registry = SchemaRegistry::new(&reg);
    assert!(registry.exists("invoices"));
    assert!(!registry.exists("orders"));
}

#[test]
fn engine_rename_moves_catalog_dir() {
    let dir = make_dir();
    make_table(&dir, "orders");
    let (cat, reg) = engine_paths(&dir);
    rename_table(&cat, &reg, "orders", "invoices", false).unwrap();
    assert!(cat.join("invoices").is_dir());
    assert!(!cat.join("orders").exists());
}

#[test]
fn engine_rename_preserves_schema_columns() {
    let dir = make_dir();
    make_table(&dir, "src");
    let (cat, reg) = engine_paths(&dir);
    rename_table(&cat, &reg, "src", "dst", false).unwrap();
    let registry = SchemaRegistry::new(&reg);
    let schema = registry.load("dst").unwrap();
    assert_eq!(schema.col_count(), 2);
}

#[test]
fn engine_rename_nonexistent_errors() {
    let dir = make_dir();
    let (cat, reg) = engine_paths(&dir);
    std::fs::create_dir_all(&cat).unwrap();
    let err = rename_table(&cat, &reg, "ghost", "new", false).unwrap_err();
    assert!(err.to_string().contains("ghost"));
}

#[test]
fn engine_rename_to_existing_errors_without_overwrite() {
    let dir = make_dir();
    make_table(&dir, "a");
    make_table(&dir, "b");
    let (cat, reg) = engine_paths(&dir);
    let err = rename_table(&cat, &reg, "a", "b", false).unwrap_err();
    assert!(err.to_string().contains("b"));
}

#[test]
fn engine_rename_with_overwrite_replaces_target() {
    let dir = make_dir();
    make_table(&dir, "a");
    make_table(&dir, "b");
    let (cat, reg) = engine_paths(&dir);
    rename_table(&cat, &reg, "a", "b", true).unwrap();
    let registry = SchemaRegistry::new(&reg);
    assert!(registry.exists("b"));
    assert!(!registry.exists("a"));
}

#[test]
fn engine_rename_ghost_entry_renames_schema_only() {
    // Ghost: registry entry exists but no catalog dir.
    let dir = make_dir();
    let (cat, reg) = engine_paths(&dir);
    let registry = SchemaRegistry::new(&reg);
    let schema = TableSchema::new(vec![ColumnDef::u64("pk")]).unwrap();
    registry.save("ghost", &schema).unwrap();
    rename_table(&cat, &reg, "ghost", "renamed_ghost", false).unwrap();
    assert!(registry.exists("renamed_ghost"));
    assert!(!registry.exists("ghost"));
}

#[test]
fn engine_rename_idempotent_schema_read() {
    let dir = make_dir();
    make_table(&dir, "t");
    let (cat, reg) = engine_paths(&dir);
    rename_table(&cat, &reg, "t", "u", false).unwrap();
    let registry = SchemaRegistry::new(&reg);
    // Loading the renamed schema twice should give same result.
    let s1 = registry.load("u").unwrap();
    let s2 = registry.load("u").unwrap();
    assert_eq!(s1.col_count(), s2.col_count());
}
