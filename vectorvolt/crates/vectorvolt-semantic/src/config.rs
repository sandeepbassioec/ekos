//! YAML config structs for partitioning and read models (VV-CFG-040, VV-CFG-041, VV-PRJ-017).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// `partitioning:` block on an entity (VV-CFG-040).
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct PartitionConfig {
    /// Routing strategy for writes.
    #[serde(default)]
    pub strategy: PartitionStrategyKind,
    /// Column that drives partitioning (the partition key column name).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<String>,
    /// For `date`: year | month | day.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub granularity: Option<String>,
    /// Total shard count (used by hash/expression strategies).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shard_count: Option<usize>,
    /// For `expression`: Rhai formula applied over column values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formula: Option<String>,
    /// For `explicit`: discrete value → shard index table.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub shard_map: HashMap<String, usize>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PartitionStrategyKind {
    #[default]
    None,
    Date,
    Customer,
    Hash,
    Expression,
    Explicit,
}

/// `cache:` block under a read model (VV-CFG-041).
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CacheConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_process: Option<InProcessCacheConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis: Option<RedisCacheConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InProcessCacheConfig {
    pub max_entries: usize,
    pub ttl_s: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RedisCacheConfig {
    pub url: String,
    pub ttl_s: u64,
    #[serde(default = "default_key_prefix")]
    pub key_prefix: String,
}

fn default_key_prefix() -> String {
    "vv:rm:".into()
}

/// One entry in the `read_models:` YAML list (VV-PRJ-017).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadModelConfig {
    pub name: String,
    pub source: String,
    pub reducer: ReducerKind,
    /// Path to a `.rhai` file (only when `reducer: custom`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reducer_script: Option<String>,
    /// HTTP endpoint for the callback reducer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Column or expression to group by (e.g. `customer_id`, `month(created_at)`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_by: Option<String>,
    #[serde(default)]
    pub cache: CacheConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReducerKind {
    Count,
    Sum,
    Min,
    Max,
    Custom,
    HttpCallback,
}
