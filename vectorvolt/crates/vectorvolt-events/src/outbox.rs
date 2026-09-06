//! EventOutbox (at-least-once) + EventDlq (dead-letter queue). VV-EVT-008, VV-EVT-012.

use std::collections::VecDeque;
use std::sync::Mutex;

use tracing::warn;

use crate::envelope::DomainEventEnvelope;
use crate::sink::EventSink;

// ── EventOutbox ──────────────────────────────────────────────────────────────

/// Transactional outbox — pending envelopes written before broker delivery (VV-EVT-008).
///
/// In production, entries are persisted to a WAL OUTBOX frame. Here we use an in-memory
/// queue so the interface is fully testable without WAL infrastructure.
pub struct EventOutbox {
    pending: Mutex<VecDeque<DomainEventEnvelope>>,
}

impl EventOutbox {
    pub fn new() -> Self {
        Self {
            pending: Mutex::new(VecDeque::new()),
        }
    }

    /// Append an envelope to the outbox (called in the commit path).
    pub fn push(&self, envelope: DomainEventEnvelope) {
        self.pending.lock().unwrap().push_back(envelope);
    }

    /// Flush pending envelopes to the given sink, moving failures to DLQ.
    pub fn flush(&self, sink: &dyn EventSink, dlq: &EventDlq) {
        let mut guard = self.pending.lock().unwrap();
        while let Some(env) = guard.pop_front() {
            if let Err(e) = sink.send(&env) {
                warn!("outbox flush failed: {e}");
                dlq.push(env, e.to_string());
            }
        }
    }

    pub fn pending_count(&self) -> usize {
        self.pending.lock().unwrap().len()
    }
}

impl Default for EventOutbox {
    fn default() -> Self {
        Self::new()
    }
}

// ── EventDlq ─────────────────────────────────────────────────────────────────

/// Dead-letter queue for failed deliveries (VV-EVT-012).
pub struct EventDlq {
    entries: Mutex<Vec<DlqEntry>>,
}

pub struct DlqEntry {
    pub envelope: DomainEventEnvelope,
    pub error: String,
}

impl EventDlq {
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(Vec::new()),
        }
    }

    pub fn push(&self, envelope: DomainEventEnvelope, error: String) {
        self.entries
            .lock()
            .unwrap()
            .push(DlqEntry { envelope, error });
    }

    /// Returns (event_type, error) pairs for `vvctl events dlq list`.
    pub fn list(&self) -> Vec<(String, String)> {
        self.entries
            .lock()
            .unwrap()
            .iter()
            .map(|e| (e.envelope.event_type.clone(), e.error.clone()))
            .collect()
    }

    /// Replay all DLQ entries back through a sink, removing successfully delivered ones.
    pub fn replay(&self, sink: &dyn EventSink) {
        let mut guard = self.entries.lock().unwrap();
        guard.retain(|e| sink.send(&e.envelope).is_err());
    }

    /// Drop all DLQ entries.
    pub fn drop_all(&self) {
        self.entries.lock().unwrap().clear();
    }

    pub fn len(&self) -> usize {
        self.entries.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.lock().unwrap().is_empty()
    }
}

impl Default for EventDlq {
    fn default() -> Self {
        Self::new()
    }
}
