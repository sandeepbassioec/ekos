//! Integration tests for L122: `vvctl catalog config-*` (VV-CFG-010).

use std::path::PathBuf;

use tempfile::TempDir;
use vectorvolt_engine::{ThreadsConfig, VvConfig};
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

fn config_path(dir: &TempDir) -> PathBuf {
    dir.path().join("vectorvolt.yaml")
}

fn write_config(dir: &TempDir, yaml: &str) -> PathBuf {
    let p = config_path(dir);
    std::fs::write(&p, yaml).unwrap();
    p
}

// ── CLI integration ───────────────────────────────────────────────────────────

#[test]
fn cli_config_show_prints_yaml() {
    let dir = make_dir();
    write_config(&dir, "threads:\n  writer_count: 4\n");
    run(args(
        &dir,
        CatalogCommand::ConfigShow {
            file: config_path(&dir),
        },
    ))
    .unwrap();
}

#[test]
fn cli_config_show_missing_file_errors() {
    let dir = make_dir();
    let result = run(args(
        &dir,
        CatalogCommand::ConfigShow {
            file: dir.path().join("missing.yaml"),
        },
    ));
    assert!(result.is_err());
}

#[test]
fn cli_config_validate_clean_exits_ok() {
    let dir = make_dir();
    write_config(&dir, "storage:\n  compaction_strategy: leveled\n");
    run(args(
        &dir,
        CatalogCommand::ConfigValidate {
            file: config_path(&dir),
        },
    ))
    .unwrap();
}

#[test]
fn cli_config_validate_bad_value_errors() {
    let dir = make_dir();
    write_config(&dir, "hal:\n  huge_pages: 4gb\n");
    let result = run(args(
        &dir,
        CatalogCommand::ConfigValidate {
            file: config_path(&dir),
        },
    ));
    assert!(result.is_err());
}

#[test]
fn cli_config_set_creates_file() {
    let dir = make_dir();
    run(args(
        &dir,
        CatalogCommand::ConfigSet {
            file: config_path(&dir),
            key: "threads.writer_count".into(),
            value: "8".into(),
        },
    ))
    .unwrap();
    assert!(config_path(&dir).exists());
}

#[test]
fn cli_config_set_updates_existing() {
    let dir = make_dir();
    write_config(&dir, "threads:\n  writer_count: 2\n");
    run(args(
        &dir,
        CatalogCommand::ConfigSet {
            file: config_path(&dir),
            key: "threads.writer_count".into(),
            value: "16".into(),
        },
    ))
    .unwrap();
    let cfg = VvConfig::from_file(config_path(&dir)).unwrap();
    assert_eq!(cfg.threads.writer_count, Some(16));
}

#[test]
fn cli_config_set_unknown_key_errors() {
    let dir = make_dir();
    let result = run(args(
        &dir,
        CatalogCommand::ConfigSet {
            file: config_path(&dir),
            key: "bogus.field".into(),
            value: "x".into(),
        },
    ));
    assert!(result.is_err());
}

// ── engine API ────────────────────────────────────────────────────────────────

#[test]
fn engine_empty_yaml_default() {
    let cfg = VvConfig::from_yaml("{}").unwrap();
    assert_eq!(cfg, VvConfig::default());
}

#[test]
fn engine_round_trip_yaml() {
    let mut cfg = VvConfig::default();
    cfg.threads.writer_count = Some(4);
    cfg.network.bind_addr = Some("0.0.0.0:50051".into());
    let yaml = cfg.to_yaml().unwrap();
    let cfg2 = VvConfig::from_yaml(&yaml).unwrap();
    assert_eq!(cfg, cfg2);
}

#[test]
fn engine_set_key_numeric() {
    let mut cfg = VvConfig::default();
    cfg.set_key("threads.writer_count", "12").unwrap();
    assert_eq!(cfg.threads.writer_count, Some(12));
}

#[test]
fn engine_set_key_string() {
    let mut cfg = VvConfig::default();
    cfg.set_key("storage.compaction_strategy", "tiered")
        .unwrap();
    assert_eq!(cfg.storage.compaction_strategy.as_deref(), Some("tiered"));
}

#[test]
fn engine_set_key_unknown_errors() {
    let mut cfg = VvConfig::default();
    let err = cfg.set_key("unknown.key", "v").unwrap_err();
    assert!(err.to_string().contains("unknown.key"));
}

#[test]
fn engine_validate_warns_on_bad_trace_level() {
    let mut cfg = VvConfig::default();
    cfg.observability.trace_level = Some("verbose".into());
    let warns = cfg.validate();
    assert!(!warns.is_empty());
    assert!(warns.iter().any(|w| w.contains("trace_level")));
}

#[test]
fn engine_threads_config_partial_yaml() {
    let yaml = "threads:\n  writer_count: 4\n  writer_priority: realtime\n";
    let cfg = VvConfig::from_yaml(yaml).unwrap();
    assert_eq!(
        cfg.threads,
        ThreadsConfig {
            writer_count: Some(4),
            writer_priority: Some("realtime".into()),
            ring_buffer_size: None,
        }
    );
}
