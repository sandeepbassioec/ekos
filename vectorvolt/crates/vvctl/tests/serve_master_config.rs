//! Integration tests for L123: `vvctl serve --master-config` (VV-CFG-011).
//!
//! Tests use `vvctl::commands::serve::bind()` to exercise the config
//! loading path without starting a live server.

use std::net::SocketAddr;
use std::path::PathBuf;

use tempfile::TempDir;
use vvctl::commands::serve::{ServeArgs, bind};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}

fn base_args(dir: &TempDir) -> ServeArgs {
    ServeArgs {
        dir: dir.path().join("data"),
        columns: vec!["pk".to_string()],
        addr: "127.0.0.1:0".parse::<SocketAddr>().unwrap(),
        max_connections: 64,
        secret: None,
        key_id: "default".into(),
        key_scope: None,
        key_file: None,
        cert: None,
        key: None,
        multi: false,
        config: None,
        master_config: None,
        flush_threshold: None,
        durability: None,
        max_partitions: None,
        catalog: false,
        restore_catalog: false,
        http_addr: None,
        http_token: None,
        transport: "tcp".to_string(),
        xdp_iface: "lo".to_string(),
        xdp_queue: 0,
        xdp_ring_size: 256,
        grpc_port: 50051,
        no_grpc: false,
        grpc_host: std::net::IpAddr::from([127, 0, 0, 1]),
        grpc_token: None,
        tpm_unseal: false,
        encrypt_at_rest: false,
        metrics_port: 0,
        metrics_host: std::net::IpAddr::from([127, 0, 0, 1]),
        pgwire_port: None,
        cluster_bind: None,
        cluster_seeds: vec![],
        cluster_node_id: None,
        cluster_status_port: 7900,
    }
}

fn write_master_config(dir: &TempDir, yaml: &str) -> PathBuf {
    let p = dir.path().join("vectorvolt.yaml");
    std::fs::write(&p, yaml).unwrap();
    p
}

// â”€â”€ CLI / bind integration â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn bind_without_master_config_master_config_is_none() {
    let dir = make_dir();
    let args = base_args(&dir);
    let bound = bind(&args).unwrap();
    assert!(bound.master_config.is_none());
}

#[test]
fn bind_with_master_config_loaded() {
    let dir = make_dir();
    let cfg_path = write_master_config(&dir, "threads:\n  writer_count: 8\n");
    let mut args = base_args(&dir);
    args.master_config = Some(cfg_path);
    let bound = bind(&args).unwrap();
    assert!(bound.master_config.is_some());
    let mc = bound.master_config.unwrap();
    assert_eq!(mc.threads.writer_count, Some(8));
}

#[test]
fn master_config_missing_file_errors() {
    let dir = make_dir();
    let mut args = base_args(&dir);
    args.master_config = Some(dir.path().join("nonexistent.yaml"));
    let result = bind(&args);
    assert!(result.is_err());
}

#[test]
fn master_config_flush_threshold_applied() {
    let dir = make_dir();
    let cfg_path = write_master_config(&dir, "storage:\n  l0_flush_threshold: 512\n");
    let mut args = base_args(&dir);
    args.master_config = Some(cfg_path);
    let bound = bind(&args).unwrap();
    assert_eq!(bound.engine_opts_flush_threshold, 512);
}

#[test]
fn cli_flush_threshold_overrides_master_config() {
    let dir = make_dir();
    let cfg_path = write_master_config(&dir, "storage:\n  l0_flush_threshold: 512\n");
    let mut args = base_args(&dir);
    args.master_config = Some(cfg_path);
    args.flush_threshold = Some(1024);
    let bound = bind(&args).unwrap();
    assert_eq!(bound.engine_opts_flush_threshold, 1024);
}

#[test]
fn master_config_writer_count_used_as_partition_count() {
    let dir = make_dir();
    let cfg_path = write_master_config(&dir, "threads:\n  writer_count: 6\n");
    let mut args = base_args(&dir);
    args.master_config = Some(cfg_path);
    let bound = bind(&args).unwrap();
    let mc = bound.master_config.unwrap();
    assert_eq!(mc.threads.writer_count, Some(6));
}

#[test]
fn cli_max_partitions_overrides_master_config_writer_count() {
    let dir = make_dir();
    let cfg_path = write_master_config(&dir, "threads:\n  writer_count: 6\n");
    let mut args = base_args(&dir);
    args.master_config = Some(cfg_path);
    args.max_partitions = Some(2);
    let bound = bind(&args).unwrap();
    assert!(bound.master_config.is_some());
}

#[test]
fn master_config_empty_sections_ok() {
    let dir = make_dir();
    let cfg_path = write_master_config(&dir, "{}\n");
    let mut args = base_args(&dir);
    args.master_config = Some(cfg_path);
    let bound = bind(&args).unwrap();
    assert!(bound.master_config.is_some());
}

#[test]
fn master_config_invalid_yaml_errors() {
    let dir = make_dir();
    let p = dir.path().join("bad.yaml");
    std::fs::write(&p, "threads: [\nunclosed bracket\n").unwrap();
    let mut args = base_args(&dir);
    args.master_config = Some(p);
    let result = bind(&args);
    assert!(result.is_err());
}

// â”€â”€ engine API (VvConfig itself) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn vv_config_network_section_round_trips() {
    use vectorvolt_engine::VvConfig;
    let yaml = "network:\n  bind_addr: \"0.0.0.0:5995\"\n  max_connections: 128\n";
    let cfg = VvConfig::from_yaml(yaml).unwrap();
    assert_eq!(cfg.network.bind_addr.as_deref(), Some("0.0.0.0:5995"));
    assert_eq!(cfg.network.max_connections, Some(128));
}

#[test]
fn vv_config_tables_section_round_trips() {
    use vectorvolt_engine::VvConfig;
    let yaml = "tables:\n  orders:\n    flush_threshold: 524288\n    durability: fsync\n";
    let cfg = VvConfig::from_yaml(yaml).unwrap();
    assert!(cfg.tables.contains_key("orders"));
    let t = &cfg.tables["orders"];
    assert_eq!(t.flush_threshold, Some(524288));
    assert_eq!(t.durability.as_deref(), Some("fsync"));
}

#[test]
fn vv_config_validate_all_good() {
    use vectorvolt_engine::VvConfig;
    let mut cfg = VvConfig::default();
    cfg.hal.huge_pages = Some("2mb".into());
    cfg.threads.writer_priority = Some("realtime".into());
    cfg.storage.compaction_strategy = Some("leveled".into());
    cfg.observability.trace_level = Some("info".into());
    cfg.query.index_hint = Some("auto".into());
    assert!(cfg.validate().is_empty());
}

#[test]
fn vv_config_validate_multiple_bad_values_reported() {
    use vectorvolt_engine::VvConfig;
    let mut cfg = VvConfig::default();
    cfg.hal.huge_pages = Some("4gb".into());
    cfg.storage.compaction_strategy = Some("lsm".into());
    let warns = cfg.validate();
    assert_eq!(warns.len(), 2);
}
