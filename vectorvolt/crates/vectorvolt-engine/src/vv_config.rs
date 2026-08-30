//! Master multi-section YAML configuration for VectorVolt (VV-CFG-010).
//!
//! This module provides the authoritative top-level config consumed by
//! `vvctl serve`, `vvctl catalog config`, and eventually `vectorvolt-server`.
//! It is separate from the per-table [`EngineConfig`](crate::config::EngineConfig),
//! which covers a single table's flush/durability/column settings.
//!
//! ## Sections
//!
//! | Section | Controls |
//! |---|---|
//! | `hal` | CPU affinity, NUMA, huge pages |
//! | `threads` | Writer count, ring-buffer size, priority |
//! | `storage` | Compaction, block size, L0 thresholds |
//! | `observability` | Prometheus, trace level, Jaeger |
//! | `network` | Bind address, TLS, max connections |
//! | `query` | Default projection, scan limits, index hint |
//! | `tables` | Per-table overrides (map of table name → settings) |
//!
//! ## Key setter (dot notation)
//!
//! [`VvConfig::set_key`] supports a simple dot-notation path for scripted
//! updates, e.g. `threads.writer_count = "8"`.  Only leaf string/int fields
//! are supported; nested map keys are not.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{EngineError, EngineResult};

// ── Section structs ───────────────────────────────────────────────────────────

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct HalConfig {
    /// CPU core indices to pin writer threads to.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cpu_cores: Vec<u32>,
    /// NUMA node binding (0-based index).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numa_node: Option<u32>,
    /// Huge page size: `"2mb"` | `"1gb"` | `"disabled"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub huge_pages: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThreadsConfig {
    /// Per-shard writer thread count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub writer_count: Option<u32>,
    /// LMAX Disruptor ring buffer capacity (power of 2 recommended).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ring_buffer_size: Option<u32>,
    /// Thread priority: `"normal"` | `"realtime"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub writer_priority: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct StorageConfig {
    /// Compaction strategy: `"leveled"` | `"tiered"` | `"none"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compaction_strategy: Option<String>,
    /// SSTable L0 file count that triggers a compaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub l0_flush_threshold: Option<u32>,
    /// SSTable block size in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_size: Option<u32>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObservabilityConfig {
    /// `"host:port"` for the Prometheus metrics endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prometheus_addr: Option<String>,
    /// Log/trace level: `"error"` | `"warn"` | `"info"` | `"debug"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_level: Option<String>,
    /// Jaeger collector endpoint (null = disabled).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jaeger_endpoint: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkConfig {
    /// TCP bind address for the VectorVolt binary protocol.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind_addr: Option<String>,
    /// Path to TLS certificate file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls_cert: Option<String>,
    /// Path to TLS private key file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls_key: Option<String>,
    /// Maximum simultaneous client connections.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_connections: Option<u32>,
    /// TCP keepalive interval in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keepalive_secs: Option<u32>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct QueryConfig {
    /// Default columns to return (empty = all).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub default_projection: Vec<String>,
    /// Maximum rows a single scan may return (0 = unlimited).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_scan_rows: Option<u64>,
    /// Index hint: `"auto"` | `"cuckoo"` | `"none"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index_hint: Option<String>,
}

/// Per-table settings that override engine defaults for a specific table.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flush_threshold: Option<usize>,
    /// Durability mode: `"fsync"` | `"write_only"` | `"buffered"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub durability: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partitions: Option<usize>,
    /// Logical entity name (e.g. `"Order"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity: Option<String>,
    /// Whether this is the aggregate root.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregate_root: Option<bool>,
    /// Member tables for `AggregateRouter`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub members: Vec<String>,
}

// ── Top-level config ──────────────────────────────────────────────────────────

/// Master multi-section configuration for VectorVolt (VV-CFG-010).
///
/// Every section is optional; absent sections use their in-code defaults.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct VvConfig {
    #[serde(default, skip_serializing_if = "is_default")]
    pub hal: HalConfig,
    #[serde(default, skip_serializing_if = "is_default")]
    pub threads: ThreadsConfig,
    #[serde(default, skip_serializing_if = "is_default")]
    pub storage: StorageConfig,
    #[serde(default, skip_serializing_if = "is_default")]
    pub observability: ObservabilityConfig,
    #[serde(default, skip_serializing_if = "is_default")]
    pub network: NetworkConfig,
    #[serde(default, skip_serializing_if = "is_default")]
    pub query: QueryConfig,
    /// Per-table overrides, keyed by table name.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub tables: HashMap<String, TableConfig>,
}

fn is_default<T: Default + PartialEq>(v: &T) -> bool {
    *v == T::default()
}

impl VvConfig {
    /// Parse YAML text into a [`VvConfig`].
    pub fn from_yaml(text: &str) -> EngineResult<Self> {
        serde_yaml::from_str(text)
            .map_err(|e| EngineError::InvalidConfig(format!("YAML parse error: {e}")))
    }

    /// Load a YAML config file from disk.
    pub fn from_file(path: impl AsRef<Path>) -> EngineResult<Self> {
        let text = std::fs::read_to_string(path.as_ref())?;
        Self::from_yaml(&text)
    }

    /// Serialize to YAML text.
    pub fn to_yaml(&self) -> EngineResult<String> {
        serde_yaml::to_string(self)
            .map_err(|e| EngineError::InvalidConfig(format!("YAML serialization error: {e}")))
    }

    /// Validate the config — returns a list of warning strings.
    ///
    /// Does not fail on unknown keys (YAML is permissive); only checks
    /// value-level constraints.
    pub fn validate(&self) -> Vec<String> {
        let mut warns = vec![];
        if let Some(ref hp) = self.hal.huge_pages
            && !["2mb", "1gb", "disabled"].contains(&hp.as_str())
        {
            warns.push(format!(
                "hal.huge_pages: unknown value '{hp}' (expected 2mb|1gb|disabled)"
            ));
        }
        if let Some(ref pri) = self.threads.writer_priority
            && !["normal", "realtime"].contains(&pri.as_str())
        {
            warns.push(format!(
                "threads.writer_priority: unknown value '{pri}' (expected normal|realtime)"
            ));
        }
        if let Some(ref cs) = self.storage.compaction_strategy
            && !["leveled", "tiered", "none"].contains(&cs.as_str())
        {
            warns.push(format!(
                "storage.compaction_strategy: unknown value '{cs}' (expected leveled|tiered|none)"
            ));
        }
        if let Some(ref tl) = self.observability.trace_level
            && !["error", "warn", "info", "debug"].contains(&tl.as_str())
        {
            warns.push(format!(
                "observability.trace_level: unknown value '{tl}' (expected error|warn|info|debug)"
            ));
        }
        if let Some(ref ih) = self.query.index_hint
            && !["auto", "cuckoo", "none"].contains(&ih.as_str())
        {
            warns.push(format!(
                "query.index_hint: unknown value '{ih}' (expected auto|cuckoo|none)"
            ));
        }
        warns
    }

    /// Update a single leaf field using dot-notation `section.field`.
    ///
    /// Supported keys:
    /// - `hal.numa_node`, `hal.huge_pages`
    /// - `threads.writer_count`, `threads.ring_buffer_size`, `threads.writer_priority`
    /// - `storage.compaction_strategy`, `storage.l0_flush_threshold`, `storage.block_size`
    /// - `observability.prometheus_addr`, `observability.trace_level`, `observability.jaeger_endpoint`
    /// - `network.bind_addr`, `network.tls_cert`, `network.tls_key`, `network.max_connections`, `network.keepalive_secs`
    /// - `query.max_scan_rows`, `query.index_hint`
    pub fn set_key(&mut self, key: &str, value: &str) -> EngineResult<()> {
        match key {
            "hal.numa_node" => {
                self.hal.numa_node = Some(parse_u32(key, value)?);
            }
            "hal.huge_pages" => {
                self.hal.huge_pages = Some(value.to_owned());
            }
            "threads.writer_count" => {
                self.threads.writer_count = Some(parse_u32(key, value)?);
            }
            "threads.ring_buffer_size" => {
                self.threads.ring_buffer_size = Some(parse_u32(key, value)?);
            }
            "threads.writer_priority" => {
                self.threads.writer_priority = Some(value.to_owned());
            }
            "storage.compaction_strategy" => {
                self.storage.compaction_strategy = Some(value.to_owned());
            }
            "storage.l0_flush_threshold" => {
                self.storage.l0_flush_threshold = Some(parse_u32(key, value)?);
            }
            "storage.block_size" => {
                self.storage.block_size = Some(parse_u32(key, value)?);
            }
            "observability.prometheus_addr" => {
                self.observability.prometheus_addr = Some(value.to_owned());
            }
            "observability.trace_level" => {
                self.observability.trace_level = Some(value.to_owned());
            }
            "observability.jaeger_endpoint" => {
                self.observability.jaeger_endpoint = Some(value.to_owned());
            }
            "network.bind_addr" => {
                self.network.bind_addr = Some(value.to_owned());
            }
            "network.tls_cert" => {
                self.network.tls_cert = Some(value.to_owned());
            }
            "network.tls_key" => {
                self.network.tls_key = Some(value.to_owned());
            }
            "network.max_connections" => {
                self.network.max_connections = Some(parse_u32(key, value)?);
            }
            "network.keepalive_secs" => {
                self.network.keepalive_secs = Some(parse_u32(key, value)?);
            }
            "query.max_scan_rows" => {
                self.query.max_scan_rows = Some(value.parse::<u64>().map_err(|_| {
                    EngineError::InvalidConfig(format!("key '{key}': expected u64, got '{value}'"))
                })?);
            }
            "query.index_hint" => {
                self.query.index_hint = Some(value.to_owned());
            }
            other => {
                return Err(EngineError::InvalidConfig(format!(
                    "unknown config key '{other}'"
                )));
            }
        }
        Ok(())
    }
}

fn parse_u32(key: &str, value: &str) -> EngineResult<u32> {
    value.parse::<u32>().map_err(|_| {
        EngineError::InvalidConfig(format!("key '{key}': expected u32, got '{value}'"))
    })
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_yaml_gives_default() {
        let cfg = VvConfig::from_yaml("{}").unwrap();
        assert_eq!(cfg, VvConfig::default());
    }

    #[test]
    fn round_trip_yaml() {
        let mut cfg = VvConfig::default();
        cfg.threads.writer_count = Some(8);
        cfg.network.bind_addr = Some("0.0.0.0:50051".into());
        let yaml = cfg.to_yaml().unwrap();
        let cfg2 = VvConfig::from_yaml(&yaml).unwrap();
        assert_eq!(cfg, cfg2);
    }

    #[test]
    fn set_key_numeric() {
        let mut cfg = VvConfig::default();
        cfg.set_key("threads.writer_count", "4").unwrap();
        assert_eq!(cfg.threads.writer_count, Some(4));
    }

    #[test]
    fn set_key_string() {
        let mut cfg = VvConfig::default();
        cfg.set_key("storage.compaction_strategy", "leveled")
            .unwrap();
        assert_eq!(cfg.storage.compaction_strategy, Some("leveled".into()));
    }

    #[test]
    fn set_key_unknown_errors() {
        let mut cfg = VvConfig::default();
        let err = cfg.set_key("bogus.key", "x").unwrap_err();
        assert!(err.to_string().contains("bogus.key"));
    }

    #[test]
    fn validate_bad_huge_pages() {
        let mut cfg = VvConfig::default();
        cfg.hal.huge_pages = Some("4gb".into());
        let warns = cfg.validate();
        assert!(!warns.is_empty());
        assert!(warns[0].contains("huge_pages"));
    }

    #[test]
    fn validate_clean_config() {
        let mut cfg = VvConfig::default();
        cfg.hal.huge_pages = Some("2mb".into());
        cfg.storage.compaction_strategy = Some("leveled".into());
        assert!(cfg.validate().is_empty());
    }
}
