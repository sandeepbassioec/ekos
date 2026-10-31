//! MVCC (Multi-Version Concurrency Control) transaction layer.
//!
//! Provides multi-statement BEGIN / COMMIT / ROLLBACK semantics on top of
//! [`TableEngine`].  Each transaction buffers its mutations privately; they
//! are invisible to other readers until `commit_txn` flushes them atomically
//! into the engine.  `rollback_txn` simply discards the buffer.
//!
//! ## Isolation model
//!
//! Reads inside a transaction first check the transaction's own write buffer
//! (so a `get_txn` after `insert_txn` sees the pending value), then fall
//! through to the underlying engine (snapshot at open time — the engine is
//! always consistent at the point of the last committed flush).
//!
//! This gives **Read-Committed with within-transaction read-your-own-writes**:
//! each `get_txn` sees the last committed engine state plus the caller's own
//! un-committed mutations.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use parking_lot::Mutex;

use crate::{EngineError, EngineResult};

/// Opaque transaction identifier returned by `begin`.
pub type TxnId = u64;

/// A single pending mutation: `Some(values)` = insert/update, `None` = delete.
type Intent = Option<Vec<u64>>;

/// State for one open transaction.
struct TxnState {
    /// Pending mutations keyed by pk.  Last write wins within a transaction.
    mutations: HashMap<u64, Intent>,
}

/// Shared store of open transactions, embedded into `TableEngine`.
#[derive(Default)]
pub(crate) struct MvccStore {
    next_id: AtomicU64,
    txns: Mutex<HashMap<TxnId, TxnState>>,
}

impl MvccStore {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Open a new transaction and return its id.
    pub(crate) fn begin(&self) -> TxnId {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.txns.lock().insert(
            id,
            TxnState {
                mutations: HashMap::new(),
            },
        );
        id
    }

    /// Stage an insert/update inside `txn_id`.  Returns an error if the
    /// transaction does not exist (already committed or rolled back).
    pub(crate) fn write_insert(
        &self,
        txn_id: TxnId,
        pk: u64,
        values: Vec<u64>,
    ) -> EngineResult<()> {
        let mut guard = self.txns.lock();
        let txn = guard
            .get_mut(&txn_id)
            .ok_or(EngineError::TxnNotFound(txn_id))?;
        txn.mutations.insert(pk, Some(values));
        Ok(())
    }

    /// Stage a delete inside `txn_id`.
    pub(crate) fn write_delete(&self, txn_id: TxnId, pk: u64) -> EngineResult<()> {
        let mut guard = self.txns.lock();
        let txn = guard
            .get_mut(&txn_id)
            .ok_or(EngineError::TxnNotFound(txn_id))?;
        txn.mutations.insert(pk, None);
        Ok(())
    }

    /// Read the pending intent for `pk` in `txn_id`.
    ///
    /// Returns:
    /// - `Ok(Some(Some(values)))` — row is pending insert/update in this txn
    /// - `Ok(Some(None))` — row is pending delete in this txn
    /// - `Ok(None)` — pk not in this transaction's write buffer (fall through to engine)
    /// - `Err` — txn_id is unknown
    pub(crate) fn read_intent(&self, txn_id: TxnId, pk: u64) -> EngineResult<Option<Intent>> {
        let guard = self.txns.lock();
        let txn = guard.get(&txn_id).ok_or(EngineError::TxnNotFound(txn_id))?;
        Ok(txn.mutations.get(&pk).cloned())
    }

    /// Drain and return all mutations from `txn_id` so the engine can apply them.
    pub(crate) fn drain_commit(&self, txn_id: TxnId) -> EngineResult<HashMap<u64, Intent>> {
        let mut guard = self.txns.lock();
        let txn = guard
            .remove(&txn_id)
            .ok_or(EngineError::TxnNotFound(txn_id))?;
        Ok(txn.mutations)
    }

    /// Copy of the pending mutations in `txn_id`, leaving the transaction open.
    pub(crate) fn peek(&self, txn_id: TxnId) -> EngineResult<Vec<(u64, Intent)>> {
        let guard = self.txns.lock();
        let txn = guard.get(&txn_id).ok_or(EngineError::TxnNotFound(txn_id))?;
        Ok(txn
            .mutations
            .iter()
            .map(|(pk, i)| (*pk, i.clone()))
            .collect())
    }

    /// Discard `txn_id` without applying mutations.
    pub(crate) fn rollback(&self, txn_id: TxnId) -> EngineResult<()> {
        let removed = self.txns.lock().remove(&txn_id);
        removed.ok_or(EngineError::TxnNotFound(txn_id)).map(|_| ())
    }
}
