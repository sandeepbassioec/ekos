//! `vectorvolt-txn` — Phase 2 transaction manager for VectorVolt.
//!
//! Sits on top of `vectorvolt-wal`, `vectorvolt-index`, and `vectorvolt-storage`
//! to provide a WAL-backed write path with in-memory index maintenance.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use vectorvolt_txn::{TxnManager};
//! use vectorvolt_wal::{WalConfig, WalWriter};
//! use tempfile::TempDir;
//!
//! let wal_dir = TempDir::new().unwrap();
//! let flush_dir = TempDir::new().unwrap();
//! let cfg = WalConfig::default().with_dir(wal_dir.path()).with_fsync(false);
//! let wal = WalWriter::open(cfg).unwrap();
//!
//! let mgr = TxnManager::new(
//!     wal,
//!     vec!["pk".into(), "age".into()],
//!     flush_dir.path(),
//!     1000, // flush after 1000 rows
//! );
//!
//! mgr.insert(1, vec![1, 25]).unwrap();
//! mgr.insert(2, vec![2, 30]).unwrap();
//! let result = mgr.flush().unwrap().expect("rows were buffered");
//! println!("flushed {} rows to {:?}", result.row_count, result.path);
//! ```

pub mod buffer;
pub mod error;
pub mod manager;
pub mod ops;

pub use error::{TxnError, TxnResult};
pub use manager::{FlushResult, TxnManager, WalTip};
pub use ops::Mutation;
pub use vectorvolt_wal::SyncStats;
