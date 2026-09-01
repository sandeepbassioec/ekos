//! [`Sink`] — the core async trait that every sink implements.

use async_trait::async_trait;

use crate::error::SinkResult;

/// One change event delivered to a sink.
#[derive(Debug, Clone)]
pub struct SinkEvent {
    /// Source table name.
    pub table: String,
    /// The kind of mutation.
    pub kind: SinkEventKind,
    /// Primary key.
    pub pk: u64,
    /// Column values (empty for deletions).
    pub values: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SinkEventKind {
    Insert,
    Update,
    Delete,
}

impl SinkEvent {
    pub fn insert(table: &str, pk: u64, values: Vec<u64>) -> Self {
        Self {
            table: table.to_string(),
            kind: SinkEventKind::Insert,
            pk,
            values,
        }
    }

    pub fn delete(table: &str, pk: u64) -> Self {
        Self {
            table: table.to_string(),
            kind: SinkEventKind::Delete,
            pk,
            values: vec![],
        }
    }
}

/// Async trait that every sink must implement.
#[async_trait]
pub trait Sink: Send + Sync {
    /// Human-readable identifier for this sink instance.
    fn name(&self) -> &str;

    /// Deliver one event to the downstream system.
    async fn send(&self, event: SinkEvent) -> SinkResult<()>;

    /// Flush any internal buffers.
    async fn flush(&self) -> SinkResult<()>;
}
