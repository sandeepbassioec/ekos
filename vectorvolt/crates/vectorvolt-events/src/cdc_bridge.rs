//! CDC → Domain-Event bridge (S25-A, VV-EVT-020).
//!
//! [`CdcEventBridge`] subscribes to a table's broadcast change feed and
//! re-publishes each [`ChangeEvent`] as a [`DomainEventEnvelope`] through the
//! provided [`EventSink`].  Run as a background tokio task via [`CdcEventBridge::spawn`].

use serde_json::json;
use tokio::sync::broadcast;
use tracing::warn;
use vectorvolt_engine::cdc::ChangeEvent;

use crate::envelope::DomainEventEnvelope;
use crate::sink::EventSink;

/// Bridges one table's CDC broadcast into any [`EventSink`].
pub struct CdcEventBridge {
    table: String,
    entity: String,
    rx: broadcast::Receiver<ChangeEvent>,
    sink: Box<dyn EventSink>,
}

impl CdcEventBridge {
    pub fn new(
        table: impl Into<String>,
        entity: impl Into<String>,
        rx: broadcast::Receiver<ChangeEvent>,
        sink: Box<dyn EventSink>,
    ) -> Self {
        Self {
            table: table.into(),
            entity: entity.into(),
            rx,
            sink,
        }
    }

    /// Consume events from the broadcast channel, forwarding each one to the
    /// sink.  Returns when the sender side is dropped (engine shut down) or on
    /// a fatal sink error.
    pub async fn run(mut self) {
        loop {
            match self.rx.recv().await {
                Ok(ev) => {
                    let event_type = format!(
                        "{}.{}",
                        self.table,
                        match ev.kind {
                            vectorvolt_engine::cdc::ChangeKind::Insert => "inserted",
                            vectorvolt_engine::cdc::ChangeKind::Delete => "deleted",
                        }
                    );
                    let payload = match &ev.values {
                        Some(vals) => json!({ "pk": ev.pk, "values": vals, "lsn": ev.lsn }),
                        None => json!({ "pk": ev.pk, "lsn": ev.lsn }),
                    };
                    // F8: sinks see the commit versionstamp (hex) when one was assigned.
                    let mut payload = payload;
                    if let Some(stamp) = ev.versionstamp {
                        payload["versionstamp"] =
                            json!(stamp.iter().map(|b| format!("{b:02x}")).collect::<String>());
                    }
                    let envelope = DomainEventEnvelope::new(
                        &event_type,
                        "1.0.0",
                        &self.entity,
                        ev.pk,
                        payload,
                    );
                    if let Err(e) = self.sink.send(&envelope) {
                        warn!("CdcEventBridge sink error for {}: {e}", self.table);
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    warn!("CdcEventBridge lagged {n} events on table {}", self.table);
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    }

    /// Spawn the bridge as a detached tokio task.
    pub fn spawn(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(self.run())
    }
}
