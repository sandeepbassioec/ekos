//! Batch read and write operations.
//!
//! Each batch call processes multiple rows in a single engine invocation,
//! eliminating per-row call overhead for bulk OLTP workloads.
//!
//! ## Semantics
//!
//! - **`batch_get`** — returns one `Option<EngineRow>` per requested PK, in
//!   the same order as the input slice.  Missing PKs produce `None`; errors
//!   (e.g. I/O failure reading a partition) propagate immediately.
//!
//! - **`batch_insert`** — inserts every `(pk, values)` pair.  Like the single
//!   `insert`, an existing PK is overwritten.  Returns the count of rows
//!   written.  The first schema-mismatch error aborts the batch.
//!
//! - **`batch_delete`** — deletes each PK that exists and silently skips
//!   missing ones.  Returns the count of rows actually deleted.  Any other
//!   error (I/O, etc.) propagates immediately.

use crate::engine::TableEngine;
use crate::{EngineResult, EngineRow};

impl TableEngine {
    /// Fetch multiple rows by primary key in one call.
    ///
    /// Returns one `Option<EngineRow>` per element of `pks`, preserving order.
    /// `None` means the PK was not found; an `Err` means an I/O or storage
    /// fault occurred and the batch is aborted.
    pub fn batch_get(&self, pks: &[u64]) -> EngineResult<Vec<Option<EngineRow>>> {
        pks.iter().map(|&pk| self.get(pk)).collect()
    }

    /// Insert multiple rows in one call.
    ///
    /// Rows are inserted in order.  An existing PK is overwritten (same
    /// behaviour as single `insert`).  Returns the number of rows written.
    /// A schema-mismatch or I/O error aborts the batch at the failing row.
    pub fn batch_insert(
        &self,
        rows: impl IntoIterator<Item = (u64, Vec<u64>)>,
    ) -> EngineResult<usize> {
        let mut count = 0usize;
        for (pk, values) in rows {
            self.insert(pk, values)?;
            count += 1;
        }
        Ok(count)
    }

    /// Delete multiple rows by primary key in one call.
    ///
    /// Missing PKs are silently skipped.  Returns the number of rows that were
    /// actually deleted.  Any non-`PkNotFound` error propagates immediately.
    pub fn batch_delete(&self, pks: &[u64]) -> EngineResult<usize> {
        let mut deleted = 0usize;
        for &pk in pks {
            match self.delete(pk) {
                Ok(_) => deleted += 1,
                Err(crate::EngineError::Txn(vectorvolt_txn::TxnError::PkNotFound(_))) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(deleted)
    }
}
