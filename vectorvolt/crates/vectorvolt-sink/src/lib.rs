//! `vectorvolt-sink` — sink trait, registry, WAL bridge, Postgres and Kafka sinks.

pub mod bridge;
pub mod error;
pub mod kafka;
pub mod postgres;
pub mod registry;
pub mod sink;

pub use bridge::WalSinkBridge;
pub use error::{SinkError, SinkResult};
pub use kafka::{KafkaSink, KafkaSinkReceiver};
pub use postgres::PostgresSink;
pub use registry::SinkRegistry;
pub use sink::{Sink, SinkEvent, SinkEventKind};
