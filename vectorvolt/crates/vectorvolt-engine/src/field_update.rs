//! Field-level atomic updates.
//!
//! These operations read an existing row, modify one column, and re-insert —
//! all under [`TableEngine::cas_lock`] so no concurrent writer can interleave
//! between the read and the write.
//!
//! ## API
//!
//! | Method | Returns | Notes |
//! |--------|---------|-------|
//! | `update_field(pk, col, new_val)` | `Option<u64>` old value | `None` when PK absent |
//! | `increment_field(pk, col, delta)` | `Option<u64>` new value | wrapping add; `None` when PK absent |
//! | `decrement_field(pk, col, delta)` | `Option<u64>` new value | saturating sub; `None` when PK absent |
//!
//! All three return `Ok(None)` (not an error) when the PK does not exist,
//! leaving the table unchanged.  Schema-mismatch and I/O errors propagate as
//! `Err`.
//!
//! ## Atomicity
//!
//! Each operation holds `cas_lock` for the duration of its read-modify-write
//! sequence.  Concurrent CAS, upsert, or other field-update calls on the same
//! (or different) PKs are serialised against each other through the same mutex.
//! This is a coarse but correct serialisation point; a fine-grained per-PK
//! lock is a future optimisation.

use crate::EngineResult;
use crate::engine::TableEngine;

impl TableEngine {
    /// Replace the value at `col_idx` in the row identified by `pk`.
    ///
    /// Returns `Some(old_value)` if the row existed and was updated, or
    /// `None` if the PK was not found (no write is performed).
    ///
    /// # Errors
    ///
    /// Returns `Err` when `col_idx` is out of range, or when the underlying
    /// read or write fails.
    pub fn update_field(&self, pk: u64, col_idx: usize, new_val: u64) -> EngineResult<Option<u64>> {
        self.check_col(col_idx)?;
        let _guard = self.cas_lock.lock();
        let Some(row) = self.get(pk)? else {
            return Ok(None);
        };
        let old_val = row.values[col_idx];
        let mut new_values = row.values.clone();
        new_values[col_idx] = new_val;
        self.insert(pk, new_values)?;
        Ok(Some(old_val))
    }

    /// Add `delta` to the value at `col_idx` in the row identified by `pk`
    /// (wrapping on overflow).
    ///
    /// Returns `Some(new_value)` if the row existed and was updated, or
    /// `None` if the PK was not found (no write is performed).
    pub fn increment_field(
        &self,
        pk: u64,
        col_idx: usize,
        delta: u64,
    ) -> EngineResult<Option<u64>> {
        self.check_col(col_idx)?;
        let _guard = self.cas_lock.lock();
        let Some(row) = self.get(pk)? else {
            return Ok(None);
        };
        let new_val = row.values[col_idx].wrapping_add(delta);
        let mut new_values = row.values.clone();
        new_values[col_idx] = new_val;
        self.insert(pk, new_values)?;
        Ok(Some(new_val))
    }

    /// Subtract `delta` from the value at `col_idx` in the row identified by
    /// `pk` (saturating at zero on underflow).
    ///
    /// Returns `Some(new_value)` if the row existed and was updated, or
    /// `None` if the PK was not found (no write is performed).
    pub fn decrement_field(
        &self,
        pk: u64,
        col_idx: usize,
        delta: u64,
    ) -> EngineResult<Option<u64>> {
        self.check_col(col_idx)?;
        let _guard = self.cas_lock.lock();
        let Some(row) = self.get(pk)? else {
            return Ok(None);
        };
        let new_val = row.values[col_idx].saturating_sub(delta);
        let mut new_values = row.values.clone();
        new_values[col_idx] = new_val;
        self.insert(pk, new_values)?;
        Ok(Some(new_val))
    }
}
