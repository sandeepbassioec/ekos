//! Integration tests for the config precedence chain (VV-ENG-017 L104).
//!
//! Precedence: SystemProfile defaults < file < VECTORVOLT_* env vars < CLI overrides.
//!
//! Env-var tests use `apply_env_overrides_with` with an inline map so they never
//! mutate the real process environment — parallel test execution is safe.

use std::collections::HashMap;
use std::fs;

use tempfile::TempDir;
use vectorvolt_engine::{
    CliOverrides, SystemProfile, apply_cli_overrides, apply_env_overrides_with,
    config::EngineConfig, load_config_full, load_config_with_profile,
};

fn profile() -> SystemProfile {
    SystemProfile::probe()
}

fn write_yaml(dir: &TempDir, content: &str) -> std::path::PathBuf {
    let p = dir.path().join("engine.yaml");
    fs::write(&p, content).unwrap();
    p
}

fn empty_cfg() -> EngineConfig {
    EngineConfig {
        flush_threshold: None,
        durability: None,
        max_partitions: None,
        columns: vec![],
    }
}

fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
    let map: HashMap<String, String> = pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    move |key| map.get(key).cloned()
}

// ── apply_env_overrides_with ──────────────────────────────────────────────────

#[test]
fn env_flush_threshold_overrides_none() {
    let mut cfg = empty_cfg();
    apply_env_overrides_with(&mut cfg, env(&[("VECTORVOLT_FLUSH_THRESHOLD", "9999")])).unwrap();
    assert_eq!(cfg.flush_threshold, Some(9999));
}

#[test]
fn env_flush_threshold_overrides_file_value() {
    let mut cfg = empty_cfg();
    cfg.flush_threshold = Some(1024);
    apply_env_overrides_with(&mut cfg, env(&[("VECTORVOLT_FLUSH_THRESHOLD", "8192")])).unwrap();
    assert_eq!(cfg.flush_threshold, Some(8192));
}

#[test]
fn env_durability_set() {
    let mut cfg = empty_cfg();
    apply_env_overrides_with(&mut cfg, env(&[("VECTORVOLT_DURABILITY", "write_only")])).unwrap();
    assert_eq!(cfg.durability.as_deref(), Some("write_only"));
}

#[test]
fn env_max_partitions_set() {
    let mut cfg = empty_cfg();
    apply_env_overrides_with(&mut cfg, env(&[("VECTORVOLT_MAX_PARTITIONS", "16")])).unwrap();
    assert_eq!(cfg.max_partitions, Some(16));
}

#[test]
fn env_invalid_flush_threshold_errors() {
    let mut cfg = empty_cfg();
    let result = apply_env_overrides_with(&mut cfg, env(&[("VECTORVOLT_FLUSH_THRESHOLD", "bad")]));
    assert!(result.is_err());
}

#[test]
fn env_vars_absent_leaves_cfg_unchanged() {
    let mut cfg = empty_cfg();
    cfg.flush_threshold = Some(512);
    apply_env_overrides_with(&mut cfg, env(&[])).unwrap();
    assert_eq!(cfg.flush_threshold, Some(512));
}

// ── apply_cli_overrides ───────────────────────────────────────────────────────

#[test]
fn cli_overrides_all_fields() {
    let mut cfg = EngineConfig {
        flush_threshold: Some(1000),
        durability: Some("fsync".into()),
        max_partitions: Some(4),
        columns: vec![],
    };
    let cli = CliOverrides::new()
        .with_flush_threshold(2048)
        .with_durability("buffered")
        .with_max_partitions(8);
    apply_cli_overrides(&mut cfg, &cli);
    assert_eq!(cfg.flush_threshold, Some(2048));
    assert_eq!(cfg.durability.as_deref(), Some("buffered"));
    assert_eq!(cfg.max_partitions, Some(8));
}

#[test]
fn cli_empty_overrides_leave_values_intact() {
    let mut cfg = EngineConfig {
        flush_threshold: Some(512),
        durability: Some("fsync".into()),
        max_partitions: Some(2),
        columns: vec![],
    };
    apply_cli_overrides(&mut cfg, &CliOverrides::new());
    assert_eq!(cfg.flush_threshold, Some(512));
    assert_eq!(cfg.durability.as_deref(), Some("fsync"));
    assert_eq!(cfg.max_partitions, Some(2));
}

// ── load_config_full precedence ───────────────────────────────────────────────

#[test]
fn file_overrides_profile() {
    let dir = TempDir::new().unwrap();
    let path = write_yaml(&dir, "flush_threshold: 1111\ndurability: write_only\n");
    let (opts, _) = load_config_full(Some(&path), &profile(), &CliOverrides::new()).unwrap();
    assert_eq!(opts.flush_threshold, 1111);
}

#[test]
fn profile_used_when_no_file() {
    let p = profile();
    let (opts, _) = load_config_full(None, &p, &CliOverrides::new()).unwrap();
    assert_eq!(opts.flush_threshold, p.recommended_flush_threshold);
}

#[test]
fn cli_wins_over_file() {
    let dir = TempDir::new().unwrap();
    let path = write_yaml(&dir, "flush_threshold: 1111\n");
    let cli = CliOverrides::new().with_flush_threshold(9999);
    let (opts, _) = load_config_full(Some(&path), &profile(), &cli).unwrap();
    assert_eq!(opts.flush_threshold, 9999);
}

#[test]
fn load_config_with_profile_still_works() {
    let dir = TempDir::new().unwrap();
    let path = write_yaml(&dir, "flush_threshold: 2222\n");
    let (opts, _) = load_config_with_profile(Some(&path), &profile()).unwrap();
    assert_eq!(opts.flush_threshold, 2222);
}
