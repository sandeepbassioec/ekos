//! Layer 70 — CDC Webhook Sink registry.
//!
//! A webhook sink subscribes to a table's CDC broadcast channel and POSTs
//! every [`ChangeEvent`] as JSON to a caller-supplied URL.  Each registration
//! returns a unique numeric ID that can be used to remove the sink.
//!
//! ## Delivery guarantees
//!
//! - Best-effort: if the target URL is unreachable the event is dropped.
//! - Fire-and-forget: the background task does not block the write path.
//! - Lagged receivers (broadcast buffer overflow) skip missed events and
//!   continue from the next available event.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use parking_lot::RwLock;
use tokio::sync::broadcast;
use vectorvolt_engine::ChangeEvent;

/// One registered webhook sink.
#[derive(Debug, Clone)]
pub struct WebhookEntry {
    pub id: u64,
    pub table: String,
    pub url: String,
}

/// Global registry: table name → list of active webhook entries.
///
/// Each entry also has a cancellation handle (the task's abort handle) so
/// that `remove` can stop the background worker immediately.
pub struct WebhookRegistry {
    next_id: AtomicU64,
    /// id → (entry, abort_handle)
    entries: RwLock<HashMap<u64, (WebhookEntry, tokio::task::AbortHandle)>>,
}

impl Default for WebhookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl WebhookRegistry {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            entries: RwLock::new(HashMap::new()),
        }
    }

    /// Register a new webhook sink for `table` posting to `url`.
    ///
    /// Spawns a background task that subscribes to `rx` and forwards every
    /// `ChangeEvent` as a JSON POST.  Returns the assigned sink ID.
    pub fn register(
        self: &Arc<Self>,
        table: String,
        url: String,
        mut rx: broadcast::Receiver<ChangeEvent>,
    ) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let entry = WebhookEntry {
            id,
            table: table.clone(),
            url: url.clone(),
        };

        let task = tokio::spawn(async move {
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap_or_default();
            loop {
                match rx.recv().await {
                    Ok(event) => {
                        let _ = client.post(&url).json(&event).send().await;
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });

        self.entries
            .write()
            .insert(id, (entry, task.abort_handle()));
        id
    }

    /// List all sinks registered for `table`.
    pub fn list_for_table(&self, table: &str) -> Vec<WebhookEntry> {
        self.entries
            .read()
            .values()
            .filter(|(e, _)| e.table == table)
            .map(|(e, _)| e.clone())
            .collect()
    }

    /// Remove a sink by ID.  Returns `true` if it existed.
    pub fn remove(&self, id: u64) -> bool {
        if let Some((_, handle)) = self.entries.write().remove(&id) {
            handle.abort();
            true
        } else {
            false
        }
    }

    /// Check whether a sink ID belongs to `table`.
    pub fn belongs_to(&self, id: u64, table: &str) -> bool {
        self.entries
            .read()
            .get(&id)
            .map(|(e, _)| e.table == table)
            .unwrap_or(false)
    }
}
