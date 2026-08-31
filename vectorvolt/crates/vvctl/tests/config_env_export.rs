//! Integration tests for Layer 157: vvctl config env-export (VV-CTL-157).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::config::{ConfigCommand, run};

fn make_config(d: &TempDir, yaml: &str) -> PathBuf {
    let p = d.path().join("vv.yaml");
    std::fs::write(&p, yaml).unwrap();
    p
}

#[test]
fn env_export_empty_config_ok() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "{}");
    assert!(run(ConfigCommand::EnvExport { file: p }).is_ok());
}

#[test]
fn env_export_full_config_ok() {
    let d = TempDir::new().unwrap();
    let yaml = r#"
threads:
  writer_count: 4
storage:
  compaction_strategy: leveled
  l0_flush_threshold: 4
network:
  bind_addr: "127.0.0.1:5995"
  max_connections: 256
observability:
  trace_level: info
query:
  index_hint: auto
"#;
    let p = make_config(&d, yaml);
    assert!(run(ConfigCommand::EnvExport { file: p }).is_ok());
}

#[test]
fn env_export_missing_file_errors() {
    assert!(
        run(ConfigCommand::EnvExport {
            file: PathBuf::from("/no/file.yaml")
        })
        .is_err()
    );
}

#[test]
fn env_export_bind_addr_only_ok() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "network:\n  bind_addr: \"0.0.0.0:5995\"\n");
    assert!(run(ConfigCommand::EnvExport { file: p }).is_ok());
}

#[test]
fn env_export_trace_level_only_ok() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "observability:\n  trace_level: debug\n");
    assert!(run(ConfigCommand::EnvExport { file: p }).is_ok());
}

#[test]
fn env_export_writer_count_only_ok() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "threads:\n  writer_count: 8\n");
    assert!(run(ConfigCommand::EnvExport { file: p }).is_ok());
}

#[test]
fn env_export_flush_threshold_ok() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "storage:\n  l0_flush_threshold: 16\n");
    assert!(run(ConfigCommand::EnvExport { file: p }).is_ok());
}

#[test]
fn env_export_repeated_calls_ok() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "{}");
    assert!(run(ConfigCommand::EnvExport { file: p.clone() }).is_ok());
    assert!(run(ConfigCommand::EnvExport { file: p }).is_ok());
}

#[test]
fn env_export_hal_huge_pages_ok() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "hal:\n  huge_pages: 2mb\n");
    assert!(run(ConfigCommand::EnvExport { file: p }).is_ok());
}

#[test]
fn env_export_index_hint_ok() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "query:\n  index_hint: auto\n");
    assert!(run(ConfigCommand::EnvExport { file: p }).is_ok());
}

#[test]
fn env_export_max_connections_ok() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "network:\n  max_connections: 512\n");
    assert!(run(ConfigCommand::EnvExport { file: p }).is_ok());
}

#[test]
fn env_export_block_size_ok() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "storage:\n  block_size: 65536\n");
    assert!(run(ConfigCommand::EnvExport { file: p }).is_ok());
}

#[test]
fn env_export_compaction_strategy_ok() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "storage:\n  compaction_strategy: leveled\n");
    assert!(run(ConfigCommand::EnvExport { file: p }).is_ok());
}
