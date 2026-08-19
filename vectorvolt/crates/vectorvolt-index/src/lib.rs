//! `vectorvolt-index` — nanosecond-latency index layer for VectorVolt.
//!
//! ## Data structures
//!
//! | Type | Purpose | Target latency |
//! |---|---|---|
//! | [`CuckooMap`] | Primary-key → [`RowDescriptor`] lookup | ~120 ns |
//! | [`RowDescriptor`] | Bit-packed location: file + partition + row offset | — |
//! | [`SecondaryIndex`] | Column-value → row-ID set (Roaring Bitmap) | ~500 ns/1K rows |
//! | [`ZoneMap`] | Per-partition min/max for range-scan pruning | ~10 ns skip |
//! | [`CuckooFilter`] | Probabilistic FK existence check (zero I/O) | ~150 ns |
//!
//! ## Design invariants
//!
//! - No heap allocation on the lookup hot path (CuckooMap + CuckooFilter).
//! - All types are `Send + Sync` — they live behind a shard-level `RwLock`.
//! - `RowDescriptor` fits in a single `u64` so it occupies one cache-line slot.

pub mod cuckoo_filter;
pub mod cuckoo_map;
pub mod row_descriptor;
pub mod secondary_index;
pub mod zone_map;

pub use cuckoo_filter::CuckooFilter;
pub use cuckoo_map::CuckooMap;
pub use row_descriptor::RowDescriptor;
pub use secondary_index::SecondaryIndex;
pub use zone_map::{ZoneEntry, ZoneMap, ZoneVec};

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum IndexError {
    #[error("CuckooMap is full (capacity {0})")]
    MapFull(usize),

    #[error("CuckooFilter is full (capacity {0})")]
    FilterFull(usize),

    #[error("key not found")]
    NotFound,
}

pub type IndexResult<T> = Result<T, IndexError>;
