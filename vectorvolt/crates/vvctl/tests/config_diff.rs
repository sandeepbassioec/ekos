//! Integration tests for L126: vvctl config diff (VV-CFG-014).

use std::path::PathBuf;

use tempfile::TempDir;
use vectorvolt_engine::{ConfigDiffKind, VvConfig};
use vvctl::commands::config::{ConfigCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}

fn write_yaml(dir: &TempDir, name: &str, yaml: &str) -> PathBuf {
    let p = dir.path().join(name);
    std::fs::write(&p, yaml).unwrap();
    p
}

// ── engine API: VvConfig::diff ────────────────────────────────────────────────

#[test]
fn diff_identical_configs_empty() {
    let a = VvConfig::default();
    let b = VvConfig::default();
    assert!(a.diff(&b).is_empty());
}

#[test]
fn diff_detects_added_field() {
    let base = VvConfig::default();
    let mut other = VvConfig::default();
    other.threads.writer_count = Some(4);
    let diffs = base.diff(&other);
    assert_eq!(diffs.len(), 1);
    assert_eq!(diffs[0].key, "threads.writer_count");
    assert_eq!(diffs[0].kind, ConfigDiffKind::Added);
    assert!(diffs[0].base.is_none());
    assert_eq!(diffs[0].other.as_deref(), Some("4"));
}

#[test]
fn diff_detects_removed_field() {
    let mut base = VvConfig::default();
    base.network.bind_addr = Some("0.0.0.0:5995".into());
    let other = VvConfig::default();
    let diffs = base.diff(&other);
    assert_eq!(diffs.len(), 1);
    assert_eq!(diffs[0].key, "network.bind_addr");
    assert_eq!(diffs[0].kind, ConfigDiffKind::Removed);
    assert_eq!(diffs[0].base.as_deref(), Some("0.0.0.0:5995"));
    assert!(diffs[0].other.is_none());
}

#[test]
fn diff_detects_changed_field() {
    let mut base = VvConfig::default();
    base.storage.compaction_strategy = Some("leveled".into());
    let mut other = VvConfig::default();
    other.storage.compaction_strategy = Some("tiered".into());
    let diffs = base.diff(&other);
    assert_eq!(diffs.len(), 1);
    assert_eq!(diffs[0].kind, ConfigDiffKind::Changed);
    assert_eq!(diffs[0].base.as_deref(), Some("leveled"));
    assert_eq!(diffs[0].other.as_deref(), Some("tiered"));
}

#[test]
fn diff_multiple_fields() {
    let mut base = VvConfig::default();
    base.threads.writer_count = Some(2);
    base.observability.trace_level = Some("warn".into());
    let mut other = VvConfig::default();
    other.threads.writer_count = Some(8);
    other.network.bind_addr = Some("127.0.0.1:5995".into());
    let diffs = base.diff(&other);
    // writer_count changed, trace_level removed, bind_addr added
    assert_eq!(diffs.len(), 3);
}

#[test]
fn diff_u64_field_max_scan_rows() {
    let mut base = VvConfig::default();
    base.query.max_scan_rows = Some(1000);
    let mut other = VvConfig::default();
    other.query.max_scan_rows = Some(2000);
    let diffs = base.diff(&other);
    assert_eq!(diffs.len(), 1);
    assert_eq!(diffs[0].key, "query.max_scan_rows");
    assert_eq!(diffs[0].kind, ConfigDiffKind::Changed);
}

// ── CLI: config diff ──────────────────────────────────────────────────────────

#[test]
fn diff_cmd_identical_files_succeeds() {
    let dir = make_dir();
    let yaml = "threads:\n  writer_count: 4\n";
    let a = write_yaml(&dir, "a.yaml", yaml);
    let b = write_yaml(&dir, "b.yaml", yaml);
    run(ConfigCommand::Diff { base: a, other: b }).unwrap();
}

#[test]
fn diff_cmd_different_files_errors() {
    let dir = make_dir();
    let a = write_yaml(&dir, "a.yaml", "threads:\n  writer_count: 4\n");
    let b = write_yaml(&dir, "b.yaml", "threads:\n  writer_count: 8\n");
    let result = run(ConfigCommand::Diff { base: a, other: b });
    assert!(result.is_err());
}

#[test]
fn diff_cmd_missing_base_errors() {
    let dir = make_dir();
    let b = write_yaml(&dir, "b.yaml", "{}");
    let result = run(ConfigCommand::Diff {
        base: dir.path().join("missing.yaml"),
        other: b,
    });
    assert!(result.is_err());
}

#[test]
fn diff_cmd_missing_other_errors() {
    let dir = make_dir();
    let a = write_yaml(&dir, "a.yaml", "{}");
    let result = run(ConfigCommand::Diff {
        base: a,
        other: dir.path().join("missing.yaml"),
    });
    assert!(result.is_err());
}

#[test]
fn diff_cmd_empty_vs_populated() {
    let dir = make_dir();
    let a = write_yaml(&dir, "a.yaml", "{}");
    let b = write_yaml(&dir, "b.yaml", "network:\n  bind_addr: 127.0.0.1:5995\n");
    let result = run(ConfigCommand::Diff { base: a, other: b });
    assert!(result.is_err()); // difference found → exit 1
}

#[test]
fn diff_symmetric_add_remove() {
    let mut base = VvConfig::default();
    base.hal.huge_pages = Some("2mb".into());
    let other = VvConfig::default();
    let forward = base.diff(&other);
    let backward = other.diff(&base);
    assert_eq!(forward.len(), backward.len());
    assert_eq!(forward[0].kind, ConfigDiffKind::Removed);
    assert_eq!(backward[0].kind, ConfigDiffKind::Added);
}

#[test]
fn diff_reflexive_same_instance() {
    let mut cfg = VvConfig::default();
    cfg.threads.writer_count = Some(4);
    cfg.storage.compaction_strategy = Some("leveled".into());
    assert!(cfg.diff(&cfg.clone()).is_empty());
}
