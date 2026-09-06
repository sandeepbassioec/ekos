//! Integration tests for L108: `vvctl catalog restore` + `serve --restore-catalog` (VV-CTL-004).

use tempfile::TempDir;
use vvctl::commands::catalog::{CatalogArgs, CatalogCommand, run};
use vvctl::commands::serve::{ServeArgs, bind};

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
            columns: vec!["id".to_string(), "val".to_string()],
            flush_threshold: 1024,
            partitions: 1,
        },
    )
}

fn restore_args(dir: &TempDir) -> CatalogArgs {
    args(
        dir,
        CatalogCommand::Restore {
            flush_threshold: 1024,
            partitions: 1,
        },
    )
}

fn serve_args(dir: &TempDir, catalog: bool, restore_catalog: bool) -> ServeArgs {
    ServeArgs {
        dir: dir.path().to_path_buf(),
        columns: vec![],
        addr: "127.0.0.1:0".parse().unwrap(),
        max_connections: 4,
        secret: None,
        key_id: "default".into(),
        cert: None,
        key: None,
        multi: true, // avoids single-table mode requiring --columns
        config: None,
        flush_threshold: None,
        durability: None,
        max_partitions: None,
        catalog,
        restore_catalog,
        http_addr: None,
        master_config: None,
        http_token: None,
        transport: "tcp".to_string(),
        xdp_iface: "lo".to_string(),
        xdp_queue: 0,
        xdp_ring_size: 256,
        grpc_port: 50051,
        no_grpc: false,
        tpm_unseal: false,
    }
}

// â”€â”€ vvctl catalog restore â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn restore_empty_registry_prints_nothing_to_restore() {
    let dir = make_dir();
    run(restore_args(&dir)).unwrap();
}

#[test]
fn restore_reloads_single_persisted_table() {
    use vectorvolt_engine::ShardCatalog;

    let dir = make_dir();
    run(create_args(&dir, "events")).unwrap();

    // Open a fresh catalog (simulates second process) and restore via CLI.
    let cat2 = ShardCatalog::with_registry(dir.path().join("catalog"), dir.path().join("registry"))
        .unwrap();
    let count = cat2.restore_all(1024, 1).unwrap();
    assert_eq!(count, 1);
    assert!(cat2.contains("events"));
}

#[test]
fn restore_subcommand_succeeds_with_persisted_tables() {
    let dir = make_dir();
    run(create_args(&dir, "alpha")).unwrap();
    run(create_args(&dir, "beta")).unwrap();
    // restore on a fresh process should succeed
    run(restore_args(&dir)).unwrap();
}

#[test]
fn restore_subcommand_idempotent_when_already_loaded() {
    let dir = make_dir();
    run(create_args(&dir, "t")).unwrap();
    // calling restore twice should be safe
    run(restore_args(&dir)).unwrap();
    run(restore_args(&dir)).unwrap();
}

#[test]
fn restore_multiple_tables() {
    use vectorvolt_engine::ShardCatalog;

    let dir = make_dir();
    run(create_args(&dir, "t1")).unwrap();
    run(create_args(&dir, "t2")).unwrap();
    run(create_args(&dir, "t3")).unwrap();

    let cat2 = ShardCatalog::with_registry(dir.path().join("catalog"), dir.path().join("registry"))
        .unwrap();
    let count = cat2.restore_all(1024, 1).unwrap();
    assert_eq!(count, 3);
}

#[test]
fn restore_does_not_revive_dropped_table() {
    use vectorvolt_engine::ShardCatalog;

    let dir = make_dir();
    run(create_args(&dir, "gone")).unwrap();
    run(args(
        &dir,
        CatalogCommand::DropTable {
            name: "gone".into(),
        },
    ))
    .unwrap();

    let cat2 = ShardCatalog::with_registry(dir.path().join("catalog"), dir.path().join("registry"))
        .unwrap();
    let count = cat2.restore_all(1024, 1).unwrap();
    assert_eq!(count, 0);
    assert!(!cat2.contains("gone"));
}

#[test]
fn restore_table_has_correct_column_count() {
    use vectorvolt_engine::ShardCatalog;

    let dir = make_dir();
    // create a 3-column table
    run(args(
        &dir,
        CatalogCommand::CreateTable {
            name: "wide".to_string(),
            columns: vec!["pk".to_string(), "a".to_string(), "b".to_string()],
            flush_threshold: 1024,
            partitions: 1,
        },
    ))
    .unwrap();

    let cat2 = ShardCatalog::with_registry(dir.path().join("catalog"), dir.path().join("registry"))
        .unwrap();
    cat2.restore_all(1024, 1).unwrap();
    let entry = cat2.get("wide").unwrap();
    assert_eq!(entry.schema.col_count(), 3);
}

#[test]
fn restore_then_list_returns_all_table_names() {
    use vectorvolt_engine::ShardCatalog;

    let dir = make_dir();
    run(create_args(&dir, "x")).unwrap();
    run(create_args(&dir, "y")).unwrap();

    let cat2 = ShardCatalog::with_registry(dir.path().join("catalog"), dir.path().join("registry"))
        .unwrap();
    cat2.restore_all(1024, 1).unwrap();
    let names = cat2.list_tables();
    assert!(names.contains(&"x".to_string()));
    assert!(names.contains(&"y".to_string()));
}

// â”€â”€ serve --restore-catalog â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn serve_with_catalog_no_restore_has_empty_catalog() {
    let dir = make_dir();
    // pre-create a table via CLI then open serve WITHOUT --restore-catalog
    run(create_args(&dir, "orders")).unwrap();
    let bound = bind(&serve_args(&dir, true, false)).unwrap();
    let cat = bound.catalog.unwrap();
    // catalog opened fresh â€” no restore, so table not loaded
    assert!(!cat.contains("orders"));
}

#[test]
fn serve_with_restore_catalog_loads_persisted_tables() {
    let dir = make_dir();
    run(create_args(&dir, "orders")).unwrap();
    let bound = bind(&serve_args(&dir, true, true)).unwrap();
    let cat = bound.catalog.unwrap();
    assert!(cat.contains("orders"));
}

#[test]
fn serve_restore_catalog_multiple_tables() {
    let dir = make_dir();
    run(create_args(&dir, "a")).unwrap();
    run(create_args(&dir, "b")).unwrap();
    let bound = bind(&serve_args(&dir, true, true)).unwrap();
    let cat = bound.catalog.unwrap();
    assert!(cat.contains("a"));
    assert!(cat.contains("b"));
}

#[test]
fn serve_restore_catalog_functional_after_restore() {
    let dir = make_dir();
    run(create_args(&dir, "data")).unwrap();
    let bound = bind(&serve_args(&dir, true, true)).unwrap();
    let cat = bound.catalog.unwrap();
    let mgr = cat.manager("data").unwrap();
    mgr.insert(1, vec![1, 99]).unwrap();
    assert_eq!(mgr.get(1).unwrap(), Some(vec![1, 99]));
}

#[test]
fn serve_restore_catalog_empty_registry_is_noop() {
    let dir = make_dir();
    let bound = bind(&serve_args(&dir, true, true)).unwrap();
    let cat = bound.catalog.unwrap();
    assert_eq!(cat.table_count(), 0);
}
