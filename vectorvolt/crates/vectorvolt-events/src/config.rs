//! YAML config structs for domain events (VV-EVT-001, VV-EVT-009).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level `domain_events:` + `sinks:` config block (VV-EVT-009).
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct YamlEventRouteConfig {
    #[serde(default)]
    pub domain_events: Vec<EntityEventConfig>,
    #[serde(default)]
    pub sinks: SinkConfig,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EntityEventConfig {
    pub entity: String,
    #[serde(default)]
    pub events: Vec<DomainEventSchema>,
}

/// One event declaration under an entity (VV-EVT-001).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEventSchema {
    pub name: String,
    pub version: String,
    /// Domain / entity this event belongs to (e.g. "orders"). Used by the registry for grouping.
    #[serde(default)]
    pub entity: String,
    pub trigger: TriggerKind,
    /// Optional Rhai filter — event is suppressed when this evaluates to false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub when: Option<String>,
    #[serde(default)]
    pub fields: HashMap<String, EventFieldDef>,
    #[serde(default)]
    pub routes: Vec<EventRouteConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerKind {
    Insert,
    Update,
    Delete,
    Condition,
}

/// Field definition — direct, computed, nested, or lookup (VV-EVT-001, 014, 015, 016).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventFieldDef {
    /// Direct column ref: `"pk"` or `"values[N]"`.
    Direct(String),
    /// Structured field type.
    Structured(StructuredField),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredField {
    /// Rhai expression (VV-EVT-014).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expr: Option<String>,
    /// Nested object sub-fields (VV-EVT-015).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<HashMap<String, String>>,
    /// Cross-entity lookup (VV-EVT-016).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lookup: Option<LookupDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LookupDef {
    pub table: String,
    /// Source expression for the PK to look up (e.g. `"values[0]"`).
    pub pk: String,
    /// Column index in the foreign table.
    pub column: usize,
}

/// One sink route on an event (VV-EVT-009).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRouteConfig {
    pub sink: SinkKind,
    // Kafka fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partition_key: Option<String>,
    // Redis Stream fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxlen: Option<u64>,
    // Webhook fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hmac_secret: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SinkKind {
    Kafka,
    RedisStream,
    Webhook,
    InProcess,
}

/// Global sink connection settings (VV-EVT-009).
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SinkConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kafka: Option<KafkaSinkConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis_stream: Option<RedisSinkConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<WebhookSinkConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaSinkConfig {
    pub brokers: Vec<String>,
    #[serde(default = "default_compression")]
    pub compression: String,
    #[serde(default = "default_acks")]
    pub acks: String,
    #[serde(default = "default_retries")]
    pub retries: u32,
}

fn default_compression() -> String {
    "lz4".into()
}
fn default_acks() -> String {
    "all".into()
}
fn default_retries() -> u32 {
    3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisSinkConfig {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookSinkConfig {
    #[serde(default = "default_timeout")]
    pub timeout_s: u64,
    #[serde(default = "default_retry_max")]
    pub retry_max: u32,
    #[serde(default = "default_backoff")]
    pub retry_backoff_ms: u64,
}

fn default_timeout() -> u64 {
    5
}
fn default_retry_max() -> u32 {
    3
}
fn default_backoff() -> u64 {
    500
}
