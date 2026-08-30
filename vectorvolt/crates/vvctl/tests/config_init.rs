//! Integration tests for L124: `vvctl config init/show/validate` (VV-CFG-012).

use std::path::PathBuf;

use tempfile::TempDir;
use vectorvolt_engine::{SystemProfile, VvConfig};
use vvctl::commands::config::{ConfigCommand, config_from_profile, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}

fn output_path(dir: &TempDir) -> PathBuf {
    dir.path().join("vectorvolt.yaml")
}

// ── CLI: init ────────────────────────────────────────────────────────────────

#[test]
fn init_creates_file() {
    let dir = make_dir();
    run(ConfigCommand::Init {
        output: output_path(&dir),
        force: false,
    })
    .unwrap();
    assert!(output_path(&dir).exists());
}

#[test]
fn init_file_is_valid_vv_config() {
    let dir = make_dir();
    run(ConfigCommand::Init {
        output: output_path(&dir),
        force: false,
    })
    .unwrap();
    let cfg = VvConfig::from_file(output_path(&dir)).unwrap();
    assert!(cfg.validate().is_empty());
}

#[test]
fn init_existing_file_without_force_errors() {
    let dir = make_dir();
    std::fs::write(output_path(&dir), "{}").unwrap();
    let result = run(ConfigCommand::Init {
        output: output_path(&dir),
        force: false,
    });
    assert!(result.is_err());
}

#[test]
fn init_force_overwrites_existing() {
    let dir = make_dir();
    std::fs::write(output_path(&dir), "threads:\n  writer_count: 1\n").unwrap();
    run(ConfigCommand::Init {
        output: output_path(&dir),
        force: true,
    })
    .unwrap();
    let cfg = VvConfig::from_file(output_path(&dir)).unwrap();
    assert!(cfg.threads.writer_count.unwrap_or(0) >= 1);
}

#[test]
fn init_sets_network_bind_addr() {
    let dir = make_dir();
    run(ConfigCommand::Init {
        output: output_path(&dir),
        force: false,
    })
    .unwrap();
    let cfg = VvConfig::from_file(output_path(&dir)).unwrap();
    assert!(cfg.network.bind_addr.is_some());
}

#[test]
fn init_sets_storage_compaction_strategy() {
    let dir = make_dir();
    run(ConfigCommand::Init {
        output: output_path(&dir),
        force: false,
    })
    .unwrap();
    let cfg = VvConfig::from_file(output_path(&dir)).unwrap();
    assert_eq!(cfg.storage.compaction_strategy.as_deref(), Some("leveled"));
}

#[test]
fn init_sets_query_index_hint() {
    let dir = make_dir();
    run(ConfigCommand::Init {
        output: output_path(&dir),
        force: false,
    })
    .unwrap();
    let cfg = VvConfig::from_file(output_path(&dir)).unwrap();
    assert_eq!(cfg.query.index_hint.as_deref(), Some("auto"));
}

// ── CLI: show ────────────────────────────────────────────────────────────────

#[test]
fn show_prints_yaml() {
    let dir = make_dir();
    std::fs::write(output_path(&dir), "threads:\n  writer_count: 4\n").unwrap();
    run(ConfigCommand::Show {
        file: output_path(&dir),
    })
    .unwrap();
}

#[test]
fn show_missing_file_errors() {
    let dir = make_dir();
    let result = run(ConfigCommand::Show {
        file: dir.path().join("missing.yaml"),
    });
    assert!(result.is_err());
}

// ── CLI: validate ────────────────────────────────────────────────────────────

#[test]
fn validate_clean_config_succeeds() {
    let dir = make_dir();
    std::fs::write(
        output_path(&dir),
        "storage:\n  compaction_strategy: leveled\n",
    )
    .unwrap();
    run(ConfigCommand::Validate {
        file: output_path(&dir),
    })
    .unwrap();
}

#[test]
fn validate_bad_value_errors() {
    let dir = make_dir();
    std::fs::write(output_path(&dir), "hal:\n  huge_pages: 4gb\n").unwrap();
    let result = run(ConfigCommand::Validate {
        file: output_path(&dir),
    });
    assert!(result.is_err());
}

// ── engine API ───────────────────────────────────────────────────────────────

#[test]
fn config_from_profile_sets_writer_count() {
    let profile = SystemProfile::probe();
    let cfg = config_from_profile(&profile);
    assert_eq!(
        cfg.threads.writer_count,
        Some(profile.recommended_partition_count as u32)
    );
}

#[test]
fn config_from_profile_passes_validate() {
    let profile = SystemProfile::probe();
    let cfg = config_from_profile(&profile);
    assert!(cfg.validate().is_empty());
}
