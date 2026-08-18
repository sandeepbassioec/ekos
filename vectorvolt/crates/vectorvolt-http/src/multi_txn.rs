//! Layer 71 — Multi-Table Transaction coordinator.
//!
//! Buffers a sequence of insert / delete operations across multiple tables and
//! applies them atomically (serially) on commit.  Each multi-table transaction
//! is identified by a unique numeric ID.
//!
//! ## Isolation
//!
//! Buffered ops are invisible to other readers until commit.  Commit applies
//! ops in the order they were staged; if the engine returns an error mid-commit
//! the already-applied ops are **not** rolled back automatically — callers that
//! need strict atomicity should issue a `rollback` and re-try.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use parking_lot::Mutex;
use vectorvolt_engine::TableRegistry;

/// One buffered operation inside a multi-table transaction.
#[derive(Debug, Clone)]
pub struct MultiTxnOp {
    pub table: String,
    pub pk: u64,
    /// `Some(values)` = insert/upsert, `None` = delete.
    pub values: Option<Vec<u64>>,
}

struct TxnState {
    ops: Vec<MultiTxnOp>,
}

/// Shared registry of open multi-table transactions.
pub struct MultiTxnRegistry {
    next_id: AtomicU64,
    txns: Mutex<HashMap<u64, TxnState>>,
}

impl Default for MultiTxnRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiTxnRegistry {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            txns: Mutex::new(HashMap::new()),
        }
    }

    /// Open a new multi-table transaction and return its ID.
    pub fn begin(&self) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.txns.lock().insert(id, TxnState { ops: Vec::new() });
        id
    }

    /// Stage one operation.  Returns `Err` if `txn_id` does not exist.
    pub fn stage(&self, txn_id: u64, op: MultiTxnOp) -> Result<(), String> {
        let mut guard = self.txns.lock();
        match guard.get_mut(&txn_id) {
            Some(state) => {
                state.ops.push(op);
                Ok(())
            }
            None => Err(format!("multi-txn {txn_id} not found")),
        }
    }

    /// Return the number of pending ops, or `None` if `txn_id` does not exist.
    pub fn op_count(&self, txn_id: u64) -> Option<usize> {
        self.txns.lock().get(&txn_id).map(|s| s.ops.len())
    }

    /// Apply all staged ops to the registry in order, then remove the txn.
    ///
    /// Returns `Err` if `txn_id` does not exist or any engine op fails.
    pub fn commit(&self, txn_id: u64, registry: &Arc<TableRegistry>) -> Result<usize, CommitError> {
        let ops = {
            let mut guard = self.txns.lock();
            guard
                .remove(&txn_id)
                .ok_or(CommitError::NotFound(txn_id))?
                .ops
        };
        let count = ops.len();
        for op in ops {
            let eng = registry
                .get(&op.table)
                .map_err(|e| CommitError::Engine(e.to_string()))?;
            let res = match op.values {
                Some(vals) => eng.insert(op.pk, vals).map(|_| ()),
                None => eng.delete(op.pk).map(|_| ()),
            };
            res.map_err(|e| CommitError::Engine(e.to_string()))?;
        }
        Ok(count)
    }

    /// Discard all staged ops without applying them.  Returns `false` if not found.
    pub fn rollback(&self, txn_id: u64) -> bool {
        self.txns.lock().remove(&txn_id).is_some()
    }
}

#[derive(Debug)]
pub enum CommitError {
    NotFound(u64),
    Engine(String),
}

impl std::fmt::Display for CommitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommitError::NotFound(id) => write!(f, "multi-txn {id} not found"),
            CommitError::Engine(msg) => write!(f, "{msg}"),
        }
    }
}
