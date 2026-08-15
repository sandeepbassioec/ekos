//! Conditional write operations: insert-if-absent, compare-and-swap, upsert.
//!
//! ## Atomicity guarantee
//!
//! Each operation holds `TableEngine::cas_lock` for the duration of its
//! read-check-write sequence.  Because `txn.insert` acquires `pk_index.write()`
//! internally, we cannot hold the index lock externally — instead, `cas_lock`
//! is a dedicated mutex that serialises all conditional write callers relative
//! to each other.
//!
//! Regular `insert` / `delete` / `get` calls are NOT gated on `cas_lock`, so
//! they remain fully concurrent with reads and with each other.  The guarantee
//! is: *two concurrent CAS-family calls will not interleave their read and
//! write halves*.  A concurrent plain `insert` between the CAS read and CAS
//! write is visible to the CAS — same as any linearisable KV store.
//!
//! ## Operations
//!
//! | Method | Reads? | Writes? | Returns |
//! |--------|--------|---------|---------|
//! | `insert_if_absent` | yes | on miss | `true` = inserted, `false` = already existed |
//! | `compare_and_swap`  | yes | on match | `true` = swapped, `false` = mismatch / not found |
//! | `upsert`            | yes | always | `UpsertResult::Inserted` or `::Updated` |

use crate::engine::TableEngine;
use crate::{EngineResult, EngineRow};

/// Outcome of a [`TableEngine::upsert`] call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpsertResult {
    /// The PK was not present; a new row was inserted.
    Inserted,
    /// The PK already existed; its values were replaced.
    Updated(EngineRow),
}

impl TableEngine {
    /// Insert `(pk, values)` only if `pk` does not already exist.
    ///
    /// Returns `true` when the row was inserted, `false` when a row with that
    /// PK was already present (existing row is left unchanged).
    pub fn insert_if_absent(&self, pk: u64, values: Vec<u64>) -> EngineResult<bool> {
        let _guard = self.cas_lock.lock();
        if self.get(pk)?.is_some() {
            return Ok(false);
        }
        self.insert(pk, values)?;
        Ok(true)
    }

    /// Atomically replace `pk`'s values with `new_values` if the current row
    /// values equal `expected`.
    ///
    /// Returns `true` when the swap was performed, `false` when the row was
    /// not found or its values did not match `expected`.
    ///
    /// Value comparison is element-wise over the full `values` slice.  If
    /// `expected` is shorter than the stored row's values, only the first
    /// `expected.len()` values are compared.
    pub fn compare_and_swap(
        &self,
        pk: u64,
        expected: &[u64],
        new_values: Vec<u64>,
    ) -> EngineResult<bool> {
        let _guard = self.cas_lock.lock();
        let current = match self.get(pk)? {
            None => return Ok(false),
            Some(row) => row,
        };
        let matches = current
            .values
            .iter()
            .zip(expected.iter())
            .all(|(a, b)| a == b);
        if !matches {
            return Ok(false);
        }
        self.insert(pk, new_values)?;
        Ok(true)
    }

    /// Insert `(pk, values)` if absent, or overwrite the existing row if the
    /// PK is already present.
    ///
    /// Returns [`UpsertResult::Inserted`] when the row is new, or
    /// [`UpsertResult::Updated`] carrying the **previous** row when an existing
    /// row was replaced.
    pub fn upsert(&self, pk: u64, values: Vec<u64>) -> EngineResult<UpsertResult> {
        let _guard = self.cas_lock.lock();
        match self.get(pk)? {
            None => {
                self.insert(pk, values)?;
                Ok(UpsertResult::Inserted)
            }
            Some(old) => {
                self.insert(pk, values)?;
                Ok(UpsertResult::Updated(old))
            }
        }
    }
}
