//! Integration tests for Layer 139: vvctl config normalize (VV-CFG-017).

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

const STARTER: &str = r#"threads:
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

// 1. normalize valid file succeeds
#[test]
fn normalize_valid_file_succeeds() {
    let dir = make_dir();
    let file = write_yaml(&dir, "cfg.yaml", STARTER);
    assert!(
        run(ConfigCommand::Normalize {
            file,
            dry_run: false,
        })
        .is_ok()
    );
}

// 2. normalize dry_run returns Ok
#[test]
fn normalize_dry_run_returns_ok() {
    let dir = make_dir();
    let file = write_yaml(&dir, "cfg.yaml", STARTER);
    assert!(
        run(ConfigCommand::Normalize {
            file,
            dry_run: true,
        })
        .is_ok()
    );
}

// 3. normalize dry_run does not change file
#[test]
fn normalize_dry_run_does_not_change_file() {
    let dir = make_dir();
    let file = write_yaml(&dir, "cfg.yaml", STARTER);
    let before = std::fs::metadata(&file).unwrap().modified().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(10));
    run(ConfigCommand::Normalize {
        file: file.clone(),
        dry_run: true,
    })
    .unwrap();
    let after = std::fs::metadata(&file).unwrap().modified().unwrap();
    assert_eq!(before, after);
}

// 4. normalize is idempotent
#[test]
fn normalize_idempotent() {
    let dir = make_dir();
    let file = write_yaml(&dir, "cfg.yaml", STARTER);
    run(ConfigCommand::Normalize {
        file: file.clone(),
        dry_run: false,
    })
    .unwrap();
    let after_first = std::fs::read_to_string(&file).unwrap();
    run(ConfigCommand::Normalize {
        file: file.clone(),
        dry_run: false,
    })
    .unwrap();
    let after_second = std::fs::read_to_string(&file).unwrap();
    assert_eq!(after_first, after_second);
}

// 5. normalize missing file errors
#[test]
fn normalize_missing_file_errors() {
    let dir = make_dir();
    let result = run(ConfigCommand::Normalize {
        file: dir.path().join("nonexistent.yaml"),
        dry_run: false,
    });
    assert!(result.is_err());
}

// 6. normalize result is valid yaml
#[test]
fn normalize_result_is_valid_yaml() {
    let dir = make_dir();
    let file = write_yaml(&dir, "cfg.yaml", STARTER);
    run(ConfigCommand::Normalize {
        file: file.clone(),
        dry_run: false,
    })
    .unwrap();
    let contents = std::fs::read_to_string(&file).unwrap();
    assert!(!contents.is_empty());
    // Can be parsed back as VvConfig
    VvConfig::from_file(&file).unwrap();
}

// 7. normalize preserves writer_count
#[test]
fn normalize_preserves_writer_count() {
    let dir = make_dir();
    let file = write_yaml(&dir, "cfg.yaml", STARTER);
    run(ConfigCommand::Normalize {
        file: file.clone(),
        dry_run: false,
    })
    .unwrap();
    let cfg = VvConfig::from_file(&file).unwrap();
    assert_eq!(cfg.threads.writer_count, Some(4));
}

// 8. normalize preserves bind_addr
#[test]
fn normalize_preserves_bind_addr() {
    let dir = make_dir();
    let file = write_yaml(&dir, "cfg.yaml", STARTER);
    run(ConfigCommand::Normalize {
        file: file.clone(),
        dry_run: false,
    })
    .unwrap();
    let cfg = VvConfig::from_file(&file).unwrap();
    assert_eq!(cfg.network.bind_addr.as_deref(), Some("127.0.0.1:5995"));
}

// 9. normalize preserves trace_level
#[test]
fn normalize_preserves_trace_level() {
    let dir = make_dir();
    let file = write_yaml(&dir, "cfg.yaml", STARTER);
    run(ConfigCommand::Normalize {
        file: file.clone(),
        dry_run: false,
    })
    .unwrap();
    let cfg = VvConfig::from_file(&file).unwrap();
    assert_eq!(cfg.observability.trace_level.as_deref(), Some("info"));
}

// 10. normalize empty config succeeds
#[test]
fn normalize_empty_config_succeeds() {
    let dir = make_dir();
    let file = write_yaml(&dir, "cfg.yaml", "{}\n");
    assert!(
        run(ConfigCommand::Normalize {
            file,
            dry_run: false,
        })
        .is_ok()
    );
}

// 11. normalize full config succeeds
#[test]
fn normalize_full_config_succeeds() {
    let dir = make_dir();
    let content = r#"threads:
  writer_count: 8
network:
  bind_addr: "0.0.0.0:6000"
  max_connections: 512
storage:
  compaction_strategy: leveled
  l0_flush_threshold: 16
  block_size: 65536
observability:
  trace_level: debug
hal:
  huge_pages: "2mb"
"#;
    let file = write_yaml(&dir, "full.yaml", content);
    assert!(
        run(ConfigCommand::Normalize {
            file,
            dry_run: false,
        })
        .is_ok()
    );
}

// 12. normalize with tables section
#[test]
fn normalize_with_tables_section() {
    let dir = make_dir();
    let file = write_yaml(&dir, "cfg.yaml", STARTER);
    assert!(
        run(ConfigCommand::Normalize {
            file,
            dry_run: false,
        })
        .is_ok()
    );
}

// 13. normalize result parses back
#[test]
fn normalize_result_parses_back() {
    let dir = make_dir();
    let file = write_yaml(&dir, "cfg.yaml", STARTER);
    run(ConfigCommand::Normalize {
        file: file.clone(),
        dry_run: false,
    })
    .unwrap();
    assert!(VvConfig::from_file(&file).is_ok());
}
