//! Typed Domain Events + YAML Event Routing + Broker Sinks (S23).
//!
//! VV-EVT-001..016

pub mod cdc_bridge;
pub mod config;
pub mod emitter;
pub mod envelope;
pub mod field;
pub mod outbox;
pub mod registry;
pub mod sink;

pub use cdc_bridge::CdcEventBridge;
pub use config::{
    DomainEventSchema, EventFieldDef, EventRouteConfig, SinkKind, YamlEventRouteConfig,
};
pub use emitter::DomainEventEmitter;
pub use envelope::DomainEventEnvelope;
pub use outbox::{EventDlq, EventOutbox};
pub use registry::SchemaVersionRegistry;
pub use sink::{EventSink, HttpWebhookSink, InProcessChannelSink, RedisStreamSink};
