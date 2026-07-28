//! ProjectionDispatcher — range/prefix scan engine over memtable + SSTables.
//!
//! This is the read fan-out layer: it merges results from the active memtable
//! and all live SSTables, applies range filtering, resolves tombstones, and
//! returns an ordered stream of live entries.
//!
//! ## Design
//!
//! Read path (newest → oldest, key wins on first occurrence):
//!   1. Active memtable   (most recent, held under RwLock)
//!   2. Immutable memtables (frozen, awaiting flush)  [future]
//!   3. SSTables newest → oldest
//!
//! A key seen as a tombstone at any level is excluded from results even if a
//! value exists at a lower level.  This correctly implements the delete
//! semantic across levels.
//!
//! ## Projection trait
//!
//! Implement `Projection` to transform or filter entries before they are
//! returned — e.g. JSON field extraction, schema projection, redaction.
//! The identity projection is provided as `IdentityProjection`.
//!
//! ## Future work (Phase 4+)
//!
//! - Parallel SSTable reads (one Tokio task per SSTable for large scans)
//! - Bloom filter skip for point lookups
//! - Vector similarity projection for EKOS Knowledge Object queries
//! - WASM-sandboxed user projections

use std::sync::Arc;

use bytes::Bytes;

use crate::StoreResult;
use crate::scan::{ScanEntry, ScanRequest};
use crate::store::KvStore;

// --------------------------------------------------------------------------
// Projection trait
// --------------------------------------------------------------------------

/// A transform applied to each entry before it is returned from a scan.
/// Return `None` to exclude the entry from results.
pub trait Projection: Send + Sync {
    fn apply(&self, entry: ScanEntry) -> Option<ScanEntry>;
}

/// No-op projection — returns every entry unchanged.
pub struct IdentityProjection;

impl Projection for IdentityProjection {
    fn apply(&self, entry: ScanEntry) -> Option<ScanEntry> {
        Some(entry)
    }
}

/// Key-only projection — zeroes the value, useful for existence checks.
pub struct KeyOnlyProjection;

impl Projection for KeyOnlyProjection {
    fn apply(&self, entry: ScanEntry) -> Option<ScanEntry> {
        Some(ScanEntry {
            key: entry.key,
            value: Bytes::new(),
        })
    }
}

/// Predicate projection — keeps only entries where `f(entry)` returns true.
pub struct FilterProjection<F>(pub F)
where
    F: Fn(&ScanEntry) -> bool + Send + Sync;

impl<F> Projection for FilterProjection<F>
where
    F: Fn(&ScanEntry) -> bool + Send + Sync,
{
    fn apply(&self, entry: ScanEntry) -> Option<ScanEntry> {
        if (self.0)(&entry) { Some(entry) } else { None }
    }
}

// --------------------------------------------------------------------------
// ProjectionDispatcher
// --------------------------------------------------------------------------

pub struct ProjectionDispatcher {
    store: Arc<KvStore>,
}

impl ProjectionDispatcher {
    pub fn new(store: Arc<KvStore>) -> Self {
        ProjectionDispatcher { store }
    }

    /// Execute a scan with the identity projection.
    pub fn scan(&self, req: ScanRequest) -> StoreResult<Vec<ScanEntry>> {
        self.scan_with(req, &IdentityProjection)
    }

    /// Execute a scan applying `projection` to every live entry.
    pub fn scan_with(
        &self,
        req: ScanRequest,
        projection: &dyn Projection,
    ) -> StoreResult<Vec<ScanEntry>> {
        // Delegate the merge + range work to KvStore::scan_raw, then apply projection.
        let raw = self.store.scan_raw(&req)?;
        let limit = req.limit.unwrap_or(usize::MAX);

        let mut results = Vec::new();
        for entry in raw {
            if results.len() >= limit {
                break;
            }
            if let Some(projected) = projection.apply(entry) {
                results.push(projected);
            }
        }
        Ok(results)
    }

    /// Convenience: prefix scan with identity projection.
    pub fn scan_prefix(&self, prefix: impl Into<Bytes>) -> StoreResult<Vec<ScanEntry>> {
        self.scan(ScanRequest::prefix(prefix))
    }

    /// Convenience: prefix scan with a limit.
    pub fn scan_prefix_limit(
        &self,
        prefix: impl Into<Bytes>,
        limit: usize,
    ) -> StoreResult<Vec<ScanEntry>> {
        self.scan(ScanRequest::prefix(prefix).with_limit(limit))
    }

    /// Count matching entries without materialising values.
    pub fn count(&self, req: ScanRequest) -> StoreResult<usize> {
        Ok(self.scan_with(req, &KeyOnlyProjection)?.len())
    }

    /// Return only keys (no values) for a scan.
    pub fn keys(&self, req: ScanRequest) -> StoreResult<Vec<Bytes>> {
        Ok(self
            .scan_with(req, &KeyOnlyProjection)?
            .into_iter()
            .map(|e| e.key)
            .collect())
    }
}

// --------------------------------------------------------------------------
// KvStore::scan_raw  (added via impl block in store.rs — declared here)
// --------------------------------------------------------------------------
// The actual merge logic lives in store.rs as `KvStore::scan_raw`.
// We declare the return type here so projection.rs doesn't need to import
// store internals directly.

/// A sorted, deduplicated, tombstone-free list of entries from a scan.
pub type RawScanResult = Vec<ScanEntry>;

// --------------------------------------------------------------------------
// Multi-projection pipeline (compose projections)
// --------------------------------------------------------------------------

/// Apply a chain of projections in order — stops at the first `None`.
pub struct PipelineProjection(pub Vec<Box<dyn Projection>>);

impl Projection for PipelineProjection {
    fn apply(&self, entry: ScanEntry) -> Option<ScanEntry> {
        self.0.iter().try_fold(entry, |e, p| p.apply(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::ScanEntry;

    #[test]
    fn identity_passes_through() {
        let e = ScanEntry {
            key: Bytes::from("k"),
            value: Bytes::from("v"),
        };
        assert_eq!(IdentityProjection.apply(e.clone()), Some(e));
    }

    #[test]
    fn filter_projection() {
        let p = FilterProjection(|e: &ScanEntry| e.key.starts_with(b"user:"));
        let user = ScanEntry {
            key: Bytes::from("user:alice"),
            value: Bytes::from("v"),
        };
        let order = ScanEntry {
            key: Bytes::from("order:1"),
            value: Bytes::from("v"),
        };
        assert!(p.apply(user).is_some());
        assert!(p.apply(order).is_none());
    }

    #[test]
    fn pipeline_short_circuits() {
        let p = PipelineProjection(vec![
            Box::new(FilterProjection(|e: &ScanEntry| e.key != "skip".as_bytes())),
            Box::new(IdentityProjection),
        ]);
        let skip = ScanEntry {
            key: Bytes::from("skip"),
            value: Bytes::from("v"),
        };
        let keep = ScanEntry {
            key: Bytes::from("keep"),
            value: Bytes::from("v"),
        };
        assert!(p.apply(skip).is_none());
        assert!(p.apply(keep).is_some());
    }
}
