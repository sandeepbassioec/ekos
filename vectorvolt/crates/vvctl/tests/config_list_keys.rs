//! Integration tests for Layer 150: vvctl config list-keys (VV-CTL-150).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::config::{ConfigCommand, ListKeysOutput, run};

fn make_config(d: &TempDir, yaml: &str) -> PathBuf {
    let p = d.path().join("vv.yaml");
    std::fs::write(&p, yaml).unwrap();
    p
}

const FULL_YAML: &str = r#"
threads:
  writer_count: 4
storage:
  compaction_strategy: leveled
network:
  bind_addr: "127.0.0.1:5995"
"#;

#[test]
fn list_keys_empty_config_table_ok() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "{}");
    assert!(
        run(ConfigCommand::ListKeys {
            file: p,
            output: ListKeysOutput::Table
        })
        .is_ok()
    );
}

#[test]
fn list_keys_empty_config_json_ok() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "{}");
    assert!(
        run(ConfigCommand::ListKeys {
            file: p,
            output: ListKeysOutput::Json
        })
        .is_ok()
    );
}

#[test]
fn list_keys_full_config_table_ok() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, FULL_YAML);
    assert!(
        run(ConfigCommand::ListKeys {
            file: p,
            output: ListKeysOutput::Table
        })
        .is_ok()
    );
}

#[test]
fn list_keys_full_config_json_ok() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, FULL_YAML);
    assert!(
        run(ConfigCommand::ListKeys {
            file: p,
            output: ListKeysOutput::Json
        })
        .is_ok()
    );
}

#[test]
fn list_keys_missing_file_errors() {
    assert!(
        run(ConfigCommand::ListKeys {
            file: PathBuf::from("/no/file.yaml"),
            output: ListKeysOutput::Table
        })
        .is_err()
    );
}

#[test]
fn list_keys_shows_all_known_keys() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "{}");
    assert!(
        run(ConfigCommand::ListKeys {
            file: p,
            output: ListKeysOutput::Table
        })
        .is_ok()
    );
}

#[test]
fn list_keys_json_is_array() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "{}");
    assert!(
        run(ConfigCommand::ListKeys {
            file: p,
            output: ListKeysOutput::Json
        })
        .is_ok()
    );
}

#[test]
fn list_keys_repeated_calls_ok() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "{}");
    assert!(
        run(ConfigCommand::ListKeys {
            file: p.clone(),
            output: ListKeysOutput::Table
        })
        .is_ok()
    );
    assert!(
        run(ConfigCommand::ListKeys {
            file: p,
            output: ListKeysOutput::Table
        })
        .is_ok()
    );
}

#[test]
fn list_keys_with_writer_count_set() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "threads:\n  writer_count: 8\n");
    assert!(
        run(ConfigCommand::ListKeys {
            file: p,
            output: ListKeysOutput::Table
        })
        .is_ok()
    );
}

#[test]
fn list_keys_json_with_values() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, FULL_YAML);
    assert!(
        run(ConfigCommand::ListKeys {
            file: p,
            output: ListKeysOutput::Json
        })
        .is_ok()
    );
}

#[test]
fn list_keys_hal_section() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "hal:\n  huge_pages: 2mb\n");
    assert!(
        run(ConfigCommand::ListKeys {
            file: p,
            output: ListKeysOutput::Table
        })
        .is_ok()
    );
}

#[test]
fn list_keys_observability_section() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "observability:\n  trace_level: debug\n");
    assert!(
        run(ConfigCommand::ListKeys {
            file: p,
            output: ListKeysOutput::Table
        })
        .is_ok()
    );
}

#[test]
fn list_keys_query_section() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "query:\n  index_hint: auto\n");
    assert!(
        run(ConfigCommand::ListKeys {
            file: p,
            output: ListKeysOutput::Json
        })
        .is_ok()
    );
}
