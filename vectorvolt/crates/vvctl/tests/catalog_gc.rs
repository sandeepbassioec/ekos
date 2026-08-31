//! Integration tests for L114: `vvctl catalog gc` (VV-CTL-010).

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

fn gc(dir: &TempDir) -> CatalogArgs {
    args(
        dir,
        CatalogCommand::Gc {
            max_snapshot_age: None,
            dry_run: false,
        },
    )
}

fn gc_dry(dir: &TempDir) -> CatalogArgs {
    args(
        dir,
        CatalogCommand::Gc {
            max_snapshot_age: None,
            dry_run: true,
        },
    )
}

fn gc_with_age(dir: &TempDir, secs: u64) -> CatalogArgs {
    args(
        dir,
        CatalogCommand::Gc {
            max_snapshot_age: Some(secs),
            dry_run: false,
        },
    )
}

// â”€â”€ basic â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn gc_empty_catalog_is_noop() {
    let dir = make_dir();
    run(gc(&dir)).unwrap();
}

#[test]
fn gc_clean_catalog_is_noop() {
    let dir = make_dir();
    run(create_args(&dir, "orders")).unwrap();
    run(gc(&dir)).unwrap();
    // Table directory must still exist.
    assert!(dir.path().join("catalog").join("orders").exists());
}

#[test]
fn gc_multiple_valid_tables_is_noop() {
    let dir = make_dir();
    run(create_args(&dir, "a")).unwrap();
    run(create_args(&dir, "b")).unwrap();
    run(gc(&dir)).unwrap();
    assert!(dir.path().join("catalog").join("a").exists());
    assert!(dir.path().join("catalog").join("b").exists());
}

// â”€â”€ orphan dirs â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn gc_removes_orphan_dir() {
    let dir = make_dir();
    let orphan = dir.path().join("catalog").join("orphan");
    std::fs::create_dir_all(&orphan).unwrap();
    run(gc(&dir)).unwrap();
    assert!(!orphan.exists(), "orphan dir must be removed");
}

#[test]
fn gc_dry_run_does_not_remove_orphan() {
    let dir = make_dir();
    let orphan = dir.path().join("catalog").join("orphan");
    std::fs::create_dir_all(&orphan).unwrap();
    run(gc_dry(&dir)).unwrap();
    assert!(orphan.exists(), "dry-run must not delete");
}

#[test]
fn gc_valid_table_not_removed_alongside_orphan() {
    let dir = make_dir();
    run(create_args(&dir, "orders")).unwrap();
    let orphan = dir.path().join("catalog").join("orphan");
    std::fs::create_dir_all(&orphan).unwrap();
    run(gc(&dir)).unwrap();
    assert!(!orphan.exists());
    assert!(dir.path().join("catalog").join("orders").exists());
}

// â”€â”€ ghost entries â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn gc_removes_ghost_entry() {
    use vectorvolt_engine::{ColumnDef, SchemaRegistry, TableSchema};
    let dir = make_dir();
    let reg = SchemaRegistry::new(dir.path().join("registry"));
    let schema = TableSchema::new(vec![ColumnDef::u64("pk")]).unwrap();
    reg.save("ghost", &schema).unwrap();
    run(gc(&dir)).unwrap();
    assert!(reg.load("ghost").is_err(), "ghost schema must be deleted");
}

#[test]
fn gc_dry_run_does_not_remove_ghost() {
    use vectorvolt_engine::{ColumnDef, SchemaRegistry, TableSchema};
    let dir = make_dir();
    let reg = SchemaRegistry::new(dir.path().join("registry"));
    let schema = TableSchema::new(vec![ColumnDef::u64("pk")]).unwrap();
    reg.save("ghost", &schema).unwrap();
    run(gc_dry(&dir)).unwrap();
    assert!(reg.load("ghost").is_ok(), "dry-run must not delete");
}

// â”€â”€ stale snapshots â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn gc_no_max_age_keeps_snapshots() {
    use vectorvolt_engine::save_snapshot;
    let dir = make_dir();
    run(create_args(&dir, "t")).unwrap();
    let reg = dir.path().join("registry");
    let snaps = dir.path().join("snapshots");
    save_snapshot(&reg, &snaps).unwrap();
    run(gc(&dir)).unwrap();
    // Snapshot must still be there.
    let entries: Vec<_> = std::fs::read_dir(&snaps)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(entries.len(), 1);
}

#[test]
fn gc_high_max_age_keeps_fresh_snapshot() {
    use vectorvolt_engine::save_snapshot;
    let dir = make_dir();
    run(create_args(&dir, "t")).unwrap();
    let reg = dir.path().join("registry");
    let snaps = dir.path().join("snapshots");
    save_snapshot(&reg, &snaps).unwrap();
    // 1 hour max age â€” just-created snapshot must survive.
    run(gc_with_age(&dir, 3600)).unwrap();
    let entries: Vec<_> = std::fs::read_dir(&snaps)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(entries.len(), 1);
}

// â”€â”€ combined â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn gc_removes_both_orphan_and_ghost() {
    use vectorvolt_engine::{ColumnDef, SchemaRegistry, TableSchema};
    let dir = make_dir();
    let orphan = dir.path().join("catalog").join("orphan");
    std::fs::create_dir_all(&orphan).unwrap();
    let reg = SchemaRegistry::new(dir.path().join("registry"));
    let schema = TableSchema::new(vec![ColumnDef::u64("pk")]).unwrap();
    reg.save("ghost", &schema).unwrap();
    run(gc(&dir)).unwrap();
    assert!(!orphan.exists(), "orphan dir must be removed");
    assert!(reg.load("ghost").is_err(), "ghost schema must be deleted");
}

// â”€â”€ engine API â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn engine_gc_clean_catalog_returns_zero_removed() {
    use vectorvolt_engine::{ColumnDef, ShardCatalog, TableSchema, run_catalog_gc};
    let dir = TempDir::new().unwrap();
    let cat = dir.path().join("catalog");
    let reg = dir.path().join("registry");
    let snaps = dir.path().join("snapshots");
    let catalog = ShardCatalog::with_registry(&cat, &reg).unwrap();
    let schema = TableSchema::new(vec![ColumnDef::u64("pk"), ColumnDef::u64("v")]).unwrap();
    catalog.create_table("t", schema, 1024, 1).unwrap();
    let report = run_catalog_gc(&cat, &reg, &snaps, None, false).unwrap();
    assert!(report.is_clean());
    assert_eq!(report.total_removed(), 0);
}

#[test]
fn engine_gc_dry_run_flag_is_propagated() {
    use vectorvolt_engine::run_catalog_gc;
    let dir = TempDir::new().unwrap();
    let cat = dir.path().join("catalog");
    let reg = dir.path().join("registry");
    let snaps = dir.path().join("snapshots");
    std::fs::create_dir_all(&cat).unwrap();
    let report = run_catalog_gc(&cat, &reg, &snaps, None, true).unwrap();
    assert!(report.dry_run);
}
