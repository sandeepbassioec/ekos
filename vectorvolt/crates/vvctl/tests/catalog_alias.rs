//! Integration tests for L121: `vvctl catalog alias-*` (VV-CTL-017).

use tempfile::TempDir;
use vectorvolt_engine::{
    ColumnDef, ShardCatalog, TableSchema, create_alias, list_aliases, remove_alias, resolve_alias,
};
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

// ── CLI integration ───────────────────────────────────────────────────────────

#[test]
fn cli_alias_create_succeeds() {
    let dir = make_dir();
    run(create_args(&dir, "orders_v2")).unwrap();
    run(args(
        &dir,
        CatalogCommand::AliasCreate {
            alias: "orders".into(),
            target: "orders_v2".into(),
            overwrite: false,
        },
    ))
    .unwrap();
    let target = resolve_alias(reg_dir(&dir), "orders").unwrap();
    assert_eq!(target.as_deref(), Some("orders_v2"));
}

#[test]
fn cli_alias_create_nonexistent_target_fails() {
    let dir = make_dir();
    run(create_args(&dir, "real")).unwrap();
    let result = run(args(
        &dir,
        CatalogCommand::AliasCreate {
            alias: "ptr".into(),
            target: "ghost".into(),
            overwrite: false,
        },
    ));
    assert!(result.is_err());
}

#[test]
fn cli_alias_remove_existing() {
    let dir = make_dir();
    run(create_args(&dir, "t1")).unwrap();
    run(args(
        &dir,
        CatalogCommand::AliasCreate {
            alias: "a".into(),
            target: "t1".into(),
            overwrite: false,
        },
    ))
    .unwrap();
    run(args(
        &dir,
        CatalogCommand::AliasRemove { alias: "a".into() },
    ))
    .unwrap();
    assert!(resolve_alias(reg_dir(&dir), "a").unwrap().is_none());
}

#[test]
fn cli_alias_remove_absent_is_noop() {
    let dir = make_dir();
    run(create_args(&dir, "t1")).unwrap();
    run(args(
        &dir,
        CatalogCommand::AliasRemove {
            alias: "ghost".into(),
        },
    ))
    .unwrap();
}

#[test]
fn cli_alias_resolve_prints_target() {
    let dir = make_dir();
    run(create_args(&dir, "t1")).unwrap();
    run(args(
        &dir,
        CatalogCommand::AliasCreate {
            alias: "ptr".into(),
            target: "t1".into(),
            overwrite: false,
        },
    ))
    .unwrap();
    run(args(
        &dir,
        CatalogCommand::AliasResolve { name: "ptr".into() },
    ))
    .unwrap();
}

#[test]
fn cli_alias_resolve_unknown_fails() {
    let dir = make_dir();
    run(create_args(&dir, "t1")).unwrap();
    let result = run(args(
        &dir,
        CatalogCommand::AliasResolve {
            name: "no_such_alias".into(),
        },
    ));
    assert!(result.is_err());
}

#[test]
fn cli_alias_list_shows_all() {
    let dir = make_dir();
    run(create_args(&dir, "t1")).unwrap();
    run(create_args(&dir, "t2")).unwrap();
    run(args(
        &dir,
        CatalogCommand::AliasCreate {
            alias: "a1".into(),
            target: "t1".into(),
            overwrite: false,
        },
    ))
    .unwrap();
    run(args(
        &dir,
        CatalogCommand::AliasCreate {
            alias: "a2".into(),
            target: "t2".into(),
            overwrite: false,
        },
    ))
    .unwrap();
    let pairs = list_aliases(reg_dir(&dir)).unwrap();
    assert_eq!(pairs.len(), 2);
    run(args(&dir, CatalogCommand::AliasList)).unwrap();
}

// ── engine API ────────────────────────────────────────────────────────────────

#[test]
fn engine_create_and_resolve() {
    let dir = make_dir();
    make_table(&dir, "orders_prod");
    create_alias(reg_dir(&dir), "orders", "orders_prod", false).unwrap();
    let t = resolve_alias(reg_dir(&dir), "orders").unwrap();
    assert_eq!(t.as_deref(), Some("orders_prod"));
}

#[test]
fn engine_alias_file_stored_in_registry() {
    let dir = make_dir();
    make_table(&dir, "t1");
    create_alias(reg_dir(&dir), "ptr", "t1", false).unwrap();
    assert!(reg_dir(&dir).join("ptr.alias").is_file());
}

#[test]
fn engine_overwrite_replaces_target() {
    let dir = make_dir();
    make_table(&dir, "t1");
    make_table(&dir, "t2");
    create_alias(reg_dir(&dir), "ptr", "t1", false).unwrap();
    create_alias(reg_dir(&dir), "ptr", "t2", true).unwrap();
    let t = resolve_alias(reg_dir(&dir), "ptr").unwrap();
    assert_eq!(t.as_deref(), Some("t2"));
}

#[test]
fn engine_list_aliases_sorted() {
    let dir = make_dir();
    make_table(&dir, "t1");
    create_alias(reg_dir(&dir), "z_ptr", "t1", false).unwrap();
    create_alias(reg_dir(&dir), "a_ptr", "t1", false).unwrap();
    let pairs = list_aliases(reg_dir(&dir)).unwrap();
    assert_eq!(pairs[0].0, "a_ptr");
    assert_eq!(pairs[1].0, "z_ptr");
}

#[test]
fn engine_remove_alias_cleans_file() {
    let dir = make_dir();
    make_table(&dir, "t1");
    create_alias(reg_dir(&dir), "ptr", "t1", false).unwrap();
    remove_alias(reg_dir(&dir), "ptr").unwrap();
    assert!(!reg_dir(&dir).join("ptr.alias").exists());
}

#[test]
fn engine_schema_unaffected_by_alias() {
    let dir = make_dir();
    make_table(&dir, "t1");
    create_alias(reg_dir(&dir), "ptr", "t1", false).unwrap();
    let reg = vectorvolt_engine::SchemaRegistry::new(reg_dir(&dir));
    let schema = reg.load("t1").unwrap();
    assert_eq!(schema.col_count(), 1);
}
