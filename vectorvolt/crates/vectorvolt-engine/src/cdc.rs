//! Change Data Capture (CDC) types.
//!
//! Every successful mutation published to the per-engine broadcast channel
//! carries a [`ChangeEvent`].  HTTP consumers receive events over SSE at
//! `GET /tables/{t}/subscribe`.

use serde::{Deserialize, Serialize};

/// The kind of change that occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    Insert,
    Delete,
}

/// A single mutation broadcast to all active subscribers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeEvent {
    pub kind: ChangeKind,
    pub pk: u64,
    /// New row values — present on [`ChangeKind::Insert`], absent on [`ChangeKind::Delete`].
    pub values: Option<Vec<u64>>,
}

impl ChangeEvent {
    pub fn insert(pk: u64, values: Vec<u64>) -> Self {
        Self {
            kind: ChangeKind::Insert,
            pk,
            values: Some(values),
        }
    }

    pub fn delete(pk: u64) -> Self {
        Self {
            kind: ChangeKind::Delete,
            pk,
            values: None,
        }
    }
}
