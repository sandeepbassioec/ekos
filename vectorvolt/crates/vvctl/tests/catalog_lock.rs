//! Integration tests for L120: `vvctl catalog lock-table/unlock-table/lock-status` (VV-CTL-016).

use tempfile::TempDir;
use vectorvolt_engine::{
    ColumnDef, ShardCatalog, TableSchema, get_lock_info, is_locked, lock_table, unlock_table,
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

// â”€â”€ CLI integration â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn cli_lock_table_succeeds() {
    let dir = make_dir();
    run(create_args(&dir, "t1")).unwrap();
    run(args(
        &dir,
        CatalogCommand::LockTable {
            table: "t1".into(),
            reason: None,
            force: false,
        },
    ))
    .unwrap();
    assert!(is_locked(reg_dir(&dir), "t1").unwrap());
}

#[test]
fn cli_unlock_table_clears_lock() {
    let dir = make_dir();
    run(create_args(&dir, "t1")).unwrap();
    run(args(
        &dir,
        CatalogCommand::LockTable {
            table: "t1".into(),
            reason: None,
            force: false,
        },
    ))
    .unwrap();
    run(args(
        &dir,
        CatalogCommand::UnlockTable { table: "t1".into() },
    ))
    .unwrap();
    assert!(!is_locked(reg_dir(&dir), "t1").unwrap());
}

#[test]
fn cli_lock_twice_without_force_fails() {
    let dir = make_dir();
    run(create_args(&dir, "t1")).unwrap();
    run(args(
        &dir,
        CatalogCommand::LockTable {
            table: "t1".into(),
            reason: None,
            force: false,
        },
    ))
    .unwrap();
    let result = run(args(
        &dir,
        CatalogCommand::LockTable {
            table: "t1".into(),
            reason: None,
            force: false,
        },
    ));
    assert!(result.is_err());
}

#[test]
fn cli_lock_with_force_overwrites() {
    let dir = make_dir();
    run(create_args(&dir, "t1")).unwrap();
    run(args(
        &dir,
        CatalogCommand::LockTable {
            table: "t1".into(),
            reason: Some("first".into()),
            force: false,
        },
    ))
    .unwrap();
    run(args(
        &dir,
        CatalogCommand::LockTable {
            table: "t1".into(),
            reason: Some("second".into()),
            force: true,
        },
    ))
    .unwrap();
    let info = get_lock_info(reg_dir(&dir), "t1").unwrap().unwrap();
    assert_eq!(info.reason.as_deref(), Some("second"));
}

#[test]
fn cli_lock_status_reports_locked() {
    let dir = make_dir();
    run(create_args(&dir, "t1")).unwrap();
    run(args(
        &dir,
        CatalogCommand::LockTable {
            table: "t1".into(),
            reason: Some("maint".into()),
            force: false,
        },
    ))
    .unwrap();
    run(args(
        &dir,
        CatalogCommand::LockStatus { table: "t1".into() },
    ))
    .unwrap();
}

#[test]
fn cli_unlock_noop_on_unlocked_table() {
    let dir = make_dir();
    run(create_args(&dir, "t1")).unwrap();
    run(args(
        &dir,
        CatalogCommand::UnlockTable { table: "t1".into() },
    ))
    .unwrap();
}

// â”€â”€ engine API â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn engine_lock_stores_reason() {
    let dir = make_dir();
    make_table(&dir, "orders");
    lock_table(reg_dir(&dir), "orders", Some("backup"), false).unwrap();
    let info = get_lock_info(reg_dir(&dir), "orders").unwrap().unwrap();
    assert_eq!(info.reason.as_deref(), Some("backup"));
}

#[test]
fn engine_lock_file_in_registry_dir() {
    let dir = make_dir();
    make_table(&dir, "orders");
    lock_table(reg_dir(&dir), "orders", None, false).unwrap();
    assert!(reg_dir(&dir).join("orders.lock").is_file());
}

#[test]
fn engine_unlock_removes_lock_file() {
    let dir = make_dir();
    make_table(&dir, "orders");
    lock_table(reg_dir(&dir), "orders", None, false).unwrap();
    unlock_table(reg_dir(&dir), "orders").unwrap();
    assert!(!reg_dir(&dir).join("orders.lock").exists());
}

#[test]
fn engine_get_lock_info_none_when_unlocked() {
    let dir = make_dir();
    make_table(&dir, "t1");
    let info = get_lock_info(reg_dir(&dir), "t1").unwrap();
    assert!(info.is_none());
}

#[test]
fn engine_lock_stores_locked_at_timestamp() {
    let dir = make_dir();
    make_table(&dir, "t1");
    lock_table(reg_dir(&dir), "t1", None, false).unwrap();
    let info = get_lock_info(reg_dir(&dir), "t1").unwrap().unwrap();
    assert!(info.locked_at > 0);
}

#[test]
fn engine_nonexistent_table_errors_on_lock() {
    let dir = make_dir();
    make_table(&dir, "real");
    let err = lock_table(reg_dir(&dir), "ghost", None, false).unwrap_err();
    assert!(err.to_string().contains("ghost"));
}

#[test]
fn engine_schema_unaffected_by_lock() {
    let dir = make_dir();
    make_table(&dir, "t1");
    lock_table(reg_dir(&dir), "t1", None, false).unwrap();
    let reg = vectorvolt_engine::SchemaRegistry::new(reg_dir(&dir));
    let schema = reg.load("t1").unwrap();
    assert_eq!(schema.col_count(), 1);
}
