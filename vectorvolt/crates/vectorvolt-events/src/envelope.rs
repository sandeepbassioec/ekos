//! DomainEventEnvelope — wire format (VV-EVT-003).

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Wire envelope wrapping every domain event.
///
/// Serialised as JSON (primary) and MessagePack (compact transport).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEventEnvelope {
    pub event_type: String,
    pub schema_version: String,
    pub entity: String,
    pub pk: u64,
    pub timestamp_ns: u64,
    pub payload: Value,
}

impl DomainEventEnvelope {
    pub fn new(
        event_type: impl Into<String>,
        schema_version: impl Into<String>,
        entity: impl Into<String>,
        pk: u64,
        payload: Value,
    ) -> Self {
        let timestamp_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        Self {
            event_type: event_type.into(),
            schema_version: schema_version.into(),
            entity: entity.into(),
            pk,
            timestamp_ns,
            payload,
        }
    }

    /// JSON serialisation.
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    /// MessagePack serialisation (VV-EVT-003).
    pub fn to_msgpack(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec(self)
    }

    /// Deserialise from MessagePack.
    pub fn from_msgpack(bytes: &[u8]) -> Result<Self, rmp_serde::decode::Error> {
        rmp_serde::from_slice(bytes)
    }
}
