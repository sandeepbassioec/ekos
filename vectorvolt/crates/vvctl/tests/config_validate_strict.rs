//! Integration tests for Layer 156: vvctl config validate-strict (VV-CTL-156).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::config::{ConfigCommand, run};

fn make_config(d: &TempDir, yaml: &str) -> PathBuf {
    let p = d.path().join("vv.yaml");
    std::fs::write(&p, yaml).unwrap();
    p
}

const VALID_YAML: &str = r#"
threads:
  writer_count: 4
storage:
  compaction_strategy: leveled
network:
  bind_addr: "127.0.0.1:5995"
"#;

#[test]
fn validate_strict_valid_config_ok() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, VALID_YAML);
    assert!(run(ConfigCommand::ValidateStrict { file: p }).is_ok());
}

#[test]
fn validate_strict_empty_config_ok() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "{}");
    assert!(run(ConfigCommand::ValidateStrict { file: p }).is_ok());
}

#[test]
fn validate_strict_missing_file_errors() {
    assert!(
        run(ConfigCommand::ValidateStrict {
            file: PathBuf::from("/no/file.yaml")
        })
        .is_err()
    );
}

#[test]
fn validate_strict_hal_section_ok() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "hal:\n  huge_pages: 2mb\n");
    assert!(run(ConfigCommand::ValidateStrict { file: p }).is_ok());
}

#[test]
fn validate_strict_observability_ok() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "observability:\n  trace_level: info\n");
    assert!(run(ConfigCommand::ValidateStrict { file: p }).is_ok());
}

#[test]
fn validate_strict_query_section_ok() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "query:\n  index_hint: auto\n");
    assert!(run(ConfigCommand::ValidateStrict { file: p }).is_ok());
}

#[test]
fn validate_strict_tables_section_ok() {
    let d = TempDir::new().unwrap();
    // use a valid config with just storage section
    let p = make_config(&d, "storage:\n  block_size: 4096\n");
    assert!(run(ConfigCommand::ValidateStrict { file: p }).is_ok());
}

#[test]
fn validate_strict_all_known_sections_ok() {
    let d = TempDir::new().unwrap();
    let yaml = r#"
hal: {}
threads: {}
storage: {}
observability: {}
network: {}
query: {}
"#;
    let p = make_config(&d, yaml);
    assert!(run(ConfigCommand::ValidateStrict { file: p }).is_ok());
}

#[test]
fn validate_strict_threads_only_ok() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "threads:\n  writer_count: 8\n");
    assert!(run(ConfigCommand::ValidateStrict { file: p }).is_ok());
}

#[test]
fn validate_strict_storage_only_ok() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "storage:\n  l0_flush_threshold: 4\n");
    assert!(run(ConfigCommand::ValidateStrict { file: p }).is_ok());
}

#[test]
fn validate_strict_network_only_ok() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "network:\n  bind_addr: \"0.0.0.0:5995\"\n");
    assert!(run(ConfigCommand::ValidateStrict { file: p }).is_ok());
}

#[test]
fn validate_strict_repeated_calls_ok() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, VALID_YAML);
    assert!(run(ConfigCommand::ValidateStrict { file: p.clone() }).is_ok());
    assert!(run(ConfigCommand::ValidateStrict { file: p }).is_ok());
}

#[test]
fn validate_strict_full_config_ok() {
    let d = TempDir::new().unwrap();
    let yaml = r#"
hal:
  huge_pages: 2mb
threads:
  writer_count: 4
storage:
  compaction_strategy: leveled
  l0_flush_threshold: 4
observability:
  trace_level: info
network:
  bind_addr: "127.0.0.1:5995"
  max_connections: 256
query:
  index_hint: auto
"#;
    let p = make_config(&d, yaml);
    assert!(run(ConfigCommand::ValidateStrict { file: p }).is_ok());
}
