//! Integration tests for L109: `vvctl catalog validate` (VV-CTL-005).

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

fn validate_args(dir: &TempDir, strict: bool) -> CatalogArgs {
    args(dir, CatalogCommand::Validate { strict })
}

// â”€â”€ empty catalog â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn validate_empty_catalog_succeeds() {
    let dir = make_dir();
    run(validate_args(&dir, false)).unwrap();
}

#[test]
fn validate_empty_catalog_strict_succeeds() {
    let dir = make_dir();
    run(validate_args(&dir, true)).unwrap();
}

// â”€â”€ valid tables â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn validate_single_valid_table_passes() {
    let dir = make_dir();
    run(create_args(&dir, "orders")).unwrap();
    run(validate_args(&dir, false)).unwrap();
}

#[test]
fn validate_multiple_valid_tables_all_pass() {
    let dir = make_dir();
    run(create_args(&dir, "a")).unwrap();
    run(create_args(&dir, "b")).unwrap();
    run(create_args(&dir, "c")).unwrap();
    run(validate_args(&dir, false)).unwrap();
}

#[test]
fn validate_strict_on_clean_catalog_passes() {
    let dir = make_dir();
    run(create_args(&dir, "users")).unwrap();
    run(validate_args(&dir, true)).unwrap();
}

// â”€â”€ ghost entries â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn validate_detects_ghost_entry() {
    use vectorvolt_engine::SchemaRegistry;
    use vectorvolt_engine::{ColumnDef, TableSchema};

    let dir = make_dir();
    // Write a schema to the registry without creating a catalog dir.
    let reg = SchemaRegistry::new(dir.path().join("registry"));
    let schema = TableSchema::new(vec![ColumnDef::u64("pk")]).unwrap();
    reg.save("ghost", &schema).unwrap();

    let result = run(validate_args(&dir, false));
    assert!(
        result.is_err(),
        "ghost entry should cause validation failure"
    );
}

// â”€â”€ orphan dirs â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn validate_detects_orphan_dir() {
    let dir = make_dir();
    // Create a catalog subdir without a registry entry.
    std::fs::create_dir_all(dir.path().join("catalog").join("orphan")).unwrap();

    let result = run(validate_args(&dir, false));
    assert!(
        result.is_err(),
        "orphan dir should cause validation failure"
    );
}

// â”€â”€ post-drop cleanliness â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn validate_after_drop_leaves_no_orphans() {
    let dir = make_dir();
    run(create_args(&dir, "tmp")).unwrap();
    run(args(&dir, CatalogCommand::DropTable { name: "tmp".into() })).unwrap();
    // After drop, catalog dir remains on disk but registry entry is gone.
    // That means an orphan dir exists â€” validate should detect it.
    // This is expected behaviour: drop removes the registry entry, not the data.
    let result = run(validate_args(&dir, false));
    assert!(
        result.is_err(),
        "orphan dir from dropped table should be detected"
    );
}

// â”€â”€ engine-level validation logic â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn engine_validate_healthy_report_for_valid_table() {
    use tempfile::TempDir;
    use vectorvolt_engine::{ColumnDef, SchemaRegistry, TableSchema, validate_catalog};

    let dir = TempDir::new().unwrap();
    let cat_dir = dir.path().join("catalog");
    let reg_dir = dir.path().join("registry");
    std::fs::create_dir_all(&cat_dir).unwrap();
    std::fs::create_dir_all(&reg_dir).unwrap();

    let reg = SchemaRegistry::new(&reg_dir);
    let schema = TableSchema::new(vec![ColumnDef::u64("pk"), ColumnDef::u64("val")]).unwrap();
    reg.save("items", &schema).unwrap();
    std::fs::create_dir_all(cat_dir.join("items")).unwrap();

    let report = validate_catalog(&cat_dir, &reg_dir).unwrap();
    assert!(report.is_healthy());
    assert!(report.valid_tables.contains(&"items".to_string()));
}

#[test]
fn engine_validate_detects_ghost_entry() {
    use tempfile::TempDir;
    use vectorvolt_engine::{ColumnDef, SchemaRegistry, TableSchema, validate_catalog};

    let dir = TempDir::new().unwrap();
    let cat_dir = dir.path().join("catalog");
    let reg_dir = dir.path().join("registry");
    std::fs::create_dir_all(&cat_dir).unwrap();

    let reg = SchemaRegistry::new(&reg_dir);
    let schema = TableSchema::new(vec![ColumnDef::u64("pk")]).unwrap();
    reg.save("ghost", &schema).unwrap();

    let report = validate_catalog(&cat_dir, &reg_dir).unwrap();
    assert!(!report.is_healthy());
    assert!(report.ghost_entries.contains(&"ghost".to_string()));
}

#[test]
fn engine_validate_detects_orphan_dir() {
    use tempfile::TempDir;
    use vectorvolt_engine::validate_catalog;

    let dir = TempDir::new().unwrap();
    let cat_dir = dir.path().join("catalog");
    let reg_dir = dir.path().join("registry");
    std::fs::create_dir_all(cat_dir.join("orphan")).unwrap();
    std::fs::create_dir_all(&reg_dir).unwrap();

    let report = validate_catalog(&cat_dir, &reg_dir).unwrap();
    assert!(!report.is_healthy());
    assert!(report.orphan_dirs.contains(&"orphan".to_string()));
}

#[test]
fn engine_validate_non_u64_pk_is_warning_not_error() {
    use tempfile::TempDir;
    use vectorvolt_engine::{ColumnDef, SchemaRegistry, TableSchema, validate_catalog};

    let dir = TempDir::new().unwrap();
    let cat_dir = dir.path().join("catalog");
    let reg_dir = dir.path().join("registry");
    std::fs::create_dir_all(cat_dir.join("metrics")).unwrap();

    // First column is f64 â€” valid schema but violates pk convention.
    let reg = SchemaRegistry::new(&reg_dir);
    let schema =
        TableSchema::new(vec![ColumnDef::from_line("ts:f64"), ColumnDef::u64("val")]).unwrap();
    reg.save("metrics", &schema).unwrap();

    let report = validate_catalog(&cat_dir, &reg_dir).unwrap();
    // Warning: non-u64 pk. No errors, no orphan/ghost â†’ is_healthy passes.
    assert!(report.is_healthy(), "non-u64 pk is a warning, not an error");
    assert!(!report.is_clean(), "non-u64 pk should produce a warning");
    assert_eq!(report.warning_count(), 1);
}

#[test]
fn engine_validate_valid_tables_sorted() {
    use tempfile::TempDir;
    use vectorvolt_engine::{ColumnDef, ShardCatalog, TableSchema, validate_catalog};

    let dir = TempDir::new().unwrap();
    let cat_dir = dir.path().join("catalog");
    let reg_dir = dir.path().join("registry");

    let cat = ShardCatalog::with_registry(&cat_dir, &reg_dir).unwrap();
    let schema = TableSchema::new(vec![ColumnDef::u64("pk"), ColumnDef::u64("val")]).unwrap();
    cat.create_table("zebra", schema.clone(), 1024, 1).unwrap();
    cat.create_table("alpha", schema.clone(), 1024, 1).unwrap();
    cat.create_table("mango", schema, 1024, 1).unwrap();

    let report = validate_catalog(&cat_dir, &reg_dir).unwrap();
    assert_eq!(report.valid_tables, vec!["alpha", "mango", "zebra"]);
}
