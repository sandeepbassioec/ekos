//! # vectorvolt-wal
//!
//! The VectorVolt Write-Ahead Log — immutable, append-only, SHA3-256 hash-chained.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use vectorvolt_wal::{WalConfig, WalWriter, WalReader, WalCursor};
//!
//! // Write
//! let cfg = WalConfig::default().with_dir("/var/lib/vectorvolt/wal");
//! let writer = WalWriter::open(cfg.clone()).unwrap();
//! let tip = writer.append(b"hello".to_vec()).unwrap();
//! writer.flush().unwrap();
//! println!("committed sequence {}", tip.sequence);
//!
//! // Read
//! let reader = WalReader::open(cfg);
//! let frames = reader.read_all().unwrap();
//! ```
//!
//! ## GroupCommit (high-throughput)
//!
//! ```rust,no_run
//! use vectorvolt_wal::{WalConfig, WalWriter, GroupCommitBatcher};
//! use tokio::time::Duration;
//!
//! #[tokio::main]
//! async fn main() {
//!     let cfg = WalConfig::default().with_dir("/var/lib/vectorvolt/wal");
//!     let writer = WalWriter::open(cfg).unwrap();
//!     let batcher = GroupCommitBatcher::start(
//!         writer, Duration::from_micros(500), 4 * 1024 * 1024, 8192
//!     );
//!     let fut = batcher.submit(b"payload".to_vec()).unwrap();
//!     let tip = fut.wait().await.unwrap();
//! }
//! ```

pub mod config;
pub mod error;
pub mod frame;
pub mod group_commit;
pub mod metrics;
pub mod reader;
pub mod seal;
pub mod segment;
pub mod snapshot;
pub mod sync_commit;
pub mod uring_writer;
pub mod writer;

// Convenience re-exports.
pub use config::WalConfig;
pub use error::{WalError, WalResult};
pub use frame::{Frame, GENESIS_HASH, PayloadType};
pub use group_commit::{BatcherConfig, CommitFuture, GroupCommitBatcher};
pub use metrics::{BatcherMetrics, MetricsSnapshot};
pub use reader::{WalCursor, WalReader, WalStats};
pub use segment::{SegmentReader, list_segment_ids, segment_path};
pub use snapshot::SnapshotMarker;
pub use sync_commit::{SyncCommitter, SyncStats};
pub use writer::{WalTip, WalWriter};
