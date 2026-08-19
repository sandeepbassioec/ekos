//! Layer 83 — TOML config file loading (VV-ENG-017).
//!
//! Verifies EngineConfig parsing, EngineOptions conversion, and TableSchema
//! construction from TOML text and from on-disk files.

use vectorvolt_engine::config::{ColumnConfig, EngineConfig};
use vectorvolt_engine::{Durability, EngineError, load_config};

// ── from_toml: basic field parsing ───────────────────────────────────────────

#[test]
fn parse_empty_toml_gives_all_none() {
    let cfg = EngineConfig::from_toml("").unwrap();
    assert_eq!(cfg.flush_threshold, None);
    assert_eq!(cfg.durability, None);
    assert_eq!(cfg.max_partitions, None);
    assert!(cfg.columns.is_empty());
}

#[test]
fn parse_flush_threshold() {
    let cfg = EngineConfig::from_toml("flush_threshold = 65536").unwrap();
    assert_eq!(cfg.flush_threshold, Some(65536));
}

#[test]
fn parse_durability_fsync() {
    let cfg = EngineConfig::from_toml("durability = \"fsync\"").unwrap();
    assert_eq!(cfg.durability.as_deref(), Some("fsync"));
}

#[test]
fn parse_durability_buffered() {
    let cfg = EngineConfig::from_toml("durability = \"buffered\"").unwrap();
    assert_eq!(cfg.durability.as_deref(), Some("buffered"));
}

#[test]
fn parse_max_partitions() {
    let cfg = EngineConfig::from_toml("max_partitions = 64").unwrap();
    assert_eq!(cfg.max_partitions, Some(64));
}

#[test]
fn parse_columns_array() {
    let toml = r#"
[[columns]]
name = "pk"
type = "u64"

[[columns]]
name = "active"
type = "bool"
"#;
    let cfg = EngineConfig::from_toml(toml).unwrap();
    assert_eq!(cfg.columns.len(), 2);
    assert_eq!(
        cfg.columns[0],
        ColumnConfig {
            name: "pk".into(),
            col_type: "u64".into()
        }
    );
    assert_eq!(
        cfg.columns[1],
        ColumnConfig {
            name: "active".into(),
            col_type: "bool".into()
        }
    );
}

#[test]
fn invalid_toml_returns_error() {
    let err = EngineConfig::from_toml("flush_threshold = !!!").unwrap_err();
    assert!(matches!(err, EngineError::InvalidConfig(_)));
}

// ── into_engine_options ───────────────────────────────────────────────────────

#[test]
fn default_options_when_all_none() {
    let opts = EngineConfig {
        flush_threshold: None,
        durability: None,
        max_partitions: None,
        columns: vec![],
    }
    .into_engine_options()
    .unwrap();
    assert_eq!(opts.flush_threshold, 1_048_576);
    assert_eq!(opts.durability, Durability::Fsync);
}

#[test]
fn custom_flush_threshold_is_applied() {
    let opts = EngineConfig {
        flush_threshold: Some(256),
        durability: None,
        max_partitions: None,
        columns: vec![],
    }
    .into_engine_options()
    .unwrap();
    assert_eq!(opts.flush_threshold, 256);
}

#[test]
fn durability_write_only_is_parsed() {
    let opts = EngineConfig {
        flush_threshold: None,
        durability: Some("write_only".into()),
        max_partitions: None,
        columns: vec![],
    }
    .into_engine_options()
    .unwrap();
    assert_eq!(opts.durability, Durability::WriteOnly);
}

#[test]
fn unknown_durability_returns_invalid_config() {
    let err = EngineConfig {
        flush_threshold: None,
        durability: Some("nope".into()),
        max_partitions: None,
        columns: vec![],
    }
    .into_engine_options()
    .unwrap_err();
    assert!(matches!(err, EngineError::InvalidConfig(_)));
}

#[test]
fn zero_flush_threshold_returns_invalid_config() {
    let err = EngineConfig {
        flush_threshold: Some(0),
        durability: None,
        max_partitions: None,
        columns: vec![],
    }
    .into_engine_options()
    .unwrap_err();
    assert!(matches!(err, EngineError::InvalidConfig(_)));
}

// ── into_schema ───────────────────────────────────────────────────────────────

#[test]
fn empty_columns_returns_none_schema() {
    let schema = EngineConfig {
        flush_threshold: None,
        durability: None,
        max_partitions: None,
        columns: vec![],
    }
    .into_schema()
    .unwrap();
    assert!(schema.is_none());
}

#[test]
fn columns_produce_correct_schema() {
    let schema = EngineConfig {
        flush_threshold: None,
        durability: None,
        max_partitions: None,
        columns: vec![
            ColumnConfig {
                name: "pk".into(),
                col_type: "u64".into(),
            },
            ColumnConfig {
                name: "score".into(),
                col_type: "f64".into(),
            },
        ],
    }
    .into_schema()
    .unwrap()
    .unwrap();
    assert_eq!(schema.col_count(), 2);
    assert_eq!(schema.names(), vec!["pk", "score"]);
}

// ── load_config (from disk) ───────────────────────────────────────────────────

#[test]
fn load_config_from_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("engine.toml");
    std::fs::write(
        &path,
        "flush_threshold = 32768\ndurability = \"buffered\"\nmax_partitions = 16\n",
    )
    .unwrap();
    let (opts, schema) = load_config(&path).unwrap();
    assert_eq!(opts.flush_threshold, 32768);
    assert_eq!(opts.durability, Durability::Buffered);
    assert!(schema.is_none());
}

#[test]
fn load_config_missing_file_returns_error() {
    let err = load_config("/nonexistent/path/engine.toml").unwrap_err();
    assert!(matches!(err, EngineError::InvalidConfig(_)));
}
