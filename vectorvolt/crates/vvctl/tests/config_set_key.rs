//! Integration tests for Layer 134: vvctl config set-key (VV-CFG-016).

use std::path::PathBuf;

use tempfile::TempDir;
use vectorvolt_engine::VvConfig;
use vvctl::commands::config::{ConfigCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}

fn write_yaml(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, content).unwrap();
    path
}

const STARTER: &str = r#"
threads:
  writer_count: 4
network:
  bind_addr: "127.0.0.1:5995"
  max_connections: 128
storage:
  compaction_strategy: leveled
  l0_flush_threshold: 8
observability:
  trace_level: info
"#;

// â”€â”€ happy path â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn set_key_writer_count_succeeds() {
    let dir = make_dir();
    let file = write_yaml(&dir, "cfg.yaml", STARTER);
    run(ConfigCommand::SetKey {
        file: file.clone(),
        key: "threads.writer_count".into(),
        value: "8".into(),
        dry_run: false,
    })
    .unwrap();
    let contents = std::fs::read_to_string(&file).unwrap();
    assert!(contents.contains("writer_count: 8"));
}

#[test]
fn set_key_bind_addr_succeeds() {
    let dir = make_dir();
    let file = write_yaml(&dir, "cfg.yaml", STARTER);
    run(ConfigCommand::SetKey {
        file: file.clone(),
        key: "network.bind_addr".into(),
        value: "0.0.0.0:9000".into(),
        dry_run: false,
    })
    .unwrap();
    let contents = std::fs::read_to_string(&file).unwrap();
    assert!(contents.contains("0.0.0.0:9000"));
}

#[test]
fn set_key_dry_run_does_not_write() {
    let dir = make_dir();
    let file = write_yaml(&dir, "cfg.yaml", STARTER);
    let original = std::fs::read_to_string(&file).unwrap();
    run(ConfigCommand::SetKey {
        file: file.clone(),
        key: "threads.writer_count".into(),
        value: "16".into(),
        dry_run: true,
    })
    .unwrap();
    let after = std::fs::read_to_string(&file).unwrap();
    assert_eq!(original, after, "dry_run must not modify the file");
}

#[test]
fn set_key_dry_run_still_succeeds() {
    let dir = make_dir();
    let file = write_yaml(&dir, "cfg.yaml", STARTER);
    let result = run(ConfigCommand::SetKey {
        file: file.clone(),
        key: "threads.writer_count".into(),
        value: "2".into(),
        dry_run: true,
    });
    assert!(result.is_ok());
}

#[test]
fn set_key_roundtrip_readable() {
    let dir = make_dir();
    let file = write_yaml(&dir, "cfg.yaml", STARTER);
    run(ConfigCommand::SetKey {
        file: file.clone(),
        key: "threads.writer_count".into(),
        value: "12".into(),
        dry_run: false,
    })
    .unwrap();
    let cfg = VvConfig::from_file(&file).unwrap();
    assert_eq!(cfg.threads.writer_count, Some(12));
}

#[test]
fn set_key_trace_level_succeeds() {
    let dir = make_dir();
    let file = write_yaml(&dir, "cfg.yaml", STARTER);
    run(ConfigCommand::SetKey {
        file: file.clone(),
        key: "observability.trace_level".into(),
        value: "debug".into(),
        dry_run: false,
    })
    .unwrap();
    let contents = std::fs::read_to_string(&file).unwrap();
    assert!(contents.contains("debug"));
}

#[test]
fn set_key_compaction_strategy_succeeds() {
    let dir = make_dir();
    let file = write_yaml(&dir, "cfg.yaml", STARTER);
    run(ConfigCommand::SetKey {
        file: file.clone(),
        key: "storage.compaction_strategy".into(),
        value: "tiered".into(),
        dry_run: false,
    })
    .unwrap();
    let contents = std::fs::read_to_string(&file).unwrap();
    assert!(contents.contains("tiered"));
}

#[test]
fn set_key_max_connections_succeeds() {
    let dir = make_dir();
    let file = write_yaml(&dir, "cfg.yaml", STARTER);
    run(ConfigCommand::SetKey {
        file: file.clone(),
        key: "network.max_connections".into(),
        value: "512".into(),
        dry_run: false,
    })
    .unwrap();
    let cfg = VvConfig::from_file(&file).unwrap();
    assert_eq!(cfg.network.max_connections, Some(512));
}

#[test]
fn set_key_l0_flush_threshold_succeeds() {
    let dir = make_dir();
    let file = write_yaml(&dir, "cfg.yaml", STARTER);
    run(ConfigCommand::SetKey {
        file: file.clone(),
        key: "storage.l0_flush_threshold".into(),
        value: "16".into(),
        dry_run: false,
    })
    .unwrap();
    let cfg = VvConfig::from_file(&file).unwrap();
    assert_eq!(cfg.storage.l0_flush_threshold, Some(16));
}

#[test]
fn set_key_multiple_keys_sequential() {
    let dir = make_dir();
    let file = write_yaml(&dir, "cfg.yaml", STARTER);
    run(ConfigCommand::SetKey {
        file: file.clone(),
        key: "threads.writer_count".into(),
        value: "6".into(),
        dry_run: false,
    })
    .unwrap();
    run(ConfigCommand::SetKey {
        file: file.clone(),
        key: "network.max_connections".into(),
        value: "256".into(),
        dry_run: false,
    })
    .unwrap();
    let cfg = VvConfig::from_file(&file).unwrap();
    assert_eq!(cfg.threads.writer_count, Some(6));
    assert_eq!(cfg.network.max_connections, Some(256));
}

// â”€â”€ error cases â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn set_key_unknown_key_errors() {
    let dir = make_dir();
    let file = write_yaml(&dir, "cfg.yaml", STARTER);
    let result = run(ConfigCommand::SetKey {
        file,
        key: "bogus.nonexistent_key".into(),
        value: "x".into(),
        dry_run: false,
    });
    assert!(result.is_err());
}

#[test]
fn set_key_invalid_value_errors() {
    let dir = make_dir();
    let file = write_yaml(&dir, "cfg.yaml", STARTER);
    let result = run(ConfigCommand::SetKey {
        file,
        key: "threads.writer_count".into(),
        value: "not_a_number".into(),
        dry_run: false,
    });
    assert!(result.is_err());
}

#[test]
fn set_key_missing_file_errors() {
    let dir = make_dir();
    let file = dir.path().join("nonexistent.yaml");
    let result = run(ConfigCommand::SetKey {
        file,
        key: "threads.writer_count".into(),
        value: "4".into(),
        dry_run: false,
    });
    assert!(result.is_err());
}
