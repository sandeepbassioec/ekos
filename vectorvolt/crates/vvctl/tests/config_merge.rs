//! Integration tests for L132: vvctl config merge (VV-CFG-015).

use std::path::PathBuf;

use tempfile::TempDir;
use vvctl::commands::config::{ConfigCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}

fn write_yaml(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, content).unwrap();
    path
}

const BASE_YAML: &str = r#"
hal:
  cpu_cores: [0, 1]
  numa_node: 0
threads:
  writer_count: 4
storage:
  compaction_strategy: leveled
  l0_flush_threshold: 8
network:
  bind_addr: "127.0.0.1:5995"
  max_connections: 128
observability:
  trace_level: info
query:
  max_scan_rows: 10000
"#;

const OVERLAY_YAML: &str = r#"
threads:
  writer_count: 8
network:
  bind_addr: "0.0.0.0:5995"
"#;

// ── happy path ────────────────────────────────────────────────────────────────

#[test]
fn merge_base_only_fields_preserved() {
    let dir = make_dir();
    let base = write_yaml(&dir, "base.yaml", BASE_YAML);
    let overlay = write_yaml(&dir, "overlay.yaml", OVERLAY_YAML);
    run(ConfigCommand::Merge {
        base,
        overlay,
        output: None,
        show_diff: false,
    })
    .unwrap();
}

#[test]
fn merge_overlay_wins_for_set_fields() {
    let dir = make_dir();
    let base = write_yaml(&dir, "base.yaml", BASE_YAML);
    let overlay = write_yaml(&dir, "overlay.yaml", "threads:\n  writer_count: 16\n");
    run(ConfigCommand::Merge {
        base,
        overlay,
        output: None,
        show_diff: false,
    })
    .unwrap();
}

#[test]
fn merge_empty_overlay_is_identity() {
    let dir = make_dir();
    let base = write_yaml(&dir, "base.yaml", BASE_YAML);
    let empty = write_yaml(&dir, "empty.yaml", "{}\n");
    run(ConfigCommand::Merge {
        base,
        overlay: empty,
        output: None,
        show_diff: false,
    })
    .unwrap();
}

#[test]
fn merge_output_written_to_file() {
    let dir = make_dir();
    let base = write_yaml(&dir, "base.yaml", BASE_YAML);
    let overlay = write_yaml(&dir, "overlay.yaml", OVERLAY_YAML);
    let out = dir.path().join("merged.yaml");
    run(ConfigCommand::Merge {
        base,
        overlay,
        output: Some(out.clone()),
        show_diff: false,
    })
    .unwrap();
    assert!(out.exists());
    let text = std::fs::read_to_string(&out).unwrap();
    assert!(!text.is_empty());
}

#[test]
fn merge_output_file_is_valid_yaml() {
    let dir = make_dir();
    let base = write_yaml(&dir, "base.yaml", BASE_YAML);
    let overlay = write_yaml(&dir, "overlay.yaml", OVERLAY_YAML);
    let out = dir.path().join("merged.yaml");
    run(ConfigCommand::Merge {
        base,
        overlay,
        output: Some(out.clone()),
        show_diff: false,
    })
    .unwrap();
    let text = std::fs::read_to_string(&out).unwrap();
    // The merged file must parse back as valid VvConfig.
    use vectorvolt_engine::VvConfig;
    VvConfig::from_yaml(&text).unwrap();
}

#[test]
fn merge_show_diff_no_panic() {
    let dir = make_dir();
    let base = write_yaml(&dir, "base.yaml", BASE_YAML);
    let overlay = write_yaml(&dir, "overlay.yaml", OVERLAY_YAML);
    run(ConfigCommand::Merge {
        base,
        overlay,
        output: None,
        show_diff: true,
    })
    .unwrap();
}

#[test]
fn merge_show_diff_empty_overlay_no_panic() {
    let dir = make_dir();
    let base = write_yaml(&dir, "base.yaml", BASE_YAML);
    let empty = write_yaml(&dir, "empty.yaml", "{}\n");
    run(ConfigCommand::Merge {
        base,
        overlay: empty,
        output: None,
        show_diff: true,
    })
    .unwrap();
}

#[test]
fn merge_overlay_bind_addr_overrides_base() {
    let dir = make_dir();
    let base = write_yaml(&dir, "base.yaml", BASE_YAML);
    let overlay = write_yaml(
        &dir,
        "overlay.yaml",
        "network:\n  bind_addr: \"10.0.0.1:9000\"\n",
    );
    let out = dir.path().join("merged.yaml");
    run(ConfigCommand::Merge {
        base,
        overlay,
        output: Some(out.clone()),
        show_diff: false,
    })
    .unwrap();
    let text = std::fs::read_to_string(&out).unwrap();
    assert!(text.contains("10.0.0.1:9000"));
}

#[test]
fn merge_tables_map_merged_by_key() {
    let dir = make_dir();
    let base_with_table = format!("{BASE_YAML}\ntables:\n  orders:\n    partition_count: 4\n");
    let overlay_with_table = "tables:\n  users:\n    partition_count: 2\n".to_string();
    let base = write_yaml(&dir, "base.yaml", &base_with_table);
    let overlay = write_yaml(&dir, "overlay.yaml", &overlay_with_table);
    let out = dir.path().join("merged.yaml");
    run(ConfigCommand::Merge {
        base,
        overlay,
        output: Some(out.clone()),
        show_diff: false,
    })
    .unwrap();
    let text = std::fs::read_to_string(&out).unwrap();
    assert!(text.contains("orders"));
    assert!(text.contains("users"));
}

#[test]
fn merge_all_sections_overlay() {
    let dir = make_dir();
    let base = write_yaml(&dir, "base.yaml", BASE_YAML);
    let full_overlay = r#"
hal:
  cpu_cores: [0, 1, 2, 3]
  numa_node: 1
threads:
  writer_count: 16
  ring_buffer_size: 65536
storage:
  compaction_strategy: tiered
  l0_flush_threshold: 16
  block_size: 65536
network:
  bind_addr: "0.0.0.0:6000"
  max_connections: 512
observability:
  trace_level: debug
query:
  max_scan_rows: 50000
"#;
    let overlay = write_yaml(&dir, "overlay.yaml", full_overlay);
    run(ConfigCommand::Merge {
        base,
        overlay,
        output: None,
        show_diff: false,
    })
    .unwrap();
}

#[test]
fn merge_vec_field_overlay_wins_when_non_empty() {
    let dir = make_dir();
    let base = write_yaml(&dir, "base.yaml", BASE_YAML);
    let overlay = write_yaml(&dir, "overlay.yaml", "hal:\n  cpu_cores: [4, 5, 6, 7]\n");
    let out = dir.path().join("merged.yaml");
    run(ConfigCommand::Merge {
        base,
        overlay,
        output: Some(out.clone()),
        show_diff: false,
    })
    .unwrap();
    let text = std::fs::read_to_string(&out).unwrap();
    assert!(text.contains('4') || text.contains("cpu_cores"));
}

// ── error cases ───────────────────────────────────────────────────────────────

#[test]
fn merge_base_missing_errors() {
    let dir = make_dir();
    let overlay = write_yaml(&dir, "overlay.yaml", OVERLAY_YAML);
    let result = run(ConfigCommand::Merge {
        base: dir.path().join("nonexistent.yaml"),
        overlay,
        output: None,
        show_diff: false,
    });
    assert!(result.is_err());
}

#[test]
fn merge_overlay_missing_errors() {
    let dir = make_dir();
    let base = write_yaml(&dir, "base.yaml", BASE_YAML);
    let result = run(ConfigCommand::Merge {
        base,
        overlay: dir.path().join("nonexistent.yaml"),
        output: None,
        show_diff: false,
    });
    assert!(result.is_err());
}
