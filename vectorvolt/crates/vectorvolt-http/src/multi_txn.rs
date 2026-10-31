//! Layer 71 — Multi-Table Transaction coordinator.
//!
//! Buffers a sequence of insert / delete operations across multiple tables and
//! applies them atomically on commit.  Each multi-table transaction is
//! identified by a unique numeric ID.
//!
//! ## Atomicity
//!
//! Each table's ops are applied as one atomic WAL record
//! (`TableEngine::apply_batch`). When more than one table is involved, the
//! write sets are first recorded in the shared cross-table intent log
//! (`<base_dir>/_txn_intent.log`, the same one SQL and `vvctl serve` recovery
//! use), so a crash between tables is completed on restart instead of leaving
//! a partial transaction.
//!
//! ## Isolation
//!
//! Buffered ops are invisible to other readers until commit.

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

        // Group by table, keeping staging order (a later op on the same pk wins
        // because a batch is applied in order).
        let mut per_table: Vec<(String, Vec<(u64, Option<Vec<u64>>)>)> = Vec::new();
        for op in ops {
            match per_table.iter_mut().find(|(t, _)| *t == op.table) {
                Some((_, v)) => v.push((op.pk, op.values)),
                None => per_table.push((op.table, vec![(op.pk, op.values)])),
            }
        }
        // Resolve every table before writing anything.
        let mut engines = Vec::with_capacity(per_table.len());
        for (table, _) in &per_table {
            engines.push(
                registry
                    .get(table)
                    .map_err(|e| CommitError::Engine(e.to_string()))?,
            );
        }

        let engine_err = |e: vectorvolt_engine::EngineError| CommitError::Engine(e.to_string());
        if per_table.len() == 1 {
            let (_, batch) = per_table.pop().expect("one table");
            engines[0].apply_batch(batch).map_err(engine_err)?;
            return Ok(count);
        }

        let log =
            vectorvolt_engine::TxnIntentLog::open(registry.base_dir().join("_txn_intent.log"))
                .map_err(engine_err)?;
        let uuid = format!(
            "http-{}-{txn_id}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        let tables: Vec<vectorvolt_engine::TableTxn> = per_table
            .iter()
            .map(|(t, _)| vectorvolt_engine::TableTxn {
                table: t.clone(),
                txn_id: 0,
            })
            .collect();
        let write_set: Vec<vectorvolt_engine::TableWriteSet> = per_table
            .iter()
            .map(|(t, ops)| vectorvolt_engine::TableWriteSet {
                table: t.clone(),
                ops: ops.clone(),
            })
            .collect();
        log.prepare_with_write_set(&uuid, tables.clone(), write_set.clone())
            .map_err(engine_err)?;
        let mut applied = Vec::new();
        for ((table, batch), eng) in per_table.into_iter().zip(engines) {
            eng.apply_batch(batch).map_err(|e| {
                CommitError::Engine(format!(
                    "{e}; transaction {uuid} left pending, completed on next startup"
                ))
            })?;
            applied.push(table);
            log.mark_tables_applied(&uuid, tables.clone(), write_set.clone(), applied.clone())
                .map_err(engine_err)?;
        }
        log.mark_committed(&uuid, tables).map_err(engine_err)?;
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
