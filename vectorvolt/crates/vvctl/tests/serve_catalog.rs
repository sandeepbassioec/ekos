//! Integration tests for L105: vvctl serve wired to TxnManager + ShardCatalog (VV-CTL-001).
//!
//! Tests use `bind()` with port 0 (OS-assigned) so they never conflict.
//! The `run()` path (blocking event loop) is not tested here â€” CI covers that via smoke tests.

use std::net::SocketAddr;

use tempfile::TempDir;
use vectorvolt_engine::{ColumnDef, TableSchema};
use vvctl::commands::serve::{ServeArgs, bind, cli_overrides_from_args};

// â”€â”€ Helpers â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

fn any_port() -> SocketAddr {
    "127.0.0.1:0".parse().unwrap()
}

fn schema_two_cols() -> TableSchema {
    TableSchema::new(vec![
        ColumnDef::from_line("id:u64"),
        ColumnDef::from_line("val:u64"),
    ])
    .unwrap()
}

fn single_table_args(dir: &TempDir) -> ServeArgs {
    ServeArgs {
        dir: dir.path().to_path_buf(),
        columns: vec!["id".into(), "val".into()],
        addr: any_port(),
        max_connections: 4,
        secret: None,
        key_id: "default".into(),
        cert: None,
        key: None,
        multi: false,
        config: None,
        flush_threshold: None,
        durability: None,
        max_partitions: None,
        catalog: false,
        restore_catalog: false,
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

fn multi_args(dir: &TempDir) -> ServeArgs {
    ServeArgs {
        multi: true,
        ..single_table_args(dir)
    }
}

// â”€â”€ cli_overrides_from_args tests â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn cli_overrides_none_when_args_empty() {
    let dir = TempDir::new().unwrap();
    let args = single_table_args(&dir);
    let o = cli_overrides_from_args(&args);
    assert!(o.flush_threshold.is_none());
    assert!(o.durability.is_none());
    assert!(o.max_partitions.is_none());
}

#[test]
fn cli_overrides_flush_threshold_populated() {
    let dir = TempDir::new().unwrap();
    let mut args = single_table_args(&dir);
    args.flush_threshold = Some(512);
    let o = cli_overrides_from_args(&args);
    assert_eq!(o.flush_threshold, Some(512));
}

#[test]
fn cli_overrides_durability_populated() {
    let dir = TempDir::new().unwrap();
    let mut args = single_table_args(&dir);
    args.durability = Some("write_only".into());
    let o = cli_overrides_from_args(&args);
    assert_eq!(o.durability.as_deref(), Some("write_only"));
}

#[test]
fn cli_overrides_max_partitions_populated() {
    let dir = TempDir::new().unwrap();
    let mut args = single_table_args(&dir);
    args.max_partitions = Some(8);
    let o = cli_overrides_from_args(&args);
    assert_eq!(o.max_partitions, Some(8));
}

#[test]
fn cli_overrides_all_fields_populated() {
    let dir = TempDir::new().unwrap();
    let mut args = single_table_args(&dir);
    args.flush_threshold = Some(1024);
    args.durability = Some("buffered".into());
    args.max_partitions = Some(16);
    let o = cli_overrides_from_args(&args);
    assert_eq!(o.flush_threshold, Some(1024));
    assert_eq!(o.durability.as_deref(), Some("buffered"));
    assert_eq!(o.max_partitions, Some(16));
}

// â”€â”€ bind() engine-opts tests â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn bind_flush_threshold_from_cli_wins() {
    let dir = TempDir::new().unwrap();
    let mut args = single_table_args(&dir);
    args.flush_threshold = Some(2048);
    let bound = bind(&args).unwrap();
    assert_eq!(bound.engine_opts_flush_threshold, 2048);
}

#[test]
fn bind_default_flush_threshold_from_profile() {
    let dir = TempDir::new().unwrap();
    let args = single_table_args(&dir);
    let bound = bind(&args).unwrap();
    // SystemProfile fills in a profile-derived default â€” just verify > 0.
    assert!(bound.engine_opts_flush_threshold > 0);
}

#[test]
fn bind_invalid_durability_errors() {
    let dir = TempDir::new().unwrap();
    let mut args = single_table_args(&dir);
    args.durability = Some("turbo".into());
    let result = bind(&args);
    assert!(result.is_err(), "invalid durability should be an error");
}

// â”€â”€ bind() catalog-mode tests â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn bind_catalog_flag_creates_catalog_in_bound_server() {
    let dir = TempDir::new().unwrap();
    let mut args = multi_args(&dir);
    args.catalog = true;
    let bound = bind(&args).unwrap();
    assert!(
        bound.catalog.is_some(),
        "catalog should be Some when --catalog is set"
    );
}

#[test]
fn bind_no_catalog_flag_leaves_catalog_none() {
    let dir = TempDir::new().unwrap();
    let args = multi_args(&dir);
    let bound = bind(&args).unwrap();
    assert!(bound.catalog.is_none());
}

#[test]
fn bind_catalog_dir_created_on_disk() {
    let dir = TempDir::new().unwrap();
    let mut args = multi_args(&dir);
    args.catalog = true;
    bind(&args).unwrap();
    assert!(dir.path().join("catalog").exists());
}

#[test]
fn catalog_create_and_list_table() {
    let dir = TempDir::new().unwrap();
    let mut args = multi_args(&dir);
    args.catalog = true;
    let bound = bind(&args).unwrap();
    let cat = bound.catalog.unwrap();

    cat.create_table("events", schema_two_cols(), 4096, 4)
        .unwrap();
    let tables = cat.list_tables();
    assert!(tables.contains(&"events".to_string()));
}

#[test]
fn catalog_manager_insert_and_get() {
    let dir = TempDir::new().unwrap();
    let mut args = multi_args(&dir);
    args.catalog = true;
    let bound = bind(&args).unwrap();
    let cat = bound.catalog.unwrap();

    cat.create_table("items", schema_two_cols(), 4096, 2)
        .unwrap();
    let mgr = cat.manager("items").unwrap();
    mgr.insert(42, vec![42, 100]).unwrap();
    let row = mgr.get(42).unwrap().expect("row should exist");
    assert_eq!(row[0], 42);
    assert_eq!(row[1], 100);
}

#[test]
fn catalog_multiple_tables_in_one_session() {
    let dir = TempDir::new().unwrap();
    let mut args = multi_args(&dir);
    args.catalog = true;
    let bound = bind(&args).unwrap();
    let cat = bound.catalog.unwrap();

    let schema = schema_two_cols();
    cat.create_table("alpha", schema.clone(), 1024, 2).unwrap();
    cat.create_table("beta", schema.clone(), 1024, 2).unwrap();
    cat.create_table("gamma", schema.clone(), 1024, 2).unwrap();

    assert_eq!(cat.table_count(), 3);
    let names = cat.list_tables();
    assert!(names.contains(&"alpha".to_string()));
    assert!(names.contains(&"beta".to_string()));
    assert!(names.contains(&"gamma".to_string()));
}

#[test]
fn catalog_drop_table_removes_from_catalog() {
    let dir = TempDir::new().unwrap();
    let mut args = multi_args(&dir);
    args.catalog = true;
    let bound = bind(&args).unwrap();
    let cat = bound.catalog.unwrap();

    cat.create_table("tmp", schema_two_cols(), 1024, 2).unwrap();
    assert!(cat.contains("tmp"));
    cat.drop_table("tmp").unwrap();
    assert!(!cat.contains("tmp"));
}

#[test]
fn bind_single_table_uses_cli_flush_threshold() {
    let dir = TempDir::new().unwrap();
    let mut args = single_table_args(&dir);
    args.flush_threshold = Some(999);
    let bound = bind(&args).unwrap();
    assert_eq!(bound.engine_opts_flush_threshold, 999);
    // engine is present in single-table mode
    assert!(bound.engine.is_some());
}
