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
pub mod aggregate_root;
mod batch;
pub mod catalog_clone;
pub mod catalog_diff;
pub mod catalog_export;
pub mod catalog_gc;
pub mod catalog_health;
pub mod catalog_rename;
pub mod catalog_snapshot;
pub mod catalog_stats;
pub mod catalog_validate;
pub mod catalog_watch;
pub mod cdc;
pub mod cdc_log;
mod compact;
mod cond;
pub mod config;
mod distinct;
mod engine;
mod error;
mod field_update;
mod groupby;
mod index_catalog;
mod mvcc;
mod pagination;
mod project;
mod range_delete;
mod registry;
mod row;
mod scan_where;
pub mod schema;
pub mod schema_registry;
mod secondary_index;
pub mod shard_catalog;
pub mod snapshot;
mod sort;
mod stats;
mod ttl;
mod update_where;
pub mod views;
pub mod yaml_schema;

pub mod system_profile;
mod window;

pub use aggregate::HistogramBucket;
pub use aggregate_root::{AggregateRoot, AggregateRouter};
pub use catalog_clone::clone_table;
pub use catalog_diff::{CatalogDiff, ColumnChange, ColumnChangeKind, TableDiff, diff_catalogs};
pub use catalog_export::{
    CatalogExport, ExportedTable, ImportReport, export_catalog, import_catalog,
};
pub use catalog_gc::{GcReport, run_catalog_gc};
pub use catalog_health::{HealthReport, SnapshotFreshness, check_health};
pub use catalog_rename::rename_table;
pub use catalog_snapshot::{
    CatalogSnapshot, CatalogSnapshotMeta, SnapshotInfo, list_snapshots, restore_snapshot,
    save_snapshot,
};
pub use catalog_stats::{CatalogStatsReport, TableStats, catalog_stats};
pub use catalog_validate::{CatalogValidationReport, TableIssue, validate_catalog};
pub use catalog_watch::{WatchControl, WatchEvent, watch_catalog};
pub use cdc::{ChangeEvent, ChangeKind};
pub use cdc_log::{CdcLog, CdcLogStats};
pub use compact::CompactResult;
pub use cond::UpsertResult;
pub use config::{
    CliOverrides, apply_cli_overrides, apply_env_overrides, apply_env_overrides_with, load_config,
    load_config_auto, load_config_full, load_config_with_profile,
};
pub use engine::TableEngine;
pub use engine::{Durability, EngineOptions, FlushResult, WalTip};
pub use error::{EngineError, EngineResult};
pub use mvcc::TxnId;
pub use pagination::PageResult;
pub use registry::{SCHEMA_FILE, TableRegistry};
pub use row::EngineRow;
pub use schema::{ColumnDef, ColumnType, TableSchema};
pub use schema_registry::SchemaRegistry;
pub use shard_catalog::{ShardCatalog, TableEntry};
pub use snapshot::SnapshotMeta;
pub use stats::EngineStats;
pub use system_profile::SystemProfile;
pub use views::{ViewDefinition, ViewEntry, ViewKind, ViewRegistry};
pub use window::WindowRow;
