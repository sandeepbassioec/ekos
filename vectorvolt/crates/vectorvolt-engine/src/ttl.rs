//! Per-row TTL (time-to-live) expiry.
//!
//! Rows inserted with [`TableEngine::insert_with_ttl`] are tagged with an
//! opaque `expire_at` deadline (caller's epoch — nanoseconds, seconds, or
//! any monotonically increasing `u64`).  The engine never interprets the
//! unit; it only compares `now >= expire_at`.
//!
//! ## API surface
//!
//! | Method | Description |
//! |--------|-------------|
//! | `insert_with_ttl(pk, values, expire_at)` | Insert a row and record its deadline. |
//! | `get_with_ttl(pk, now)` | Like `get`, but returns `None` when the row is expired. |
//! | `scan_with_ttl(now)` | Like `scan`, but suppresses expired rows. |
//! | `purge_expired(now)` | Physically delete all expired rows; returns count removed. |
//! | `ttl_count()` | How many rows currently have a TTL set. |
//! | `row_expire_at(pk)` | Inspect the deadline recorded for a PK (for testing). |
//!
//! ## Design notes
//!
//! The TTL index (`pk → expire_at`) lives in a `BTreeMap` behind a
//! `RwLock` on `TableEngine`.  Reads acquire a shared lock; writes acquire
//! an exclusive lock only for the index update (the underlying `insert` and
//! `delete` use their own locks).  Rows without a TTL entry never expire.
//!
//! `purge_expired` first collects expired PKs under a shared lock, then
//! deletes each row (engine lock) and removes the TTL entry (exclusive lock).
//! This avoids holding the TTL write-lock across I/O.

use crate::engine::TableEngine;
use crate::{EngineResult, EngineRow};

impl TableEngine {
    /// Insert `(pk, values)` and record `expire_at` as its TTL deadline.
    ///
    /// If the row already exists it is overwritten (same as `insert`).  If
    /// the PK already had a different deadline the new one replaces it.
    pub fn insert_with_ttl(&self, pk: u64, values: Vec<u64>, expire_at: u64) -> EngineResult<()> {
        self.insert(pk, values)?;
        self.ttl_index.write().insert(pk, expire_at);
        Ok(())
    }

    /// Fetch a row by primary key, returning `None` if it does not exist **or
    /// if `now >= expire_at`** (i.e. the row has expired).
    ///
    /// Non-TTL rows (inserted with plain `insert`) are never suppressed.
    pub fn get_with_ttl(&self, pk: u64, now: u64) -> EngineResult<Option<EngineRow>> {
        if self.is_expired(pk, now) {
            return Ok(None);
        }
        self.get(pk)
    }

    /// Scan all live rows, suppressing any whose TTL deadline has passed.
    ///
    /// Rows inserted without a TTL are always included.  Order matches `scan`.
    pub fn scan_with_ttl(&self, now: u64) -> EngineResult<Vec<EngineRow>> {
        // Use col 0 (pk) with [0, u64::MAX] to match all rows.
        let all = self.scan_deduped(0, 0, u64::MAX)?;
        let ttl = self.ttl_index.read();
        Ok(all
            .into_iter()
            .filter(|row| {
                ttl.get(&row.pk)
                    .map(|&deadline| now < deadline)
                    .unwrap_or(true) // no TTL → never expires
            })
            .collect())
    }

    /// Delete all rows whose `expire_at <= now`.
    ///
    /// Returns the number of rows physically removed.  Silently ignores rows
    /// that have already been deleted by another path.
    pub fn purge_expired(&self, now: u64) -> EngineResult<usize> {
        // Collect expired PKs under a shared lock to minimise lock contention.
        let expired: Vec<u64> = {
            let ttl = self.ttl_index.read();
            ttl.iter()
                .filter(|(_, deadline)| now >= **deadline)
                .map(|(pk, _)| *pk)
                .collect()
        };

        let mut removed = 0usize;
        for pk in &expired {
            // delete() returns Err(PkNotFound) when the row is already gone —
            // treat that as a successful purge (the row is gone either way).
            match self.delete(*pk) {
                Ok(_) => removed += 1,
                Err(crate::EngineError::Txn(vectorvolt_txn::TxnError::PkNotFound(_))) => {
                    removed += 1; // row already absent — still remove TTL entry
                }
                Err(e) => return Err(e),
            }
        }

        // Remove purged entries from the TTL index.
        {
            let mut ttl = self.ttl_index.write();
            for pk in &expired {
                ttl.remove(pk);
            }
        }

        Ok(removed)
    }

    /// Number of rows that currently have a TTL deadline recorded.
    pub fn ttl_count(&self) -> usize {
        self.ttl_index.read().len()
    }

    /// Return the deadline recorded for `pk`, or `None` if it has no TTL.
    pub fn row_expire_at(&self, pk: u64) -> Option<u64> {
        self.ttl_index.read().get(&pk).copied()
    }

    // ── internal helper ───────────────────────────────────────────────────────

    /// `true` when `pk` has a TTL deadline and `now >= deadline`.
    fn is_expired(&self, pk: u64, now: u64) -> bool {
        self.ttl_index
            .read()
            .get(&pk)
            .map(|&deadline| now >= deadline)
            .unwrap_or(false)
    }
}
