//! Integration tests for L107: SchemaRegistry embedded in ShardCatalog (VV-CTL-003).

use tempfile::TempDir;
use vectorvolt_engine::{ColumnDef, SchemaRegistry, ShardCatalog, TableSchema};

fn two_col_schema() -> TableSchema {
    TableSchema::new(vec![ColumnDef::u64("pk"), ColumnDef::u64("val")]).unwrap()
}

fn make_catalog_with_registry() -> (ShardCatalog, TempDir) {
    let dir = TempDir::new().unwrap();
    let catalog =
        ShardCatalog::with_registry(dir.path().join("catalog"), dir.path().join("registry"))
            .unwrap();
    (catalog, dir)
}

// ── with_registry constructor ─────────────────────────────────────────────────

#[test]
fn with_registry_has_registry() {
    let (cat, _d) = make_catalog_with_registry();
    assert!(cat.has_registry());
}

#[test]
fn new_does_not_have_registry() {
    let dir = TempDir::new().unwrap();
    let cat = ShardCatalog::new(dir.path());
    assert!(!cat.has_registry());
}

#[test]
fn with_registry_creates_dirs() {
    let dir = TempDir::new().unwrap();
    let reg_dir = dir.path().join("registry");
    let _cat = ShardCatalog::with_registry(dir.path().join("catalog"), &reg_dir).unwrap();
    assert!(reg_dir.exists());
}

// ── auto-persist on create_table ─────────────────────────────────────────────

#[test]
fn create_table_persists_schema_to_registry() {
    let dir = TempDir::new().unwrap();
    let reg_dir = dir.path().join("registry");
    let cat = ShardCatalog::with_registry(dir.path().join("catalog"), &reg_dir).unwrap();
    cat.create_table("orders", two_col_schema(), 1000, 1)
        .unwrap();

    let reg = SchemaRegistry::new(&reg_dir);
    assert!(reg.exists("orders"));
}

#[test]
fn create_table_schema_roundtrip_via_registry() {
    let dir = TempDir::new().unwrap();
    let reg_dir = dir.path().join("registry");
    let cat = ShardCatalog::with_registry(dir.path().join("catalog"), &reg_dir).unwrap();
    cat.create_table("users", two_col_schema(), 1000, 1)
        .unwrap();

    let reg = SchemaRegistry::new(&reg_dir);
    let schema = reg.load("users").unwrap();
    assert_eq!(schema.col_count(), 2);
}

#[test]
fn create_table_without_registry_does_not_write_schema() {
    let dir = TempDir::new().unwrap();
    let cat = ShardCatalog::new(dir.path());
    cat.create_table("t", two_col_schema(), 1000, 1).unwrap();
    // no .schema files should exist under dir
    let schema_files: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "schema").unwrap_or(false))
        .collect();
    assert!(schema_files.is_empty());
}

// ── auto-remove on drop_table ─────────────────────────────────────────────────

#[test]
fn drop_table_removes_schema_from_registry() {
    let dir = TempDir::new().unwrap();
    let reg_dir = dir.path().join("registry");
    let cat = ShardCatalog::with_registry(dir.path().join("catalog"), &reg_dir).unwrap();
    cat.create_table("tmp", two_col_schema(), 1000, 1).unwrap();
    cat.drop_table("tmp").unwrap();

    let reg = SchemaRegistry::new(&reg_dir);
    assert!(!reg.exists("tmp"));
}

#[test]
fn drop_table_without_registry_does_not_panic() {
    let dir = TempDir::new().unwrap();
    let cat = ShardCatalog::new(dir.path());
    cat.create_table("t", two_col_schema(), 1000, 1).unwrap();
    cat.drop_table("t").unwrap();
}

// ── restore_all ───────────────────────────────────────────────────────────────

#[test]
fn restore_all_reloads_persisted_tables() {
    let dir = TempDir::new().unwrap();
    let cat_dir = dir.path().join("catalog");
    let reg_dir = dir.path().join("registry");

    // First process: create and drop (registry persists).
    {
        let cat = ShardCatalog::with_registry(&cat_dir, &reg_dir).unwrap();
        cat.create_table("events", two_col_schema(), 1000, 1)
            .unwrap();
    }

    // Second process: open catalog, restore from registry.
    let cat2 = ShardCatalog::with_registry(&cat_dir, &reg_dir).unwrap();
    let restored = cat2.restore_all(1000, 1).unwrap();
    assert_eq!(restored, 1);
    assert!(cat2.contains("events"));
}

#[test]
fn restore_all_returns_zero_without_registry() {
    let dir = TempDir::new().unwrap();
    let cat = ShardCatalog::new(dir.path());
    assert_eq!(cat.restore_all(1000, 1).unwrap(), 0);
}

#[test]
fn restore_all_skips_already_loaded_tables() {
    let dir = TempDir::new().unwrap();
    let cat_dir = dir.path().join("catalog");
    let reg_dir = dir.path().join("registry");

    let cat = ShardCatalog::with_registry(&cat_dir, &reg_dir).unwrap();
    cat.create_table("t", two_col_schema(), 1000, 1).unwrap();
    // t is already in-process — restore_all should skip it.
    let restored = cat.restore_all(1000, 1).unwrap();
    assert_eq!(restored, 0);
}

#[test]
fn restore_all_multiple_tables() {
    let dir = TempDir::new().unwrap();
    let cat_dir = dir.path().join("catalog");
    let reg_dir = dir.path().join("registry");

    {
        let cat = ShardCatalog::with_registry(&cat_dir, &reg_dir).unwrap();
        cat.create_table("a", two_col_schema(), 1000, 1).unwrap();
        cat.create_table("b", two_col_schema(), 1000, 1).unwrap();
        cat.create_table("c", two_col_schema(), 1000, 1).unwrap();
    }

    let cat2 = ShardCatalog::with_registry(&cat_dir, &reg_dir).unwrap();
    let restored = cat2.restore_all(1000, 1).unwrap();
    assert_eq!(restored, 3);
    assert!(cat2.contains("a"));
    assert!(cat2.contains("b"));
    assert!(cat2.contains("c"));
}

#[test]
fn restored_table_is_functional() {
    let dir = TempDir::new().unwrap();
    let cat_dir = dir.path().join("catalog");
    let reg_dir = dir.path().join("registry");

    {
        let cat = ShardCatalog::with_registry(&cat_dir, &reg_dir).unwrap();
        cat.create_table("data", two_col_schema(), 1000, 1).unwrap();
    }

    let cat2 = ShardCatalog::with_registry(&cat_dir, &reg_dir).unwrap();
    cat2.restore_all(1000, 1).unwrap();
    let mgr = cat2.manager("data").unwrap();
    mgr.insert(1, vec![1, 42]).unwrap();
    assert_eq!(mgr.get(1).unwrap(), Some(vec![1, 42]));
}
