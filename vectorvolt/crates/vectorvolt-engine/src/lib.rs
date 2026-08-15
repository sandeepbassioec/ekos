//! `vectorvolt-engine` — per-table shard layer for VectorVolt.
//!
//! [`TableEngine`] is the orchestrator that wires together the write path
//! ([`vectorvolt_txn::TxnManager`]) and the read path
//! ([`vectorvolt_storage::PartitionReader`]) for a single table.
//!
//! ## Usage
//!
//! ```no_run
//! use vectorvolt_engine::TableEngine;
//!
//! let engine = TableEngine::open(
//!     vec!["id".into(), "age".into()],
//!     "/tmp/my_table",
//!     1024,
//! ).unwrap();
//!
//! engine.insert(1, vec![1, 25]).unwrap();
//! engine.insert(2, vec![2, 30]).unwrap();
//!
//! let row = engine.get(1).unwrap().unwrap();
//! assert_eq!(row.pk, 1);
//! assert_eq!(row.values[1], 25);
//! ```

mod aggregate;
mod compact;
mod cond;
mod engine;
mod error;
mod mvcc;
mod project;
mod registry;
mod row;
mod stats;

pub use aggregate::HistogramBucket;
pub use compact::CompactResult;
pub use cond::UpsertResult;
pub use engine::TableEngine;
pub use engine::{Durability, EngineOptions, FlushResult, WalTip};
pub use error::{EngineError, EngineResult};
pub use mvcc::TxnId;
pub use registry::{SCHEMA_FILE, TableRegistry};
pub use row::EngineRow;
pub use stats::EngineStats;
