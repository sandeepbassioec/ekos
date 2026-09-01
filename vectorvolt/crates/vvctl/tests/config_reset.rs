//! Layer 184 — ConfigCommand::Reset (overwrite with defaults)

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::config::{ConfigCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}

fn cfg_path(dir: &TempDir) -> PathBuf {
    dir.path().join("vectorvolt.yaml")
}

fn init_config(dir: &TempDir) {
    run(ConfigCommand::Init {
        output: cfg_path(dir),
        force: false,
    })
    .unwrap();
}

// 1. reset creates file if absent
#[test]
fn reset_creates_file_if_absent() {
    let dir = make_dir();
    run(ConfigCommand::Reset {
        file: cfg_path(&dir),
    })
    .unwrap();
    assert!(cfg_path(&dir).exists());
}

// 2. reset overwrites existing file
#[test]
fn reset_overwrites_existing_file() {
    let dir = make_dir();
    init_config(&dir);
    assert!(
        run(ConfigCommand::Reset {
            file: cfg_path(&dir)
        })
        .is_ok()
    );
}

// 3. file is non-empty after reset
#[test]
fn reset_file_nonempty() {
    let dir = make_dir();
    run(ConfigCommand::Reset {
        file: cfg_path(&dir),
    })
    .unwrap();
    let content = std::fs::read_to_string(cfg_path(&dir)).unwrap();
    assert!(!content.trim().is_empty());
}

// 4. file contains threads key after reset
#[test]
fn reset_file_has_threads() {
    let dir = make_dir();
    run(ConfigCommand::Reset {
        file: cfg_path(&dir),
    })
    .unwrap();
    let content = std::fs::read_to_string(cfg_path(&dir)).unwrap();
    assert!(content.contains("threads"), "expected 'threads' in config");
}

// 5. reset twice returns Ok
#[test]
fn reset_twice_ok() {
    let dir = make_dir();
    run(ConfigCommand::Reset {
        file: cfg_path(&dir),
    })
    .unwrap();
    assert!(
        run(ConfigCommand::Reset {
            file: cfg_path(&dir)
        })
        .is_ok()
    );
}

// 6. config show after reset returns Ok
#[test]
fn show_after_reset_ok() {
    let dir = make_dir();
    run(ConfigCommand::Reset {
        file: cfg_path(&dir),
    })
    .unwrap();
    assert!(
        run(ConfigCommand::Show {
            file: cfg_path(&dir)
        })
        .is_ok()
    );
}

// 7. config validate after reset returns Ok
#[test]
fn validate_after_reset_ok() {
    let dir = make_dir();
    run(ConfigCommand::Reset {
        file: cfg_path(&dir),
    })
    .unwrap();
    assert!(
        run(ConfigCommand::Validate {
            file: cfg_path(&dir)
        })
        .is_ok()
    );
}

// 8. config init force=false fails on existing file
#[test]
fn init_force_false_fails_existing() {
    let dir = make_dir();
    init_config(&dir);
    assert!(
        run(ConfigCommand::Init {
            output: cfg_path(&dir),
            force: false
        })
        .is_err()
    );
}

// 9. config init force=true overwrites
#[test]
fn init_force_true_overwrites() {
    let dir = make_dir();
    init_config(&dir);
    assert!(
        run(ConfigCommand::Init {
            output: cfg_path(&dir),
            force: true
        })
        .is_ok()
    );
}

// 10. config validate on missing file returns Err
#[test]
fn validate_missing_file_returns_err() {
    let dir = make_dir();
    assert!(
        run(ConfigCommand::Validate {
            file: dir.path().join("no_file.yaml")
        })
        .is_err()
    );
}

// 11. config show missing file returns Err
#[test]
fn show_missing_file_returns_err() {
    let dir = make_dir();
    assert!(
        run(ConfigCommand::Show {
            file: dir.path().join("no_file.yaml")
        })
        .is_err()
    );
}

// 12. config env-list returns Ok
#[test]
fn env_list_ok() {
    assert!(run(ConfigCommand::EnvList).is_ok());
}

// 13. config validate-strict after reset returns Ok
#[test]
fn validate_strict_after_reset_ok() {
    let dir = make_dir();
    run(ConfigCommand::Reset {
        file: cfg_path(&dir),
    })
    .unwrap();
    assert!(
        run(ConfigCommand::ValidateStrict {
            file: cfg_path(&dir)
        })
        .is_ok()
    );
}
