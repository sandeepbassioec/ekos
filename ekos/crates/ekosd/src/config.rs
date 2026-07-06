//! ekosd runtime configuration loaded from `ekosd.toml`.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,

    #[serde(default = "default_bind")]
    pub bind: String,

    #[serde(default = "default_log_level")]
    pub log_level: String,

    #[serde(default)]
    pub store: StoreConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StoreConfig {
    /// Flush memtable to SSTable when it exceeds this size in bytes.
    #[serde(default = "default_flush_threshold")]
    pub flush_threshold_bytes: usize,

    /// Trigger compaction when SSTable count exceeds this.
    #[serde(default = "default_compaction_trigger")]
    pub compaction_trigger: usize,

    /// fsync WAL on every flush (disable for benchmarks only).
    #[serde(default = "default_true")]
    pub fsync: bool,
}

fn default_data_dir() -> PathBuf { PathBuf::from("./ekosd-data") }
fn default_bind()     -> String  { "[::1]:50051".to_string() }
fn default_log_level()-> String  { "info".to_string() }
fn default_flush_threshold()  -> usize { 4 * 1024 * 1024 }
fn default_compaction_trigger()-> usize { 8 }
fn default_true() -> bool { true }

impl Default for Config {
    fn default() -> Self {
        Config {
            data_dir:  default_data_dir(),
            bind:      default_bind(),
            log_level: default_log_level(),
            store:     StoreConfig::default(),
        }
    }
}

impl Config {
    /// Load from `ekosd.toml` if it exists; otherwise return defaults.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        if path.exists() {
            let text = std::fs::read_to_string(path)?;
            Ok(toml::from_str(&text)?)
        } else {
            Ok(Config::default())
        }
    }
}
