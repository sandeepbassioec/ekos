//! Integration tests for L119: `vvctl catalog tag-*` (VV-CTL-015).

use tempfile::TempDir;
use vectorvolt_engine::{ColumnDef, SchemaRegistry, ShardCatalog, TableSchema, get_tags, set_tag};
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
            columns: vec!["pk".to_string()],
            flush_threshold: 1024,
            partitions: 1,
        },
    )
}

fn reg_dir(dir: &TempDir) -> std::path::PathBuf {
    dir.path().join("registry")
}

fn make_table(dir: &TempDir, name: &str) {
    let cat = dir.path().join("catalog");
    let reg = reg_dir(dir);
    let catalog = ShardCatalog::with_registry(&cat, &reg).unwrap();
    let schema = TableSchema::new(vec![ColumnDef::u64("pk")]).unwrap();
    catalog.create_table(name, schema, 1024, 1).unwrap();
}

// â”€â”€ CLI integration â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn cli_tag_set_succeeds() {
    let dir = make_dir();
    run(create_args(&dir, "t1")).unwrap();
    run(args(
        &dir,
        CatalogCommand::TagSet {
            table: "t1".into(),
            key: "env".into(),
            value: "prod".into(),
        },
    ))
    .unwrap();
    let tags = get_tags(reg_dir(&dir), "t1").unwrap();
    assert_eq!(tags.get("env").map(String::as_str), Some("prod"));
}

#[test]
fn cli_tag_get_with_key_returns_value() {
    let dir = make_dir();
    run(create_args(&dir, "t1")).unwrap();
    run(args(
        &dir,
        CatalogCommand::TagSet {
            table: "t1".into(),
            key: "owner".into(),
            value: "alice".into(),
        },
    ))
    .unwrap();
    run(args(
        &dir,
        CatalogCommand::TagGet {
            table: "t1".into(),
            key: Some("owner".into()),
        },
    ))
    .unwrap();
}

#[test]
fn cli_tag_get_missing_key_errors() {
    let dir = make_dir();
    run(create_args(&dir, "t1")).unwrap();
    let result = run(args(
        &dir,
        CatalogCommand::TagGet {
            table: "t1".into(),
            key: Some("nosuchkey".into()),
        },
    ));
    assert!(result.is_err());
}

#[test]
fn cli_tag_remove_existing_key_succeeds() {
    let dir = make_dir();
    run(create_args(&dir, "t1")).unwrap();
    run(args(
        &dir,
        CatalogCommand::TagSet {
            table: "t1".into(),
            key: "team".into(),
            value: "platform".into(),
        },
    ))
    .unwrap();
    run(args(
        &dir,
        CatalogCommand::TagRemove {
            table: "t1".into(),
            key: "team".into(),
        },
    ))
    .unwrap();
    let tags = get_tags(reg_dir(&dir), "t1").unwrap();
    assert!(!tags.contains_key("team"));
}

#[test]
fn cli_tag_remove_absent_key_still_succeeds() {
    let dir = make_dir();
    run(create_args(&dir, "t1")).unwrap();
    run(args(
        &dir,
        CatalogCommand::TagRemove {
            table: "t1".into(),
            key: "ghost".into(),
        },
    ))
    .unwrap();
}

#[test]
fn cli_tag_clear_removes_all_tags() {
    let dir = make_dir();
    run(create_args(&dir, "t1")).unwrap();
    run(args(
        &dir,
        CatalogCommand::TagSet {
            table: "t1".into(),
            key: "a".into(),
            value: "1".into(),
        },
    ))
    .unwrap();
    run(args(&dir, CatalogCommand::TagClear { table: "t1".into() })).unwrap();
    let tags = get_tags(reg_dir(&dir), "t1").unwrap();
    assert!(tags.is_empty());
}

// â”€â”€ engine API â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn engine_set_tag_persists() {
    let dir = make_dir();
    make_table(&dir, "orders");
    set_tag(reg_dir(&dir), "orders", "cost_centre", "cc42").unwrap();
    let tags = get_tags(reg_dir(&dir), "orders").unwrap();
    assert_eq!(tags.get("cost_centre").map(String::as_str), Some("cc42"));
}

#[test]
fn engine_multiple_tags_all_survive() {
    let dir = make_dir();
    make_table(&dir, "orders");
    set_tag(reg_dir(&dir), "orders", "env", "staging").unwrap();
    set_tag(reg_dir(&dir), "orders", "team", "backend").unwrap();
    let tags = get_tags(reg_dir(&dir), "orders").unwrap();
    assert_eq!(tags.len(), 2);
    assert!(tags.contains_key("env"));
    assert!(tags.contains_key("team"));
}

#[test]
fn engine_schema_unaffected_by_tags() {
    let dir = make_dir();
    make_table(&dir, "items");
    set_tag(reg_dir(&dir), "items", "label", "x").unwrap();
    let reg = SchemaRegistry::new(reg_dir(&dir));
    let schema = reg.load("items").unwrap();
    assert_eq!(schema.col_count(), 1);
}

#[test]
fn engine_set_tag_nonexistent_table_errors() {
    let dir = make_dir();
    make_table(&dir, "exists");
    let err = set_tag(reg_dir(&dir), "ghost", "k", "v").unwrap_err();
    assert!(err.to_string().contains("ghost"));
}

#[test]
fn engine_tag_file_stored_in_registry_dir() {
    let dir = make_dir();
    make_table(&dir, "t1");
    set_tag(reg_dir(&dir), "t1", "x", "y").unwrap();
    assert!(reg_dir(&dir).join("t1.tags").is_file());
}

#[test]
fn engine_clear_tags_idempotent_on_empty() {
    let dir = make_dir();
    make_table(&dir, "t1");
    vectorvolt_engine::clear_tags(reg_dir(&dir), "t1").unwrap();
    vectorvolt_engine::clear_tags(reg_dir(&dir), "t1").unwrap();
}

#[test]
fn engine_get_tags_nonexistent_table_errors() {
    let dir = make_dir();
    make_table(&dir, "real");
    let err = get_tags(reg_dir(&dir), "nope").unwrap_err();
    assert!(err.to_string().contains("nope"));
}
