//! `vectorvolt-query` — Phase 2 query execution for VectorVolt.
//!
//! Sits on top of `vectorvolt-index` (CuckooMap, SecondaryIndex, ZoneMap,
//! CuckooFilter) and `vectorvolt-storage` (PartitionReader) to provide:
//!
//! | Operation | Index used | Latency target |
//! |---|---|---|
//! | `pk_lookup` | CuckooMap → PartitionReader | ~120 ns + column read |
//! | `range_scan` | ZoneMap prune + PartitionReader | ~10 ns per pruned partition |
//! | `secondary_scan` | SecondaryIndex → CuckooMap → PartitionReader | ~500 ns setup |
//! | `fk_exists` | CuckooFilter | ~150 ns |

pub mod catalog;
pub mod engine;
pub mod error;
pub mod row;

pub use catalog::TableCatalog;
pub use engine::QueryEngine;
pub use error::{QueryError, QueryResult};
pub use row::QueryRow;
