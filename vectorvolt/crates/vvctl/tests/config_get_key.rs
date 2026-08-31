//! Integration tests for Layer 149: vvctl config get-key (VV-CTL-149).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::config::{ConfigCommand, run};

fn make_config(d: &TempDir, yaml: &str) -> PathBuf {
    let p = d.path().join("vv.yaml");
    std::fs::write(&p, yaml).unwrap();
    p
}

const FULL_YAML: &str = r#"
threads:
  writer_count: 8
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

#[test]
fn get_key_writer_count() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, FULL_YAML);
    assert!(
        run(ConfigCommand::GetKey {
            file: p,
            key: "threads.writer_count".into()
        })
        .is_ok()
    );
}

#[test]
fn get_key_bind_addr() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, FULL_YAML);
    assert!(
        run(ConfigCommand::GetKey {
            file: p,
            key: "network.bind_addr".into()
        })
        .is_ok()
    );
}

#[test]
fn get_key_trace_level() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, FULL_YAML);
    assert!(
        run(ConfigCommand::GetKey {
            file: p,
            key: "observability.trace_level".into()
        })
        .is_ok()
    );
}

#[test]
fn get_key_index_hint() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, FULL_YAML);
    assert!(
        run(ConfigCommand::GetKey {
            file: p,
            key: "query.index_hint".into()
        })
        .is_ok()
    );
}

#[test]
fn get_key_unknown_errors() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, FULL_YAML);
    assert!(
        run(ConfigCommand::GetKey {
            file: p,
            key: "totally.unknown".into()
        })
        .is_err()
    );
}

#[test]
fn get_key_missing_file_errors() {
    assert!(
        run(ConfigCommand::GetKey {
            file: PathBuf::from("/no/file.yaml"),
            key: "threads.writer_count".into()
        })
        .is_err()
    );
}

#[test]
fn get_key_unset_field_ok() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "{}");
    assert!(
        run(ConfigCommand::GetKey {
            file: p,
            key: "hal.huge_pages".into()
        })
        .is_ok()
    );
}

#[test]
fn get_key_max_connections() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, FULL_YAML);
    assert!(
        run(ConfigCommand::GetKey {
            file: p,
            key: "network.max_connections".into()
        })
        .is_ok()
    );
}

#[test]
fn get_key_l0_flush_threshold() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, FULL_YAML);
    assert!(
        run(ConfigCommand::GetKey {
            file: p,
            key: "storage.l0_flush_threshold".into()
        })
        .is_ok()
    );
}

#[test]
fn get_key_compaction_strategy() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, FULL_YAML);
    assert!(
        run(ConfigCommand::GetKey {
            file: p,
            key: "storage.compaction_strategy".into()
        })
        .is_ok()
    );
}

#[test]
fn get_key_numa_node_unset() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "{}");
    assert!(
        run(ConfigCommand::GetKey {
            file: p,
            key: "hal.numa_node".into()
        })
        .is_ok()
    );
}

#[test]
fn get_key_writer_priority_unset() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "{}");
    assert!(
        run(ConfigCommand::GetKey {
            file: p,
            key: "threads.writer_priority".into()
        })
        .is_ok()
    );
}

#[test]
fn get_key_block_size() {
    let d = TempDir::new().unwrap();
    let p = make_config(&d, "storage:\n  block_size: 65536\n");
    assert!(
        run(ConfigCommand::GetKey {
            file: p,
            key: "storage.block_size".into()
        })
        .is_ok()
    );
}
